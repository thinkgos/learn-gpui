//! 浮层 Popover 示例：用 `deferred()` + `anchored()` 把内容抬到独立的浮层上，
//! 演示「默认展开」「点击展开」「两级嵌套」三种浮层，以及点击外部关闭。
//!
//! 运行方式：`cargo run --example popover`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 顶部一排两个按钮："Opened Popover" 默认就挂着一个展开的浮层；
//!      "Open Popover" 点击后展开一级浮层，浮层里还有个 "Child Popover"
//!      按钮能再开二级浮层；
//!   2. 底部一段说明文字和红黄蓝绿四条彩线，用来验证浮层确实盖在正文上。
//!
//! 三个关键 API（详见第 16 章）：
//!   1. `deferred()`：布局留在原地，绘制推迟到正文画完之后，浮层永远在最上层；
//!   2. `anchored()`：把浮层锚定到触发元素的某个角上，并贴边收回防止出窗口；
//!   3. `on_mouse_down_out`：鼠标在浮层外按下时关闭，配合 `.when()` 控制显隐。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    Anchor, App, Context, Div, Hsla, Stateful, Window, WindowOptions, anchored, deferred, div,
    prelude::*, px,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例组件：两个布尔状态分别控制一级 / 二级浮层的显隐。
struct HelloWorld {
    /// 一级浮层（"Open Popover" 按钮的浮层）的显隐状态。
    open: bool,
    /// 二级浮层（"Child Popover" 按钮的浮层）的显隐状态。
    secondary_open: bool,
}

/// 造一个黑底白字的小按钮。
/// `id` 是元素的唯一标识：`on_click` 这类交互方法只在带 id 的元素上可用。
fn button(id: &'static str) -> Stateful<Div> {
    div()
        .id(id) // 元素唯一 id：on_click 等交互方法要求元素带 id
        .bg(gpui::black()) // 背景色：background-color: #000000
        .text_color(gpui::white()) // 文字颜色：color: #ffffff
        .px_3() // 左右内边距：padding-left: 0.75rem; padding-right: 0.75rem (12px)
        .py_1() // 上下内边距：padding-top: 0.25rem; padding-bottom: 0.25rem (4px)
}

/// 浮层卡片的统一样式：白底、圆角、阴影、细边框。
fn popover() -> Div {
    div()
        .flex() // 弹性布局：display: flex
        .flex_col() // 纵向排列：flex-direction: column
        .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
        .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
        .shadow_lg() // 大阴影：box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)
        .p_3() // 四周内边距：padding: 0.75rem (12px)
        .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
        .bg(gpui::white()) // 背景色：background-color: #ffffff
        .text_color(gpui::black()) // 文字颜色：color: #000000
        .border_1() // 1 像素边框：border-width: 1px
        .text_sm() // 字号缩小：font-size: 0.875rem; line-height: 1.25rem
        .border_color(gpui::black().opacity(0.1)) // 边框颜色带 10% 不透明度：border-color: rgb(0 0 0 / 0.1)
}

/// 底部的彩条占位元素，专门用来验证浮层确实盖在正文内容之上。
fn line(color: Hsla) -> Div {
    div().w(px(480.)).h_2().bg(color.opacity(0.25))
    // w():   宽度：width: 480px
    // h_2(): 2 号高度：height: 0.5rem (8px)
    // bg():  25% 不透明度的背景色：background-color: <颜色> / 0.25
}

impl HelloWorld {
    /// 二级浮层："Child Popover" 按钮 + 嵌套的 deferred 浮层。
    /// GPUI 已支持嵌套 deferred —— 浮层里还能再开浮层。
    fn render_secondary_popover(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        button("secondary-btn")
            .mt_2() // 上外边距：margin-top: 0.5rem (8px)
            .child("Child Popover")
            .on_click(cx.listener(|this, _, _, cx| {
                // listener 闭包收四个参数：自己的状态、事件、window、cx
                this.secondary_open = true; // 置开：下面的 .when 会把浮层挂进元素树
                cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
            }))
            .when(self.secondary_open, |this| {
                // 条件渲染：二级浮层开着才把它挂进元素树
                this.child(
                    deferred(
                        // deferred(): 延迟绘制 —— 布局留在原地，绘制推迟到正文画完之后，
                        // 所以浮层永远盖在正文上，不会被后渲染的内容盖住。
                        anchored()
                            // anchored(): 锚定容器，把浮层钉在本按钮包围盒的参考点上
                            .anchor(Anchor::TopLeft) // 锚点：浮层的左上角贴着参考点
                            .snap_to_window_with_margin(px(8.)) // 贴边收回时留 8px 余量：保证浮层不出窗口
                            .child(
                                popover()
                                    .child("This is second level Popover with nested deferred!")
                                    .bg(gpui::white()) // 背景色：background-color: #ffffff
                                    .border_color(gpui::blue()) // 边框换蓝色，和一级浮层区分开：border-color: #0000ff
                                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                        // 鼠标在浮层「外」按下时收起二级浮层
                                        this.secondary_open = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .priority(2), // 浮层堆叠优先级：数字大的画在上层，二级(2)盖在一级(1)之上
                )
            })
    }
}

/// 实现 `Render`：顶部一排按钮和它们的浮层，底部一段验证用的正文。
impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .gap_3() // 子元素之间的间距：gap: 0.75rem (12px)
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .text_color(gpui::black()) // 文字颜色：color: #000000
            .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .child(
                div() // 顶部一排两个按钮
                    .flex() // 横向弹性布局：display: flex
                    .flex_row() // 显式横向排列：flex-direction: row
                    .gap_4() // 两个按钮之间的间距：gap: 1rem (16px)
                    .child(
                        // 第一个按钮：浮层默认就展开，不用点
                        button("popover0").child("Opened Popover").child(
                            deferred(
                                anchored()
                                    .anchor(Anchor::TopLeft) // 锚点：挂在按钮左上角
                                    .snap_to_window_with_margin(px(8.)) // 贴边收回留 8px 余量
                                    .child(popover().w_96().gap_3().child(
                                        // w_96(): 96 号宽度：width: 24rem (384px)
                                        // gap_3(): 子元素之间的间距：gap: 0.75rem (12px)
                                        "This is a default opened Popover, \
                                        we can use deferred to render it \
                                        in a floating layer.",
                                    )),
                            )
                            .priority(0), // 默认 0：画在浮层堆的最底层
                        ),
                    )
                    .child(
                        button("popover1")
                            .child("Open Popover")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.open = true; // 置开：下面的 .when 会把一级浮层挂进元素树
                                cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                            }))
                            .when(self.open, |this| {
                                // 条件渲染：open 为 true 才挂浮层。
                                // deferred() 没有「条件渲染」参数，显隐就靠 .when 自己控制。
                                this.child(
                                    deferred(
                                        anchored()
                                            .anchor(Anchor::TopLeft) // 锚点：挂在按钮左上角
                                            .snap_to_window_with_margin(px(8.)) // 贴边收回留 8px 余量
                                            .child(
                                                popover()
                                                    .w_96() // 宽度：width: 24rem (384px)
                                                    .gap_3() // 子元素之间的间距：gap: 0.75rem (12px)
                                                    .child(
                                                        "This is first level Popover, \
                                                   we can use deferred to render it \
                                                   in a floating layer.\n\
                                                   Click outside to close.",
                                                    )
                                                    .when(!self.secondary_open, |this| {
                                                        // 二级浮层开着时要让位：
                                                        // 否则点二级会被当成「点外面」，把一级一起关掉
                                                        this.on_mouse_down_out(cx.listener(
                                                            |this, _, _, cx| {
                                                                this.open = false; // 在浮层外按下：收起一级浮层
                                                                cx.notify();
                                                            },
                                                        ))
                                                    })
                                                    // 浮层要放在触发按钮的内容「之后」渲染，
                                                    // 确保它在同层元素的上层。
                                                    .child(
                                                        self.render_secondary_popover(window, cx),
                                                    ),
                                            ),
                                    )
                                    .priority(1), // 一级浮层层级：画在二级(2)的下面
                                )
                            }),
                    ),
            )
            .child(
                // 底部正文：配合下面的彩条，验证浮层浮在内容之上
                "Here is an example text rendered, \
                to ensure the Popover will float above this contents.",
            )
            .children([
                line(gpui::red()),    // 红色彩条
                line(gpui::yellow()), // 黄色彩条
                line(gpui::blue()),   // 蓝色彩条
                line(gpui::green()),  // 绿色彩条
            ])
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
        // 开窗口：不指定位置尺寸，用 WindowOptions::default() 的默认值。
        cx.open_window(WindowOptions::default(), |_, cx| {
            // 根视图：两级浮层的初始状态都是收起。
            cx.new(|_| HelloWorld {
                open: false,
                secondary_open: false,
            })
        })
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
