# 10. 文本输入

文本框是最常用的控件, 但GPUI**没有**内置的输入框组件 —— 既没有 `TextInput::new()`, 也没有现成的 `<input>`. 想要一个能打字的文本框, 得自己把三块拼起来: 实现 `EntityInputHandler` 接住键盘和输入法, 写一个自定义 Element 负责排版和绘制, 再把两者接到窗口的输入链路里.

## 10.1 概述: 一个输入框由什么组成

对照 HTML 的 `<input>`, 浏览器替你做掉的事在GPUI里都要自己实现:

| 职责 | HTML | GPUI里的做法 |
| :--- | :--- | :--- |
| 接收键盘/IME 输入 | 浏览器内置 | 实现 `EntityInputHandler` trait(见 10.2) |
| 文本排版与绘制 | 浏览器内置 | 自定义 Element 三阶段(见 10.4、10.5) |
| 光标、选区 | 浏览器内置 | 自己画, 状态存在实体里(见 10.7) |
| 快捷键(删除/移动/复制) | 浏览器内置 | 动作 + 按键绑定(见第 9 章) |
| 鼠标点选、拖选 | 浏览器内置 | 鼠标事件 + 坐标换算(见 10.9) |

本章以能编译的示例 [`examples/input.rs`](../examples/input.rs) 为准: `TextInput` 实体持有文本和选区状态, `TextElement` 自定义 Element 把它们画出来, 键盘行为拆成一个个动作:

```rust
actions!(
    text_input,
    [Backspace, Delete, Left, Right, SelectLeft, SelectRight, SelectAll, Home, End, Paste, Cut, Copy],
);
```

## 10.2 EntityInputHandler trait

`EntityInputHandler` 是GPUI文本输入的核心 trait: 实体实现它之后, 键盘输入、输入法(IME)组合、系统级的复制粘贴都会回调进来. 平台层真正接触的是 `InputHandler` trait, `EntityInputHandler` 面向实体(省掉手动 `update` 的样板), 由 `ElementInputHandler` 包一层转成 `InputHandler` (见 10.6).

**所有 range/offset 都是 UTF-16 码元偏移**, 这是系统文本协议(macOS `NSTextInputClient` 等)的要求, 和 Rust `String` 的字节偏移不是一回事. 实体内部按字节存文本时必须自己换算(见 10.3).

必须实现的方法:

| 方法 | 说明 |
| :--- | :--- |
| `text_for_range(range_utf16, adjusted_range, window, cx) -> Option<String>` | 取指定范围的文本; `adjusted_range` 回填实际取到的范围 |
| `selected_text_range(ignore_disabled_input, window, cx) -> Option<UTF16Selection>` | 当前选区; `UTF16Selection { range, reversed }` 的 `reversed` 表示光标在选区起始端 |
| `marked_text_range(&self, window, cx) -> Option<Range<usize>>` | IME 正在组合(未上屏)的文本范围 |
| `unmark_text(window, cx)` | 结束 IME 组合, 清掉标记 |
| `replace_text_in_range(range, text, window, cx)` | 把 `range` 换成 `text`; `range` 为 `None` 时表示替换选区(或组合段) |
| `replace_and_mark_text_in_range(range, new_text, new_selected_range, window, cx)` | 插入 `new_text` 并标记为 IME 组合段, 同时指定新光标/选区 |
| `bounds_for_range(range_utf16, element_bounds, window, cx) -> Option<Bounds<Pixels>>` | 范围的屏幕坐标, 系统用它定位 IME 候选框 |
| `character_index_for_point(point, window, cx) -> Option<usize>` | 屏幕坐标 → UTF-16 字符下标(点击定位光标用) |

有默认实现、按需覆盖的方法:

| 方法 | 说明 |
| :--- | :--- |
| `paste(item, window, cx)` | 系统发起的粘贴(如网页的 `paste` 事件), 默认插纯文本 |
| `set_selected_text_range(range_utf16, window, cx)` | 系统代为移动选区(如拖动系统选区手柄) |
| `selection_bounds_for_range(...) -> Vec<Bounds<Pixels>>` | 选区的屏幕矩形(每行/每段一个, 不是每字符一个) |
| `text_length_utf16(window, cx) -> Option<usize>` | 文档总长(UTF-16 码元数) |
| `accepts_text_input(window, cx) -> bool` | 是否接收文本输入, 默认 `true` |
| `text_input_configuration(window, cx) -> TextInputConfiguration` | 自动纠错、软键盘回车键等偏好 |
| `text_input_editable_range(window, cx) -> Option<Range<usize>>` | 允许输入法读写的范围, 默认不限制 |

**NOTE**: IME 组合态**没有** `mark_text()` 这个方法 —— 查询组合段用 `marked_text_range()`, 写入组合段用 `replace_and_mark_text_in_range()`, 结束组合用 `unmark_text()`.

**NOTE**: 方法的上下文参数是 `&mut Context<Self>`(实体上下文), **没有** `AppContext`/`ViewContext` 这两个写法; `selected_text_range()` 返回的是 `UTF16Selection` 而**不是** `Range<usize>`.

## 10.3 TextInput: 状态与 EntityInputHandler 实现

先看状态放哪. 示例的 `TextInput` 实体把文本、选区、IME 组合段、上一帧的排版结果都存下来:

```rust
struct TextInput {
    focus_handle: FocusHandle,          // 焦点句柄(见第 8 章)
    content: SharedString,              // 文本内容(UTF-8 字节)
    placeholder: SharedString,          // 空文本时的占位提示
    selected_range: Range<usize>,       // 选区(字节偏移); start == end 时光标
    selection_reversed: bool,           // 光标是否在选区的起始端
    marked_range: Option<Range<usize>>, // IME 组合段(字节偏移)
    last_layout: Option<ShapedLine>,    // 上一帧的排版结果, 换算坐标用
    last_bounds: Option<Bounds<Pixels>>, // 上一帧的元素边界
    is_selecting: bool,                 // 鼠标拖选中
}
```

**UTF-16 换算**是绕不开的一环: trait 传进来的是 UTF-16 偏移, 实体存的是字节偏移. 示例用 `offset_to_utf16()`/`offset_from_utf16()` 及其区间版互相转换, 例如取文本:

```rust
fn text_for_range(
    &mut self,
    range_utf16: Range<usize>,              // IME 给的是 UTF-16 范围
    actual_range: &mut Option<Range<usize>>, // 回填实际取到的范围
    _window: &mut Window,
    _cx: &mut Context<Self>,
) -> Option<String> {
    let range = self.range_from_utf16(&range_utf16); // 先换算成字节范围
    actual_range.replace(self.range_to_utf16(&range));
    Some(self.content[range].to_string())
}
```

插入/删除都走 `replace_text_in_range()`, 先定"替换谁": 显式传的范围 → IME 组合段 → 当前选区:

```rust
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
        (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..]).into();
    self.selected_range = range.start + new_text.len()..range.start + new_text.len();
    self.marked_range.take(); // 组合段一并清掉
    window.notify_text_input_changed(&self.focus_handle); // 同步给系统输入法
    cx.notify();
}
```

返回选区就是把 `self.range_to_utf16(&self.selected_range)` 和 `selection_reversed` 包成 `UTF16Selection`. 两个容易踩的细节: 光标移动按**字素簇**(grapheme)走(`Backspace` 要借 `unicode-segmentation` 的 `grapheme_indices()` 求前一个边界), 不然退格会把 emoji 或带音标的字符拆半; 删到开头无可删时响一声 `window.play_system_bell()`.

## 10.4 自定义 Element 三阶段渲染

文本要精确控制排版和绘制, 光用 `div()` 不够, 需要实现 `Element` trait. 一帧内每个 Element 依次走三个阶段:

| 阶段 | 方法 | 职责 |
| :--- | :--- | :--- |
| 1. 布局请求 | `request_layout()` | 向 Taffy 布局系统要尺寸, 返回 `(LayoutId, RequestLayoutState)` |
| 2. 绘制前 | `prepaint()` | 布局完成后拿到最终 `bounds`, 做坐标计算、准备绘制素材, 返回 `PrepaintState` |
| 3. 绘制 | `paint()` | 把内容画到屏幕; 输入处理器也在这一步注册(见 10.6) |

两个关联类型决定阶段之间传什么状态: `RequestLayoutState` 是 `request_layout` 产出、后两阶段读取的; `PrepaintState` 是 `prepaint` 产出、`paint` 读取的. 没有状态就用 `()`:

```rust
struct TextElement {
    input: Entity<TextInput>, // 实体句柄, 各阶段读写它的状态
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState; // 排版结果 + 光标/选区的绘制素材
    // request_layout/prepaint/paint 见 10.5
}
```

**NOTE**: 实现 `Element` 的同时还要实现 `IntoElement`(通常 `type Element = Self`, `into_element()` 返回 `self`), 这一步**没有**默认实现可省.

💡 提示: `prepaint` 常见的活儿是 `window.insert_hitbox(bounds, ...)` 登记命中区域; `TextElement` 不用插命中箱, 鼠标事件由外层 `div()` 的监听器处理(见 10.9).

## 10.5 TextElement 的三阶段实现

**request_layout** 只决定自己要多大: 宽度撑满父元素, 高度取一行行高.

```rust
fn request_layout(
    &mut self,
    _id: Option<&GlobalElementId>,
    _inspector_id: Option<&gpui::InspectorElementId>,
    window: &mut Window,
    cx: &mut App,
) -> (LayoutId, Self::RequestLayoutState) {
    let mut style = Style::default();
    style.size.width = relative(1.).into();          // 宽度撑满父元素: width: 100%
    style.size.height = window.line_height().into(); // 高度 = 一行行高
    (window.request_layout(style, [], cx), ())
}
```

**prepaint** 是重头戏: 算好"画什么"和"画在哪", 存进 `PrepaintState`. 顺序是: 选显示文本(空则占位) → 构造 `TextRun`(IME 组合段加下划线) → `shape_line()` 排版 → 用排版结果算光标/选区矩形.

```rust
// 1. 空文本时画占位提示(淡色)
let (display_text, text_color) = if content.is_empty() {
    (input.placeholder.clone(), hsla(0., 0., 0., 0.2))
} else {
    (content, style.color)
};

// 2. IME 组合段拆成独立 TextRun 加下划线(表示"还没上屏"), 其余部分共用 run
let runs = if let Some(marked_range) = input.marked_range.as_ref() {
    vec![
        TextRun { len: marked_range.start, ..run.clone() },
        TextRun { len: marked_range.end - marked_range.start,
                  underline: Some(UnderlineStyle { color: Some(run.color), thickness: px(1.0), wavy: false }), ..run.clone() },
        TextRun { len: display_text.len() - marked_range.end, ..run },
    ]
    .into_iter().filter(|run| run.len > 0).collect()
} else {
    vec![run]
};

// 3. 排版成单行 ShapedLine, 后续坐标换算全靠它
let line = window.text_system().shape_line(display_text, font_size, &runs, None);

// 4. 无选区: 2px 宽的光标竖线: width: 2px; 有选区: 半透明矩形, 不画光标
let cursor_pos = line.x_for_index(cursor); // 字节偏移 → 行内 x 坐标
let (selection, cursor) = if selected_range.is_empty() {
    let cursor_bounds = Bounds::new(point(bounds.left() + cursor_pos, bounds.top()),
                                    size(px(2.), bounds.bottom() - bounds.top()));
    (None, Some(fill(cursor_bounds, gpui::blue())))
} else {
    // 选区起止的 x 坐标同样来自 line.x_for_index
    (Some(fill(Bounds::from_corners(/* 起点 */, /* 终点 */), rgba(0x3311ff30))), None)
};
```

**paint** 把准备好的素材兑现, 顺手注册输入处理器、缓存排版结果:

```rust
fn paint(/* ... */) {
    // 1. 注册本帧的输入处理器(见 10.6)
    let focus_handle = self.input.read(cx).focus_handle.clone();
    window.handle_input(&focus_handle, ElementInputHandler::new(bounds, self.input.clone()), cx);

    // 2. 画选区、文本、光标
    if let Some(selection) = prepaint.selection.take() {
        window.paint_quad(selection)
    }
    let line = prepaint.line.take().unwrap();
    line.paint(bounds.origin, window.line_height(), gpui::TextAlign::Left, None, window, cx).unwrap();
    if focus_handle.is_focused(window) && let Some(cursor) = prepaint.cursor.take() {
        window.paint_quad(cursor); // 只有聚焦时才画光标
    }

    // 3. 缓存排版结果和边界, 9.9 的坐标换算要用
    self.input.update(cx, |input, _cx| {
        input.last_layout = Some(line);
        input.last_bounds = Some(bounds);
    });
}
```

**NOTE**: 绘制文本**没有** `cx.paint_text(bounds, text, style)` 这个方法 —— 先 `window.text_system().shape_line(text, font_size, &runs, None)` 排版成 `ShapedLine`, 再调它的 `paint()` 画出来. `shape_line` 只排单行, 文本带换行会触发断言.

## 10.6 注册输入处理器

`EntityInputHandler` 实现好之后不会自动生效, 要在**每帧的 paint 阶段**把实体包成 `ElementInputHandler` 交给窗口(上一节 paint 的第 1 步):

```rust
window.handle_input(
    &focus_handle, // 焦点句柄: 只有它持有焦点时处理器才生效
    ElementInputHandler::new(bounds, self.input.clone()), // 桥接 EntityInputHandler → InputHandler
    cx,
);
```

- `ElementInputHandler::new(element_bounds, entity)` 两个参数: 元素在窗口中的边界(供 `bounds_for_range` 用)和实体句柄.
- 注册只对**当前这一帧**有效, 下一帧要重新注册 —— 所以放在 `paint()` 里, 每帧都走.
- 内部会检查 `focus_handle.is_focused()`: 没聚焦就什么都不注册.

实体自己改了文本或选区(而不是系统输入法改的), 要额外通知系统文本服务, 否则输入法内部状态会不同步:

| 方法 | 何时调用 |
| :--- | :--- |
| `window.notify_text_input_changed(&focus_handle)` | 文本内容变了(插入、删除后) |
| `window.notify_text_selection_changed(&focus_handle)` | 只是选区/光标移动了 |
| `window.request_virtual_keyboard()` | 用户点击输入框时, 唤起被收起的软键盘 |

**NOTE**: 注册用的是 paint 阶段的 `window.handle_input(...)`, **没有** `register_input_handler()` 这个方法, 也**没有** `cx.on_window_opened(... handle_input(cx.entity().downgrade()))` 这种"窗口创建时注册一次"的写法.

## 10.7 焦点与光标

光标要不要画、输入要不要接, 全看焦点(焦点系统见第 8 章). `TextInput` 实现 `Focusable` 暴露 `focus_handle`, 渲染时用 `.track_focus()` 绑定:

```rust
// render() 里
div()
    .key_context("TextInput") // 动作派发的上下文名
    .track_focus(&self.focus_handle(cx)) // 绑定焦点句柄: 元素可聚焦
    .cursor(CursorStyle::IBeam) // 悬停时鼠标变工字形: cursor: text
```

示例启动后要主动聚焦一次(`window.focus(&handle, cx)`), 否则窗口打开时打不了字. 光标位置**不是**简单取 `selected_range.start` —— 反向选区(shift 向左选)时头在起始端, 用 `selection_reversed` 标记, `cursor_offset()` 据此返回 `start` 或 `end`.

移动光标(`move_to`)和扩展选区(`select_to`)是两个方法, 但都收束到同一套状态更新: 改 `selected_range`、`window.notify_text_selection_changed()` 通知系统、`cx.notify()` 重绘. `Left`/`Right`/`Home`/`End` 走 `move_to`, `SelectLeft`/`SelectRight`/`SelectAll` 走 `select_to`, 按键绑定和普通动作一模一样(见第 9 章).

💡 提示: 示例的光标**不闪烁(blink)**, 聚焦时一直实心. 要闪烁效果, 用 `cx.background_executor().timer(...)` 定时翻转一个可见性标志再 `cx.notify()` 即可.

## 10.8 剪贴板操作

复制、剪切、粘贴自己写动作实现, 剪贴板读写在 `App` 上(`Context` 可解引用过去):

```rust
fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
        self.replace_text_in_range(None, &text.replace("\n", " "), window, cx);
    }
}

fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
    if !self.selected_range.is_empty() {
        cx.write_to_clipboard(ClipboardItem::new_string(
            self.content[self.selected_range.clone()].to_string(),
        ));
    }
}
```

- 读: `cx.read_from_clipboard() -> Option<ClipboardItem>`, 再 `.text() -> Option<String>` 取纯文本(剪贴板还可能有图片等其他类型).
- 写: `cx.write_to_clipboard(ClipboardItem::new_string(s))`, 要构造 `ClipboardItem`.
- `cut` 就是先 `copy` 再 `replace_text_in_range(None, "", ...)` 删掉选区; 快捷键用 `KeyBinding::new("cmd-c", Copy, None)` 这类绑定接到动作上.

💡 提示: 系统自己发起的粘贴(比如网页里的 `paste` 事件、移动端的粘贴菜单)不走上面的动作, 而是回调 `EntityInputHandler::paste(item, ...)`, 默认实现就是取纯文本调 `replace_text_in_range`.

**NOTE**: 剪贴板 API 是 `cx.read_from_clipboard()`/`cx.write_to_clipboard(ClipboardItem)`, **没有** `cx.clipboard_read()`/`cx.clipboard_write("...")` 这两个方法.

## 10.9 鼠标选择文本

鼠标交互三件事: 点击定位光标、拖拽扩展选区、shift 点击扩展选区. 先定状态转换:

```rust
fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
    self.is_selecting = true;
    window.request_virtual_keyboard(); // 唤起软键盘(移动端)
    if event.modifiers.shift {
        self.select_to(self.index_for_mouse_position(event.position), window, cx); // 扩展选区
    } else {
        self.move_to(self.index_for_mouse_position(event.position), window, cx) // 定位光标
    }
}

fn on_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
    if self.is_selecting {
        self.select_to(self.index_for_mouse_position(event.position), window, cx);
    }
}

fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
    self.is_selecting = false;
}
```

三个回调都挂在外层 `div()` 上, 拖出元素外松开也要结束选择:

```rust
div()
    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up)) // 拖到元素外松开
    .on_mouse_move(cx.listener(Self::on_mouse_move))
```

**坐标 → 文本下标**的换算靠 paint 阶段缓存下来的 `last_bounds` 和 `last_layout`: x 找最近的字形位置, y 超出上/下边界就取首/尾:

```rust
fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
    if self.content.is_empty() {
        return 0;
    }
    let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref()) else {
        return 0;
    };
    if position.y < bounds.top() { return 0; }
    if position.y > bounds.bottom() { return self.content.len(); }
    line.closest_index_for_x(position.x - bounds.left()) // x 坐标 → 最近的字节偏移
}
```

**NOTE**: 点击定位**没有** `pixel_to_char_index()` 这种现成方法 —— 示例用 `ShapedLine::closest_index_for_x()` 自己换算, 这也是 `last_layout`/`last_bounds` 存在的原因. 鼠标回调的签名是 `(&mut self, &MouseDownEvent, &mut Window, &mut Context<Self>)`, **没有** `ViewContext`, 也**不是**直接收一个 `Point<Pixels>`.

## 10.10 文本输入速查表

| 类别 | API | 说明 |
| :--- | :--- | :--- |
| **输入协议** | `impl EntityInputHandler` | 键盘/IME/选区回调, range 全是 UTF-16 偏移 |
| **IME 组合** | `marked_text_range()` `replace_and_mark_text_in_range()` `unmark_text()` | 查询/设置/结束组合段 |
| **注册** | `window.handle_input(&handle, ElementInputHandler::new(bounds, entity), cx)` | 每帧 paint 阶段调用 |
| **同步系统** | `window.notify_text_input_changed()` `window.notify_text_selection_changed()` | 内容/选区变化后通知系统输入法 |
| **软键盘** | `window.request_virtual_keyboard()` | 点击输入框时唤起 |
| **三阶段** | `request_layout()` `prepaint()` `paint()` | 自定义 Element 的生命周期 |
| **排版** | `window.text_system().shape_line()` | 单行排版成 `ShapedLine` |
| **坐标换算** | `ShapedLine::x_for_index()` `closest_index_for_x()` | 字节偏移 ↔ 行内 x 坐标 |
| **绘制** | `fill(bounds, color)` `window.paint_quad()` `ShapedLine::paint()` | 选区、光标、文本 |
| **剪贴板** | `cx.read_from_clipboard()` `cx.write_to_clipboard(ClipboardItem::new_string(...))` | 读写纯文本 |
| **焦点** | `cx.focus_handle()` `.track_focus()` `handle.is_focused(window)` | 光标只在聚焦时绘制 |
| **字素边界** | `unicode_segmentation::grapheme_indices()` | 光标移动不拆 emoji |

完整示例: [`cargo run --example input`](../examples/input.rs)

## 10.11 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`input`](../examples/input.rs) | 完整输入框: `TextInput` + `TextElement` 三阶段渲染, 光标、选区、剪贴板、鼠标选字, 并实时显示按键(见 10.1–10.9) |
