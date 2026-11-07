//! 焦点样式对照示例：`.focus()` 和 `.focus_visible()` 的区别（见 8.2）。
//! 两者对应 CSS 的 `:focus` 和 `:focus-visible`，都是「样式方法」而不是两种焦点状态。
//!
//! 运行方式：`cargo run --example focus_visible`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：一行标题、一行提示消息，下面三个对照按钮：
//!   1. 只用 `.focus()`：不管鼠标点击还是 Tab，聚焦就显示黄框；
//!   2. 只用 `.focus_visible()`：只有键盘（Tab）聚焦才显示绿框，点击不显示；
//!   3. 两者叠加：点击显示黄框，Tab 显示绿框。
//!
//! 关键点（见 8.2）：`focus_visible()` 区分的是**输入方式**（键盘还是鼠标），
//! 不是「程序聚焦 vs 键盘聚焦」—— 这正是 CSS `:focus-visible` 的语义。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, Div, ElementId, FocusHandle, KeyBinding, SharedString, Stateful, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// 定义三个动作（不带数据的空动作）：Tab 前进、Shift-Tab 后退、退出。
// 按键绑定在 run_example 里做（见第 9 章）。
actions!(example, [Tab, TabPrev, Quit]);

/// 示例组件：三个按钮的焦点句柄 + 顶部提示文字。
struct Example {
    /// 根元素的焦点句柄：启动时聚焦它，Tab / Shift-Tab 动作才会派发到下面的 on_action。
    focus_handle: FocusHandle,
    /// 三个按钮各自的（焦点句柄, 按钮文案），Tab 顺序设置写在构造里。
    items: Vec<(FocusHandle, &'static str)>,
    /// 顶部提示文字，跟着点击 / 按键更新。
    message: SharedString,
}

impl Example {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let items = vec![
            (
                // 句柄版 Tab 顺序设置：tab_index 是 Tab 序编号，tab_stop(true) 允许 Tab 停上来
                cx.focus_handle().tab_index(1).tab_stop(true),
                "Button with .focus() - always shows border when focused",
            ),
            (
                cx.focus_handle().tab_index(2).tab_stop(true),
                "Button with .focus_visible() - only shows border with keyboard",
            ),
            (
                cx.focus_handle().tab_index(3).tab_stop(true),
                "Button with both .focus() and .focus_visible()",
            ),
        ];

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx); // 启动时把焦点给根元素（见 8.1）

        Self {
            focus_handle,
            items,
            message: SharedString::from(
                "Try clicking vs tabbing! Click shows no border, Tab shows border.",
            ),
        }
    }

    /// Tab：在 Tab 序里让焦点前进一位（见 8.3）。
    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
        self.message = SharedString::from("Pressed Tab - focus-visible border should appear!");
    }

    /// Shift-Tab：在 Tab 序里让焦点后退一位。
    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
        self.message =
            SharedString::from("Pressed Shift-Tab - focus-visible border should appear!");
    }

    /// 退出应用。
    fn on_quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }
}

/// 实现 `Render`：三个按钮只在焦点样式上不同，共用同一个按钮底子。
impl Render for Example {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 三个按钮共用的底子：蓝底白字的圆角条
        fn button_base(id: impl Into<ElementId>, label: &'static str) -> Stateful<Div> {
            div()
                .id(id) // 元素唯一 id：on_click 等交互方法要求元素带 id
                .h_16() // 高度：height: 4rem (64px)
                .w_full() // 宽度撑满：width: 100%
                .flex() // 弹性布局：display: flex
                .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                .bg(gpui::rgb(0x2563eb)) // 背景色：background-color: #2563eb
                .text_color(gpui::white()) // 文字颜色：color: #ffffff
                .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                .cursor_pointer() // 手型光标：cursor: pointer
                .hover(|style| style.bg(gpui::rgb(0x1d4ed8))) // 悬停时换深一点的底色：类似 :hover
                .child(label)
        }

        div()
            .id("app")
            .track_focus(&self.focus_handle) // 绑定焦点句柄：根元素可聚焦，下面的 on_action 才收得到动作
            .on_action(cx.listener(Self::on_tab)) // 动作沿焦点路径派发：Tab 前进
            .on_action(cx.listener(Self::on_tab_prev)) // Shift-Tab 后退
            .on_action(cx.listener(Self::on_quit)) // cmd-q 退出
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .p_8() // 四周内边距：padding: 2rem (32px)
            .gap_6() // 子元素之间的间距：gap: 1.5rem (24px)
            .bg(gpui::rgb(0xf3f4f6)) // 背景色：background-color: #f3f4f6
            .child(
                div() // 标题
                    .text_2xl() // 字号加大：font-size: 1.5rem; line-height: 2rem
                    .font_weight(gpui::FontWeight::BOLD) // 加粗：font-weight: bold
                    .text_color(gpui::rgb(0x111827)) // 文字颜色：color: #111827
                    .child("CSS focus-visible Demo"),
            )
            .child(
                div() // 提示消息条：显示上一次点击 / 按键的结果
                    .p_4() // 四周内边距：padding: 1rem (16px)
                    .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                    .bg(gpui::rgb(0xdbeafe)) // 背景色：background-color: #dbeafe
                    .text_color(gpui::rgb(0x1e3a8a)) // 文字颜色：color: #1e3a8a
                    .child(self.message.clone()),
            )
            .child(
                div() // 三个对照组
                    .flex() // 弹性布局：display: flex
                    .flex_col() // 纵向排列：flex-direction: column
                    .gap_4() // 对照组之间的间距：gap: 1rem (16px)
                    .child(
                        div() // 对照组 1：只用 .focus()
                            .flex() // 弹性布局：display: flex
                            .flex_col() // 纵向排列：flex-direction: column
                            .gap_2() // 小标题和按钮的间距：gap: 0.5rem (8px)
                            .child(
                                div() // 小标题
                                    .text_sm() // 字号缩小：font-size: 0.875rem; line-height: 1.25rem
                                    .font_weight(gpui::FontWeight::BOLD) // 加粗：font-weight: bold
                                    .text_color(gpui::rgb(0x374151)) // 文字颜色：color: #374151
                                    .child("1. Regular .focus() - always visible:"),
                            )
                            .child(
                                button_base("button1", self.items[0].1)
                                    .track_focus(&self.items[0].0) // 绑定这个按钮的焦点句柄
                                    .focus(|style| {
                                        // .focus()：持有焦点就生效，不管焦点是怎么来的 —— 类似 CSS 的 :focus
                                        style.border_4().border_color(gpui::rgb(0xfbbf24)) // 4px 黄框：border-width: 4px; border-color: #fbbf24
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.message =
                                            "Clicked button 1 - focus border is visible!".into();
                                        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                                    })),
                            ),
                    )
                    .child(
                        div() // 对照组 2：只用 .focus_visible()
                            .flex() // 弹性布局：display: flex
                            .flex_col() // 纵向排列：flex-direction: column
                            .gap_2() // 小标题和按钮的间距：gap: 0.5rem (8px)
                            .child(
                                div() // 小标题
                                    .text_sm() // 字号缩小：font-size: 0.875rem; line-height: 1.25rem
                                    .font_weight(gpui::FontWeight::BOLD) // 加粗：font-weight: bold
                                    .text_color(gpui::rgb(0x374151)) // 文字颜色：color: #374151
                                    .child("2. New .focus_visible() - only keyboard:"),
                            )
                            .child(
                                button_base("button2", self.items[1].1)
                                    .track_focus(&self.items[1].0) // 绑定这个按钮的焦点句柄
                                    .focus_visible(|style| {
                                        // .focus_visible()：持有焦点「且最近一次输入来自键盘」才生效
                                        // —— 鼠标点击聚焦不显示，Tab 聚焦才显示，对应 CSS 的 :focus-visible
                                        style.border_4().border_color(gpui::rgb(0x10b981)) // 4px 绿框：border-width: 4px; border-color: #10b981
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.message =
                                            "Clicked button 2 - no border! Try Tab instead.".into();
                                        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                                    })),
                            ),
                    )
                    .child(
                        div() // 对照组 3：两个样式方法叠加
                            .flex() // 弹性布局：display: flex
                            .flex_col() // 纵向排列：flex-direction: column
                            .gap_2() // 小标题和按钮的间距：gap: 0.5rem (8px)
                            .child(
                                div() // 小标题
                                    .text_sm() // 字号缩小：font-size: 0.875rem; line-height: 1.25rem
                                    .font_weight(gpui::FontWeight::BOLD) // 加粗：font-weight: bold
                                    .text_color(gpui::rgb(0x374151)) // 文字颜色：color: #374151
                                    .child(
                                        "3. Both .focus() (yellow) and .focus_visible() (green):",
                                    ),
                            )
                            .child(
                                button_base("button3", self.items[2].1)
                                    .track_focus(&self.items[2].0) // 绑定这个按钮的焦点句柄
                                    .focus(|style| {
                                        // 鼠标点击聚焦：只有 :focus 分支生效 —— 黄框
                                        style.border_4().border_color(gpui::rgb(0xfbbf24)) // 4px 黄框：border-width: 4px; border-color: #fbbf24
                                    })
                                    .focus_visible(|style| {
                                        // 键盘聚焦：两个分支都生效，后写的覆盖同名样式 —— 绿框
                                        style.border_4().border_color(gpui::rgb(0x10b981)) // 4px 绿框：border-width: 4px; border-color: #10b981
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.message =
                                            "Clicked button 3 - yellow border. Tab shows green!"
                                                .into();
                                        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                                    })),
                            ),
                    ),
            )
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 按键绑定（见第 9 章）：tab 前进、shift-tab 后退、cmd-q 退出。
        // 第三个参数是上下文约束，None 表示全局生效。
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
            KeyBinding::new("cmd-q", Quit, None),
        ]);

        // 计算窗口位置：让 800x600 的窗口出现在屏幕中央。
        // None 表示不指定显示在哪个显示器上，用默认的。
        let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default()                                 // 其余选项都用默认值
            },
            |window, cx| cx.new(|cx| Example::new(window, cx)),
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        // 让这个窗口成为活动窗口（抢到输入焦点）。
        cx.activate(true);
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
    gpui_platform::web_init();
    run_example();
}
