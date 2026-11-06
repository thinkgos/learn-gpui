//! 网格布局示例：经典的「圣杯布局」（Holy Grail）——顶部、底部各一条，
//! 中间是「目录 / 正文 / 广告」三栏。
//!
//! 运行方式：`cargo run --example grid_layout`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 这个示例重点演示两件事：
//!   1. 容器查询 `container_query`：先量出容器的实际尺寸，再用 if 选择布局。
//!      拖动窗口改变宽度：宽于 400px 时是三栏网格，
//!      窄于 400px 时折叠成单列堆叠。
//!   2. Grid 布局 API：`grid_cols` / `grid_rows` / `col_span` / `row_span`，
//!      对应 CSS Grid 的 `grid-template-columns` / `grid-column` 等。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, Hsla, Window, WindowBounds, WindowOptions, container_query, div,
    prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 「圣杯布局」示例组件（经典网页排版问题，见
/// https://en.wikipedia.org/wiki/Holy_grail_(web_design)）：
///
/// 拖动窗口改变大小：布局由 `container_query` 根据量到的容器宽度选择，
/// 窄到放不下三栏时折叠成单列堆叠。
struct HolyGrailExample {}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HolyGrailExample {
    /// `_window` 和 `_cx` 这里用不到，所有布局逻辑都在 `container_query` 的闭包里。
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 容器查询：等布局量出这个元素的实际尺寸后，把尺寸交给闭包，由闭包生成内容。
        // 闭包拿到 `container_size`（容器实际宽高），就能在 Rust 里用 if 选择布局，
        // 效果类似 CSS 的 @container 容器查询，只是断点写在代码里而不是样式表里。
        // 这个元素默认占满父容器（自带 width: 100%; height: 100%）。
        container_query(|container_size, _window, _cx| {
            // 小工具闭包：造一个「彩色方块」当占位区块，
            // 统一长相：背景色 + 白色虚线圆角边框，具体颜色由参数决定。
            let block = |color: Hsla| {
                div() // 创建一个 div 元素，类似 HTML 里的 <div>
                    .size_full() // 撑满所在的格子或空间：width: 100%; height: 100%
                    .bg(color) // 背景色：background-color: <传入的颜色>
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_dashed() // 虚线边框：border-style: dashed
                    .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                    .border_color(gpui::white()) // 边框颜色：border-color: #ffffff
                    .items_center() // 交叉轴居中：align-items: center
            };

            // 五个区块都用 block 生成，只是颜色和文字不同
            let header = block(gpui::white()).child(format!("Header — {}", container_size.width));
            // Header 文字里带上量出来的容器宽度，拖窗口时能直观看到数值变化
            let table_of_contents = block(gpui::red()).child("Table of contents");
            let content = block(gpui::green()).child("Content");
            let ad = block(gpui::blue()).child("AD :(").text_color(gpui::white()); // 文字颜色：color: #ffffff（蓝底白字）
            let footer = block(gpui::black())
                .text_color(gpui::white()) // 文字颜色：color: #ffffff（黑底白字）
                .child("Footer");

            // 两种布局共用的外层容器：
            // gap_1():     子元素之间的间距：gap: 0.25rem (4px)
            // bg(...):     背景色：background-color: #505050
            // shadow_lg(): 大阴影：box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), ...
            // size_full(): 撑满窗口：width: 100%; height: 100%
            let container = div().gap_1().bg(rgb(0x505050)).shadow_lg().size_full();

            if container_size.width < px(400.) {
                // ---------- 窄容器（< 400px）：单列堆叠 ----------
                // 从上到下：Header → 目录 → 正文 → 广告 → Footer。
                // h_12():      定高 3rem：height: 48px —— Header / Footer
                // h_20():      定高 5rem：height: 80px —— 目录 / 广告
                // flex_none(): 不随剩余空间伸缩：flex: none (flex: 0 0 auto)
                // flex_1():    吃掉剩余的所有高度：flex: 1 1 0%
                container
                    .flex() // 弹性布局：display: flex
                    .flex_col() // 纵向排列：flex-direction: column
                    .child(header.h_12().flex_none())
                    .child(table_of_contents.h_20().flex_none())
                    .child(content.flex_1())
                    .child(ad.h_20().flex_none())
                    .child(footer.h_12().flex_none())
            } else {
                // ---------- 宽容器（>= 400px）：5×5 网格的圣杯布局 ----------
                // 结构：Header / Footer 各占一行、跨满整行；
                // 中间是「目录(1 列) + 正文(3 列) + 广告(1 列)」三栏。
                //
                // 注意：Grid 的轨道只能等分（5 行一样高），这是对圣杯布局的近似。
                // 真正「不等高」的圣杯（顶栏 auto + 主区域 1fr + 底栏 auto）
                // 要用 Flex 纵向嵌套，见 docs/02-layout.md 的说明。
                container
                    .grid() // 网格布局：display: grid
                    .grid_cols(5) // 5 列等分：grid-template-columns: repeat(5, minmax(0, 1fr))
                    .grid_rows(5) // 5 行等分：grid-template-rows: repeat(5, minmax(0, 1fr))
                    .child(header.row_span(1).col_span_full())
                    // row_span(1):     占 1 行：grid-row: span 1
                    // col_span_full(): 跨满整行：grid-column: 1 / -1
                    .child(table_of_contents.col_span(1).h_56())
                    // col_span(1): 占 1 列：grid-column: span 1
                    // h_56():      定高 14rem：height: 560px
                    .child(content.col_span(3).row_span(3))
                    // col_span(3): 占 3 列：grid-column: span 3
                    // row_span(3): 占 3 行：grid-row: span 3
                    .child(ad.col_span(1).row_span(3))
                    // col_span(1): 占 1 列：grid-column: span 1
                    // row_span(3): 占 3 行：grid-row: span 3
                    .child(footer.row_span(1).col_span_full())
                    // 同 Header：占 1 行，跨满整行
            }
        })
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
            // 窗口的内容：创建一个 HolyGrailExample 实例作为根组件
            |_, cx| cx.new(|_| HolyGrailExample {}),
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
