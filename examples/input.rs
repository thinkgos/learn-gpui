//! 文本输入框示例：GPUI **没有**内置输入框，这里自己拼一个 ——
//! `TextInput` 实体持有文本和选区，实现 `EntityInputHandler` 接住键盘 / IME，
//! 再写自定义 Element `TextElement` 把它们画出来（详见第 10 章）。
//!
//! 运行方式：`cargo run --example input`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 顶栏：左边显示当前键盘布局名，右边一个 "Reset" 按钮清空输入框；
//!   2. 中间是输入框本体（白底），空文本时显示灰色占位提示 "Type here..."；
//!   3. 下面按新到旧列出最近的按键记录，用来观察键位绑定有没有生效。
//!
//! 关键概念：
//!   1. `EntityInputHandler`：键盘、IME、系统复制粘贴的回调入口，
//!      里面所有 range / offset 都是 **UTF-16 码元偏移**，和 Rust 字节偏移要互相换算；
//!   2. 自定义 Element 三阶段渲染：`request_layout`（要尺寸）→ `prepaint`
//!      （算坐标、备绘制素材）→ `paint`（画出来 + 注册输入处理器）；
//!   3. 动作 + 按键绑定：退格、移动、复制粘贴拆成一个个动作（见第 9 章）；
//!   4. 光标、选区自己画，靠 `ShapedLine` 在「字节偏移 ↔ 行内 x 坐标」之间换算。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, KeyBinding, Keystroke, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds,
    WindowOptions, actions, black, div, fill, hsla, opaque_grey, point, prelude::*, px, relative,
    rgb, rgba, size, white, yellow,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;
// 字素簇（grapheme）切分：光标按字素移动，不把 emoji / 带音标的字符拆半
use unicode_segmentation::*;

// 用 `actions!` 宏声明本示例的动作（见 9.1）：每个名字生成一个零大小的单元结构体。
// 键位绑定（见 run_example）把按键接到这些动作上，render() 里的 `.on_action()` 再接住它们。
actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
        Quit,
    ]
);

/// 输入框本体：文本、选区、IME 组合段都存在这里。
///
/// 下面几个 range 字段存的都是**字节偏移**（Rust `String` 的下标），
/// 而 `EntityInputHandler` 传进来的是 UTF-16 偏移，读写前要换算（见 10.3）。
struct TextInput {
    /// 焦点句柄（见第 8 章）：光标画不画、输入接不接，全看它有没有焦点。
    focus_handle: FocusHandle,
    /// 文本内容（UTF-8 字节）。`SharedString` 内部引用计数，克隆很便宜。
    content: SharedString,
    /// 空文本时显示的占位提示（淡色灰字）。
    placeholder: SharedString,
    /// 选区（字节偏移）；start == end 时光标就停在那里。
    selected_range: Range<usize>,
    /// 光标是否在选区的起始端（shift 向左选时为 true，见 10.7）。
    selection_reversed: bool,
    /// IME 正在组合、还没上屏的文本范围（字节偏移）。
    marked_range: Option<Range<usize>>,
    /// 上一帧的排版结果，坐标换算全靠它（见 10.5）。
    last_layout: Option<ShapedLine>,
    /// 上一帧的元素边界，鼠标坐标换算文本下标时用（见 10.9）。
    last_bounds: Option<Bounds<Pixels>>,
    /// 是否正在鼠标拖选中。
    is_selecting: bool,
}

impl TextInput {
    /// 左方向键：没选区就左移一个字素边界，有选区就收拢到选区起点。
    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }

    /// 右方向键：没选区就右移一个字素边界，有选区就收拢到选区终点。
    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }

    /// shift + 左：向左扩展选区。
    fn select_left(&mut self, _: &SelectLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }

    /// shift + 右：向右扩展选区。
    fn select_right(&mut self, _: &SelectRight, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }

    /// 全选：先把光标挪到开头，再一路扩展到结尾。
    fn select_all(&mut self, _: &SelectAll, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }

    /// Home：光标跳到行首。
    fn home(&mut self, _: &Home, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }

    /// End：光标跳到行尾。
    fn end(&mut self, _: &End, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }

    /// 退格：没选区就先选中前一个字素再删（避免把 emoji 拆半），有选区直接删选区。
    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                window.play_system_bell(); // 已经在开头，无可删，响一声系统提示音
                return;
            }
            self.select_to(prev, window, cx) // 临时把「前一个字素」变成选区
        }
        self.replace_text_in_range(None, "", window, cx) // 删掉选区
    }

    /// Delete：和 `backspace` 对称，删的是光标后面那个字素。
    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                window.play_system_bell(); // 已经在末尾，无可删
                return;
            }
            self.select_to(next, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    /// 鼠标按下：进入拖选状态；按住 shift 扩展选区，否则只定位光标。
    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        window.request_virtual_keyboard(); // 唤起被收起的软键盘（移动端）

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), window, cx)
        }
    }

    /// 鼠标抬起：结束拖选。拖到元素外松开也会调到它（见 render() 的 `on_mouse_up_out`）。
    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    /// 鼠标移动：拖选中就把选区扩展到当前位置。
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), window, cx);
        }
    }

    /// 打开系统的字符检视器（输入特殊符号、emoji 用）。
    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    /// 粘贴：读剪贴板纯文本，把换行换成空格（单行输入框放不下换行）。
    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
        }
    }

    /// 复制：把选区文本写进剪贴板（没选区就什么都不做）。
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }
    /// 剪切：先复制再删掉选区。
    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    /// 移动光标（不扩展选区）：选区塌缩成一个空区间，光标落在 `offset`。
    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        // 只是光标/选区动了，通知系统文本服务即可（不用 notify_text_input_changed）
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    /// 光标在哪个字节偏移上：反向选区（shift 向左选）时头在起始端，否则在终点。
    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    /// 鼠标坐标 → 文本字节偏移。
    ///
    /// **没有** `pixel_to_char_index()` 这种现成方法，靠上一帧缓存的
    /// `last_layout` / `last_bounds` 自己换算（见 10.9）。
    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        // 还没画过（没有排版结果）就当点在开头
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0; // 点在文本上方：取行首
        }
        if position.y > bounds.bottom() {
            return self.content.len(); // 点在文本下方：取行尾
        }
        line.closest_index_for_x(position.x - bounds.left()) // x 坐标 → 最近的字形位置
    }

    /// 扩展选区到 `offset`（不动光标那一端），并处理方向翻转。
    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset // 光标在起点，动的是起点
        } else {
            self.selected_range.end = offset
        };
        // 跨过另一端时把区间调回 start < end，并翻转光标方向
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify()
    }

    /// UTF-16 偏移 → 字节偏移：按字符累加两种编码的宽度。
    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16(); // emoji 这类字符占 2 个 UTF-16 码元
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    /// 字节偏移 → UTF-16 偏移：`offset_from_utf16` 的反向。
    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    /// 字节区间 → UTF-16 区间。
    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    /// UTF-16 区间 → 字节区间。
    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    /// `offset` 之前的那个字素边界（退格用），没有就返回 0。
    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    /// `offset` 之后的那个字素边界（Delete 用），没有就返回文本末尾。
    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    /// 清空输入框，回到初始状态。
    fn reset(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        self.last_layout = None;
        self.last_bounds = None;
        self.is_selecting = false;
        // 文本内容变了，同步给系统输入法
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }
}

// ===========================================================================
// 一、EntityInputHandler：键盘 / IME / 系统复制粘贴的回调（见 10.2）
// ===========================================================================
// 这些方法里的 range / offset 全是 **UTF-16 码元偏移**（系统文本协议的要求），
// 实体内部存的是字节偏移，每个方法都要先换算再动手。
impl EntityInputHandler for TextInput {
    /// 取指定范围的文本；`actual_range` 回填实际取到的范围（换算回 UTF-16）。
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    /// 当前选区；`reversed` 表示光标在选区起始端。
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    /// 系统代为设置选区（比如拖动系统选区手柄）。
    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.range_from_utf16(&range_utf16);
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    /// 文档总长（UTF-16 码元数）。
    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.content.encode_utf16().count())
    }

    /// IME 正在组合（未上屏）的文本范围。
    ///
    /// **NOTE**: IME 组合态**没有** `mark_text()` 这个方法 —— 查询用本方法，
    /// 写入用 `replace_and_mark_text_in_range()`，结束用 `unmark_text()`。
    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    /// 结束 IME 组合，清掉标记。
    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// 把 `range` 换成 `new_text`；`range` 为 `None` 时表示替换组合段或选区。
    ///
    /// 「替换谁」的优先级：显式传的范围 → IME 组合段 → 当前选区。
    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        // 光标落在新文本末尾（start + new_text.len()，都是字节偏移）
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take(); // 组合段一并清掉
        // 文本内容变了，同步给系统输入法
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// 插入 `new_text` 并标记为 IME 组合段，同时指定新光标 / 选区。
    ///
    /// 和 `replace_text_in_range` 的差别只在两点：新文本会挂上组合段标记，
    /// 光标位置由 `new_selected_range_utf16` 说了算（它是**相对新文本**的范围）。
    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        // 先算出插入点的 UTF-16 起点，再把「相对新文本」的选区换算成绝对区间
        let insertion_start_utf16 = self.offset_to_utf16(range.start);
        let inserted_length_utf16 = new_text.encode_utf16().count();
        self.selected_range = new_selected_range_utf16
            .map(|selected| {
                self.range_from_utf16(
                    &(insertion_start_utf16 + selected.start.min(inserted_length_utf16)
                        ..insertion_start_utf16 + selected.end.min(inserted_length_utf16)),
                )
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection_reversed = false;

        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    /// 范围的屏幕坐标，系统拿它定位 IME 候选框。
    ///
    /// `bounds` 是元素自己的边界，加上排版结果里每个下标的 x 坐标就是范围矩形。
    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        // 排版结果已经过期（文本被改过）就没法定位
        if last_layout.text != self.content {
            return None;
        }
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    /// 选区的屏幕矩形（每行/每段一个，不是每字符一个）。
    /// 这里是单行输入框，直接复用 `bounds_for_range` 的结果。
    fn selection_bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        self.bounds_for_range(range_utf16, bounds, window, cx)
            .into_iter()
            .collect()
    }

    /// 屏幕坐标 → UTF-16 字符下标（系统点击定位光标时回调）。
    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        // 先把窗口坐标转成元素内坐标
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        if last_layout.text != self.content {
            return None;
        }
        let utf8_index = last_layout.closest_index_for_x(line_point.x);
        Some(self.offset_to_utf16(utf8_index))
    }
}

// ===========================================================================
// 二、自定义 Element：三阶段渲染（见 10.4、10.5）
// ===========================================================================

/// 自定义 Element：把 `TextInput` 的文本、光标、选区画出来。
///
/// 光用 `div()` 画不了这么精确的排版，所以自己实现 `Element` 的三个阶段。
struct TextElement {
    /// 实体句柄，三个阶段都通过它读写输入框状态。
    input: Entity<TextInput>,
}

/// `prepaint` 产出、`paint` 消费的绘制素材（见 10.4）。
struct PrepaintState {
    /// 排好版的单行文本；paint 直接画，坐标换算也靠它。
    line: Option<ShapedLine>,
    /// 光标竖线（无选区时才有）。
    cursor: Option<PaintQuad>,
    /// 选区高亮矩形（有选区时才有）。
    selection: Option<PaintQuad>,
}

// 实现 `Element` 的同时还要实现 `IntoElement`，这一步**没有**默认实现可省。
impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self // 自己就是 Element，直接返回
    }
}

impl Element for TextElement {
    /// 两个关联类型决定阶段之间传什么状态（见 10.4）：
    /// `RequestLayoutState` 是 request_layout 产出、后两阶段读取的。
    type RequestLayoutState = ();
    /// `PrepaintState` 是 prepaint 产出、paint 读取的。
    type PrepaintState = PrepaintState;

    /// 不需要稳定的元素 id（`on_click` 那类交互方法用不到）。
    fn id(&self) -> Option<ElementId> {
        None
    }

    /// 调试用的源码位置，这里不提供。
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    /// 阶段 1：向布局系统要尺寸（见 10.5）。
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into(); // 宽度撑满父元素：width: 100%
        style.size.height = window.line_height().into(); // 高度 = 一行行高
        (window.request_layout(style, [], cx), ())
    }

    /// 阶段 2：布局完成后拿到最终 `bounds`，算好「画什么、画在哪」。
    ///
    /// 顺序是：选显示文本（空则占位）→ 构造 TextRun（组合段加下划线）
    /// → `shape_line()` 排版 → 用排版结果算光标 / 选区矩形。
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // 1. 从实体读出这一帧要用的状态（只读）
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        // 2. 空文本时改画占位提示，颜色淡一点（20% 透明度的黑）
        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), hsla(0., 0., 0., 0.2))
        } else {
            (content, style.color)
        };

        // 3. TextRun 描述一段文本的样式；没有组合段时整行共用一个 run
        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        // 4. 有 IME 组合段就拆成三段：前段 + 带下划线的组合段 + 后段，
        //    下划线表示「还没上屏」
        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0) // 长度为 0 的段丢掉
            .collect()
        } else {
            vec![run]
        };

        // 5. 排版成单行 ShapedLine，后续坐标换算全靠它
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        // 6. 无选区：画 2px 宽的光标竖线；有选区：画半透明矩形，不画光标
        let cursor_pos = line.x_for_index(cursor); // 字节偏移 → 行内 x 坐标
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    gpui::blue(),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x3311ff30),
                )),
                None,
            )
        };
        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    /// 阶段 3：把准备好的素材兑现，顺手注册输入处理器、缓存排版结果。
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // 1. 注册本帧的输入处理器（见 10.6）：只对当前帧有效，所以每帧都要做。
        //    `ElementInputHandler` 把 EntityInputHandler 桥接成平台层的 InputHandler。
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle, // 只有它持有焦点时处理器才生效
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        // 2. 从下往上画：选区 → 文本 → 光标
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .unwrap();

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor); // 只有聚焦时才画光标
        }

        // 3. 缓存排版结果和边界，鼠标坐标换算要用（见 10.9）
        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

// ===========================================================================
// 三、输入框的可视部分：样式 + 动作 + 鼠标事件
// ===========================================================================
impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div() // 创建一个 div 元素，类似 HTML 里的 <div>
            .flex() // 弹性布局：display: flex
            .key_context("TextInput") // 动作派发的上下文名，键位绑定可用它限定生效范围
            .track_focus(&self.focus_handle(cx)) // 绑定焦点句柄：元素可聚焦（见第 8 章）
            .cursor(CursorStyle::IBeam) // 悬停时鼠标变工字形：cursor: text
            // 下面一串 .on_action() 把按键动作接到上面的方法（见第 9 章）
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            // 鼠标交互：按下定位/扩展选区，移动时拖选，抬起结束
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up)) // 拖到元素外松开也算抬起
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .bg(rgb(0xeeeeee)) // 背景色：background-color: #eeeeee
            .line_height(px(30.)) // 行高：line-height: 30px
            .text_size(px(24.)) // 字号：font-size: 24px
            .child(
                div() // 白色输入区，文本由自定义的 TextElement 画
                    .h(px(30. + 4. * 2.)) // 高度 = 行高 + 上下各 4px 内边距：height: 38px
                    .w_full() // 宽度撑满父元素：width: 100%
                    .p(px(4.)) // 内边距：padding: 4px
                    .bg(white()) // 背景色：background-color: #ffffff
                    .child(TextElement { input: cx.entity() }),
            )
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ===========================================================================
// 四、整个示例的界面：顶栏 + 输入框 + 按键记录
// ===========================================================================

/// 示例根组件：把输入框实体挂进来，再显示最近的按键。
struct InputExample {
    /// 输入框实体句柄。
    text_input: Entity<TextInput>,
    /// 最近的按键记录，用来观察键位绑定有没有生效。
    recent_keystrokes: Vec<Keystroke>,
    /// 示例自己的焦点句柄（见第 8 章）。
    focus_handle: FocusHandle,
}

impl Focusable for InputExample {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl InputExample {
    /// 点 "Reset" 按钮：清掉按键记录和输入框内容。
    fn on_reset_click(&mut self, _: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.recent_keystrokes.clear();
        self.text_input
            .update(cx, |text_input, cx| text_input.reset(window, cx));
        cx.notify();
    }
}

impl Render for InputExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0xaaaaaa)) // 背景色：background-color: #aaaaaa
            .track_focus(&self.focus_handle(cx)) // 绑定焦点句柄：元素可聚焦
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .size_full() // 宽高撑满窗口：width: 100%; height: 100%
            .child(
                div() // 顶栏：键盘布局名 + Reset 按钮
                    .bg(white()) // 背景色：background-color: #ffffff
                    .border_b_1() // 下边框：border-bottom-width: 1px
                    .border_color(black()) // 边框颜色：border-color: #000000
                    .flex() // 弹性布局：display: flex
                    .flex_row() // 横向排列：flex-direction: row
                    .justify_between() // 两端对齐：justify-content: space-between
                    .child(format!("Keyboard {}", cx.keyboard_layout().name())) // 当前键盘布局名
                    .child(
                        div() // Reset 按钮
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(black()) // 边框颜色：border-color: #000000
                            .px_2() // 左右内边距：padding-left: 0.5rem; padding-right: 0.5rem (8px)
                            .bg(yellow()) // 背景色：background-color: #ffff00
                            .child("Reset")
                            .hover(|style| {
                                style
                                    .bg(yellow().blend(opaque_grey(0.5, 0.5))) // 悬停时背景加深
                                    .cursor_pointer() // 悬停时鼠标变手型：cursor: pointer
                            })
                            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_reset_click)),
                    ),
            )
            .child(self.text_input.clone()) // 输入框本体
            .children(self.recent_keystrokes.iter().rev().map(|ks| {
                // 按键记录倒序显示；按下的是可打印字符时，额外标出 key_char
                format!(
                    "{:} {}",
                    ks.unparse(cx.platform_info().os),
                    if let Some(key_char) = ks.key_char.as_ref() {
                        format!("-> {:?}", key_char)
                    } else {
                        "".to_owned()
                    }
                )
            }))
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // 启动主循环，在闭包里做初始化
    application().run(|cx: &mut App| {
        // 加载字体；失败就直接退出
        if !example_support::load_fonts(cx) {
            return;
        }
        // 窗口位置和尺寸：300x300，出现在屏幕中央
        let bounds = Bounds::centered(None, size(px(300.0), px(300.0)), cx);
        // 键位绑定：快捷键 → 动作（见 9.3）。"cmd-a" 是全选，"shift-left" 是向左扩展选区，
        // 第三个参数是上下文谓词，这里传 None 表示全局生效
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
        ]);

        // 打开一个新窗口，窗口内容是 InputExample 根组件
        let window = cx
            .open_window(
                WindowOptions {
                    // 窗口使用上面算出来的位置和尺寸
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    // 输入框实体：初始为空文本，占位提示 "Type here..."
                    let text_input = cx.new(|cx| TextInput {
                        focus_handle: cx.focus_handle(),
                        content: "".into(),
                        placeholder: "Type here...".into(),
                        selected_range: 0..0,
                        selection_reversed: false,
                        marked_range: None,
                        last_layout: None,
                        last_bounds: None,
                        is_selecting: false,
                    });
                    cx.new(|cx| InputExample {
                        text_input,
                        recent_keystrokes: vec![],
                        focus_handle: cx.focus_handle(),
                    })
                },
            )
            .unwrap();
        // 订阅全局按键记录：每按一次键就把 Keystroke 存进根组件，界面跟着刷新
        let view = window.update(cx, |_, _, cx| cx.entity()).unwrap();
        cx.observe_keystrokes(move |ev, _, cx| {
            view.update(cx, |view, cx| {
                view.recent_keystrokes.push(ev.keystroke.clone());
                cx.notify();
            })
        })
        .detach();
        // 键盘布局变了也要重绘（顶栏显示着布局名）
        cx.on_keyboard_layout_change({
            move |cx| {
                window.update(cx, |_, _, cx| cx.notify()).ok();
            }
        })
        .detach();

        // 激活窗口并聚焦到输入框，否则窗口打开时打不了字（见 10.7）
        window
            .update(cx, |view, window, cx| {
                window.focus(&view.text_input.focus_handle(cx), cx);
                cx.activate(true);
            })
            .unwrap();
        // 全局监听 Quit 动作：这是动作冒泡到顶后才轮到的处理器（见 9.4）
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main。
#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}

// ---------- 测试 ----------
// 跑法：cargo test --example input --features test-support
// 测试模块必须带 `--features test-support` 才会编译，否则会安静地跑 0 个用例。
#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    /// 造一个指定初始文本的输入框。
    fn input(content: &'static str, cx: &mut Context<TextInput>) -> TextInput {
        TextInput {
            focus_handle: cx.focus_handle(),
            content: content.into(),
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    /// 选区和长度对外用 UTF-16 计数：内部字节偏移要跟着换算。
    #[gpui::test]
    fn native_selection_and_length_use_utf16(cx: &mut TestAppContext) {
        let window = cx.add_window(|_, cx| input("e🙂漢", cx));
        window
            .update(cx, |input, window, cx| {
                // "e🙂漢" 共 4 个 UTF-16 码元（🙂 占 2 个）
                assert_eq!(input.text_length_utf16(window, cx), Some(4));
                input.set_selected_text_range(1..3, window, cx);
                assert_eq!(input.selected_range, 1..5); // UTF-16 1..3 → 字节 1..5
                assert_eq!(
                    input.selected_text_range(false, window, cx).unwrap().range,
                    1..3
                );
                // 越界的范围会被收敛到文本末尾
                input.set_selected_text_range(3..99, window, cx);
                assert_eq!(input.selected_range, 5..8);
            })
            .unwrap();
    }

    /// IME 组合段里的选区是**相对新插入文本**的范围，不是绝对区间。
    #[gpui::test]
    fn marked_selection_is_relative_to_inserted_text(cx: &mut TestAppContext) {
        let window = cx.add_window(|_, cx| input("A🙂Z", cx));
        window
            .update(cx, |input, window, cx| {
                input.replace_and_mark_text_in_range(Some(1..3), "é🙂", Some(1..3), window, cx);
                assert_eq!(input.content, "Aé🙂Z");
                assert_eq!(input.marked_range, Some(1..7)); // 组合段是字节区间
                // 传入的 1..3 是相对 "é🙂" 的，落到 "Aé🙂Z" 里就是字节 3..7
                assert_eq!(input.selected_range, 3..7);

                // 组合中继续输入：替换掉整个组合段，新的组合段跟着变
                input.replace_and_mark_text_in_range(None, "漢", Some(1..1), window, cx);
                assert_eq!(input.content, "A漢Z");
                assert_eq!(input.marked_range, Some(1..4));
                assert_eq!(input.selected_range, 4..4);

                // 空文本结束组合：标记清掉
                input.replace_and_mark_text_in_range(None, "", Some(0..0), window, cx);
                assert_eq!(input.content, "AZ");
                assert_eq!(input.marked_range, None);
                assert_eq!(input.selected_range, 1..1);
            })
            .unwrap();
    }

    /// 超长单行文本的选区也只产出一个几何矩形（不是每字符一个）。
    #[gpui::test]
    fn long_single_line_selection_uses_one_geometry_fragment(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, cx| input("", cx));
        view.update(cx, |input, cx| {
            input.content = "a".repeat(10_000).into();
            cx.notify();
        });
        cx.run_until_parked(); // 等一帧，让输入框真的画出来
        view.update_in(cx, |input, window, cx| {
            let bounds = input.last_bounds.expect("input was painted");
            let rectangles = input.selection_bounds_for_range(0..10_000, bounds, window, cx);
            assert_eq!(rectangles.len(), 1);
            assert!(rectangles[0].size.width > px(0.));
        });
    }
}
