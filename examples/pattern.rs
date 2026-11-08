//! 图案背景示例：`.bg(pattern_slash(...))` 画斜线网纹，演示线宽/间隔参数怎么取，
//! 以及「同高元素斜线密度保持一致」的等比取参法（见 19.3）。
//!
//! 运行方式：`cargo run --example pattern`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 600×600 白卡片居中，顶部一行大字 "Pattern Example"，整张卡片带大阴影；
//!   2. 第一组 4 根小条（54×18）：前三根参数一模一样，第四根把间隔放大一倍，
//!      看斜线密度怎么随参数变；
//!   3. 第二组绿底方框里 4 根长条：3 根同高（256×56）用同一组参数只换颜色，
//!      最后一根矮一截（256×26）沿用同组参数作对照，配文字
//!      "Elements the same height should align"；
//!   4. 最后一根纯色条 + 一根渐变条：和上面的图案条并排，说明 `.bg()` 一个入口通吃。
//!
//! 关键概念：
//!   1. `pattern_slash(颜色, 线宽, 间隔)`：线宽/间隔是普通 `f32`，**不套 `px()`**（见 19.3）；
//!   2. 想让不同尺寸的元素斜线密度一致，按元素高度等比取值
//!      （本示例里的 `18.0 / 4.0`、`56.0 / 6.0` 都是「元素高度 ÷ 份数」）；
//!   3. 图案本身只有一种颜色、不带底色；要「底色 + 斜线」的叠加效果，
//!      把纯色背景和图案背景放到父子两层元素上；
//!   4. 图案和纯色、渐变一样都是 `Background` 值：`.bg()` 是唯一的背景入口，
//!      三类背景随时可以互相替换（见 4.5）。
//!
//! **NOTE**: 棋盘格 `checkerboard(颜色, 格子边长)` 是另一种内置图案（见 19.3），
//! 本示例只演示斜线网纹 `pattern_slash`。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, AppContext, Bounds, Context, Window, WindowBounds, WindowOptions, div, linear_color_stop,
    linear_gradient, pattern_slash, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例根组件：没有状态，每次渲染都是同一张固定的对照卡片。
struct PatternExample;

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for PatternExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .gap_3() // 子元素间距：gap: 0.75rem (12px)
            .bg(rgb(0xffffff)) // 白色背景：background-color: #ffffff
            .size(px(600.0)) // 600×600 正方形卡片：width: 600px; height: 600px
            .justify_center() // 主轴（纵向）居中：justify-content: center
            .items_center() // 交叉轴（横向）居中：align-items: center
            .shadow_lg() // 大阴影（见 4.3）
            .text_xl() // 字号：font-size: 1.25rem (20px)
            .text_color(rgb(0x000000)) // 文字颜色：color: #000000
            .child("Pattern Example")
            // ---------- 第一组：4 根 54×18 小条，看线宽/间隔参数的效果 ----------
            .child(
                div()
                    .flex()
                    .flex_col()
                    .border_1() // 边框：border-width: 1px
                    .border_color(gpui::blue()) // 边框颜色：border-color: #0000ff
                    // 前三根一模一样（同尺寸、同参数）：用来对照同参数时斜线密度是否一致
                    .child(div().w(px(54.0)).h(px(18.0)).bg(pattern_slash(
                        gpui::red(), // 斜线颜色：#ff0000
                        18.0 / 4.0,  // 线宽 4.5（元素高度 18 的 1/4）
                        18.0 / 4.0,  // 间隔 4.5
                    )))
                    .child(div().w(px(54.0)).h(px(18.0)).bg(pattern_slash(
                        gpui::red(),
                        18.0 / 4.0,
                        18.0 / 4.0,
                    )))
                    .child(div().w(px(54.0)).h(px(18.0)).bg(pattern_slash(
                        gpui::red(),
                        18.0 / 4.0,
                        18.0 / 4.0,
                    )))
                    // 间隔换成 18/2 = 9：线宽不变、间隔翻倍，斜线明显变稀
                    .child(div().w(px(54.0)).h(px(18.0)).bg(pattern_slash(
                        gpui::red(),
                        18.0 / 4.0,
                        18.0 / 2.0,
                    ))),
            )
            // ---------- 第二组：绿底方框里的 4 根长条 ----------
            // 「底色 + 斜线」的叠加效果：外层放纯色背景，内层放图案背景（见 19.3 提示）
            .child(
                div()
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(gpui::blue())
                    .bg(gpui::green().opacity(0.16)) // 淡绿底色：background-color: rgb(0 128 0 / 0.16)
                    .child("Elements the same height should align")
                    // 3 根同高（256×56）的条用同一组参数、只换颜色：
                    // 线宽 = 间隔 = 56/6 ≈ 9.3，密度跟着元素高度走
                    .child(div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(
                        gpui::red(), // 斜线颜色：#ff0000
                        56.0 / 6.0,
                        56.0 / 6.0,
                    )))
                    .child(div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(
                        gpui::green(), // 斜线颜色：#008000
                        56.0 / 6.0,
                        56.0 / 6.0,
                    )))
                    .child(div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(
                        gpui::blue(), // 斜线颜色：#0000ff
                        56.0 / 6.0,
                        56.0 / 6.0,
                    )))
                    // 最后一根矮一截（高 26px）但沿用同一组参数，作为密度对照
                    .child(div().w(px(256.0)).h(px(26.0)).bg(pattern_slash(
                        gpui::yellow(), // 斜线颜色：#ffff00
                        56.0 / 6.0,
                        56.0 / 6.0,
                    ))),
            )
            // ---------- 纯色条、渐变条：和图案条并排，`.bg()` 是唯一背景入口（见 4.5）----------
            .child(
                div()
                    .border_1()
                    .border_color(gpui::blue())
                    .w(px(240.0)) // 宽度：width: 240px
                    .h(px(40.0)) // 高度：height: 40px
                    .bg(gpui::red()), // 纯色背景：background-color: #ff0000
            )
            .child(
                div()
                    .border_1()
                    .border_color(gpui::blue())
                    .w(px(240.0))
                    .h(px(40.0))
                    .bg(linear_gradient(
                        45., // 角度 45°：左下 → 右上（见 19.1）
                        linear_color_stop(gpui::red(), 0.),  // 起始色标：红色
                        linear_color_stop(gpui::blue(), 1.), // 结束色标：蓝色
                    )), // 渐变背景：background-image: linear-gradient(45deg, ...)
            )
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
        // 开一个 600×600 的窗口，位置在屏幕中央
        let bounds = Bounds::centered(None, size(px(600.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_window, cx| cx.new(|_cx| PatternExample),
        )
        .unwrap();

        // 把应用弄到前台来（忽略其他应用）
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
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
