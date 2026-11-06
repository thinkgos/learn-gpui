//! 文字排版示例：演示 GPUI 的文字对齐、下划线 / 删除线、
//! 不换行时的溢出裁剪，以及一段文字内的富文本混排。
//!
//! 运行方式：`cargo run --example text_layout`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面从上到下分五块：
//!   1. 对齐：左 / 中 / 右三行文字
//!   2. 下划线：同样三种对齐
//!   3. 删除线：同样三种对齐
//!   4. 溢出裁剪：两个定宽盒子，文字不换行、放不下就直接裁掉
//!   5. 富文本：`StyledText` 在 "ABCD" 里给单个字符套不同样式
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, FontStyle, FontWeight, StyledText, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例组件（结构体名字沿用上游的 HelloWorld，其实和 Hello World 无关）：
/// 界面就是一列文字排版演示块，从上往下依次是
/// 对齐、下划线、删除线、溢出裁剪、富文本混排。
struct HelloWorld {}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HelloWorld {
    /// 界面是一个纵向排列的 div，每个 `.child(...)` 就是一块演示。
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 最外层容器，一列演示块从上往下排：
        // bg(...):     背景色：background-color: #ffffff
        // flex():      弹性布局：display: flex
        // flex_col():  纵向排列：flex-direction: column
        // gap_2():     演示块之间的间距：gap: 0.5rem (8px)
        // p_4():       内边距：padding: 1rem (16px)
        // size_full(): 撑满窗口：width: 100%; height: 100%
        div()
            .bg(gpui::white())
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .size_full()
            // ---------- 一、对齐：text-align ----------
            .child(div().child("Text left")) // 不写对齐默认就是左对齐
            .child(div().text_center().child("Text center")) // 文字居中：text-align: center
            .child(div().text_right().child("Text right")) // 文字右对齐：text-align: right
            // ---------- 二、下划线 ----------
            // text_decoration_1(): 1 像素下划线：text-decoration-line: underline; text-decoration-thickness: 1px
            // 下面三行只差对齐方式
            .child(div().text_decoration_1().child("Text left (underline)"))
            .child(
                div()
                    .text_center() // 文字居中：text-align: center
                    .text_decoration_1() // 下划线：同上
                    .child("Text center (underline)"),
            )
            .child(
                div()
                    .text_right() // 文字右对齐：text-align: right
                    .text_decoration_1() // 下划线：同上
                    .child("Text right (underline)"),
            )
            // ---------- 三、删除线 ----------
            // line_through(): 删除线：text-decoration: line-through（线宽 1px）
            // 同样三行只差对齐方式
            .child(div().line_through().child("Text left (line_through)"))
            .child(
                div()
                    .text_center() // 文字居中：text-align: center
                    .line_through() // 删除线：同上
                    .child("Text center (line_through)"),
            )
            .child(
                div()
                    .text_right() // 文字右对齐：text-align: right
                    .line_through() // 删除线：同上
                    .child("Text right (line_through)"),
            )
            // ---------- 四、溢出裁剪 ----------
            // 一行放两个定宽盒子（两端对齐拉开），都禁止换行、放不下的部分直接裁掉：
            .child(
                div()
                    .flex() // 弹性布局：display: flex
                    .gap_2() // 盒子之间的间距：gap: 0.5rem (8px)
                    .justify_between() // 两端对齐：justify-content: space-between
                    .child(
                        div()
                            .w(px(400.)) // 定宽 400 像素：width: 400px
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::blue()) // 边框颜色：border-color: #0000ff
                            .p_1() // 内边距：padding: 0.25rem (4px)
                            .whitespace_nowrap() // 禁止换行：white-space: nowrap
                            .overflow_hidden() // 超出盒子的部分裁掉不显示：overflow: hidden
                            .text_center() // 文字居中：text-align: center
                            .child("A long non-wrapping text align center"),
                        // 这行字比 400px 宽，又不让换行：居中对齐时左右两边都被裁掉一截
                    )
                    .child(
                        div()
                            .w_32() // 32 号宽度：width: 8rem (128px)，编号按 Tailwind 刻度 n × 0.25rem
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::blue()) // 边框颜色：border-color: #0000ff
                            .p_1() // 内边距：padding: 0.25rem (4px)
                            .whitespace_nowrap() // 禁止换行：white-space: nowrap
                            .overflow_hidden() // 超出盒子的部分裁掉不显示：overflow: hidden
                            .text_right() // 文字右对齐：text-align: right
                            .child("100%"),
                    ),
            )
            // ---------- 五、富文本混排 StyledText ----------
            // 普通 .child("文本") 只能套用容器统一的文字样式；
            // StyledText 可以在一段文字内部按「字节区间」套不同样式，
            // 类似 HTML 里一段文字中嵌 <span> 的富文本。
            // with_highlights 的每一项是 (字符区间, 样式)，区间是字节偏移且必须落在字符边界上。
            .child(div().flex().gap_2().justify_between().child(
                // 外层这个 flex 容器是照抄上一行的写法，里面只有一个子元素，
                // gap / justify_between 在这里实际不起作用
                StyledText::new("ABCD").with_highlights([
                    (0..1, FontWeight::EXTRA_BOLD.into()), // "A"（第 0 字节）：特粗字重，FontWeight 转成 HighlightStyle
                    (2..3, FontStyle::Italic.into()), // "C"（第 2 字节）：斜体，FontStyle 转成 HighlightStyle
                ]),
            ))
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）
        if !example_support::load_fonts(cx) {
            return;
        }

        // 计算窗口位置：让 800x600 的窗口出现在屏幕中央
        // None 表示不指定显示在哪个显示器上，用默认的
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        // 打开一个新窗口
        cx.open_window(
            WindowOptions {
                // 窗口使用我们上面算出来的位置和尺寸（而不是最大化 / 全屏）
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // 其余选项都用默认值
                ..Default::default()
            },
            // 窗口的内容：创建一个 HelloWorld 实例作为根组件
            |_, cx| cx.new(|_| HelloWorld {}),
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        // 让这个窗口成为活动窗口（抢到输入焦点）
        cx.activate(true);
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main
#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
