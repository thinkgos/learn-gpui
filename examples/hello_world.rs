//! 最简单的 GPUI 示例：打开一个窗口，显示 "Hello, World!" 和一排彩色方块。
//!
//! 运行方式：`cargo run --example hello_world`
//! 这份代码同时支持桌面平台和 WebAssembly（wasm）平台，所以下面会有两套入口。
//!
//! GPUI 的样式 API 是 Tailwind CSS 风格的，注释格式为「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 我们要展示的“应用状态”。
///
/// GPUI 里的组件就是普通的 Rust 结构体，只要实现 `Render` trait，
/// 就能描述自己长什么样。
struct HelloWorld {
    /// 要显示的文本内容。`SharedString` 是 GPUI 里的字符串类型，
    /// 内部是引用计数的，复制很便宜。
    text: SharedString,
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HelloWorld {
    /// 返回值是"任意可以转换成 Element 的东西"，通常就是一个 div() 布局树。
    /// `_window` 和 `_cx` 分别是窗口上下文和应用上下文，这里暂时用不到，所以用下划线开头。
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div() // 创建一个 div 元素，类似 HTML 里的 <div>
            .flex() // 弹性布局：display: flex
            .flex_col() // 改成纵向排列：flex-direction: column
            .gap_3() // 子元素之间的间距：gap: 0.75rem (12px)
            .bg(rgb(0x505050)) // 背景色：background-color: #505050
            .size(px(500.0)) // 宽高都是 500 像素：width: 500px; height: 500px
            .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .shadow_lg() // 大阴影：box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)
            .border_1() // 1 像素边框：border-width: 1px
            .border_color(rgb(0x0000ff)) // 边框颜色：border-color: #0000ff
            .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
            .text_color(rgb(0xffffff)) // 文字颜色：color: #ffffff
            // 第一个子元素：一行文字
            .child(format!("Hello, {}!", self.text))
            // 第二个子元素：一排彩色小方块
            .child(
                div() // 这个 div 专门用来横向排列方块
                    .flex() // 弹性布局，不写 flex_col 就是横向：display: flex（默认 row）
                    .gap_2() // 方块之间的间距：gap: 0.5rem (8px)
                    // 下面每个方块的写法都一样：一个正方形 div，
                    // 带背景色、虚线边框、圆角。只是颜色不同。
                    .child(
                        div() // 红色方块
                            .size_8() // 8 号尺寸的正方形：width: 2rem; height: 2rem (32px)
                            .bg(gpui::red()) // 背景色：background-color: #ff0000
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                            .border_color(gpui::white()), // 边框颜色：border-color: #ffffff
                    )
                    .child(
                        div() // 绿色方块
                            .size_8() // 正方形：width: 2rem; height: 2rem
                            .bg(gpui::green()) // 背景色：background-color: #00ff00
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角：border-radius: 0.375rem
                            .border_color(gpui::white()), // 边框颜色：border-color: #ffffff
                    )
                    .child(
                        div() // 蓝色方块
                            .size_8() // 正方形：width: 2rem; height: 2rem
                            .bg(gpui::blue()) // 背景色：background-color: #0000ff
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角：border-radius: 0.375rem
                            .border_color(gpui::white()), // 边框颜色：border-color: #ffffff
                    )
                    .child(
                        div() // 黄色方块
                            .size_8() // 正方形：width: 2rem; height: 2rem
                            .bg(gpui::yellow()) // 背景色：background-color: #ffff00
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角：border-radius: 0.375rem
                            .border_color(gpui::white()), // 边框颜色：border-color: #ffffff
                    )
                    .child(
                        div() // 黑色方块（圆角写了两次，重复了但无害）
                            .size_8() // 正方形：width: 2rem; height: 2rem
                            .bg(gpui::black()) // 背景色：background-color: #000000
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角（重复了一次）：border-radius: 0.375rem
                            .border_color(gpui::white()), // 边框颜色：border-color: #ffffff
                    )
                    .child(
                        div() // 白色方块（边框用黑色，因为白色边框看不见）
                            .size_8() // 正方形：width: 2rem; height: 2rem
                            .bg(gpui::white()) // 背景色：background-color: #ffffff
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_dashed() // 虚线边框：border-style: dashed
                            .rounded_md() // 中等圆角：border-radius: 0.375rem
                            .border_color(gpui::black()), // 边框颜色：border-color: #000000
                    ),
            )
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

        // 计算窗口位置：让 500x500 的窗口出现在屏幕中央
        // None 表示不指定显示在哪个显示器上，用默认的
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        // 打开一个新窗口
        cx.open_window(
            WindowOptions {
                // 窗口使用我们上面算出来的位置和尺寸（而不是最大化 / 全屏）
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // 其余选项都用默认值
                ..Default::default()
            },
            // 窗口的内容：创建一个 HelloWorld 实例作为根组件
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
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
