//! 多屏定位示例：遍历所有显示器，在每块屏的四角、四边中点和中心
//! 各开一个小窗口（每屏 9 个），直观展示屏幕坐标系和 `Bounds` 的方位方法。
//!
//! 运行方式：`cargo run --example window_positioning`
//! 同样同时支持桌面和 wasm 两套入口。多接一台显示器效果更明显：每块屏都会摆满。
//!
//! 界面分块：
//!   1. 每个 350x75 的小窗口里显示两行坐标：一行是开窗时传入的 bounds，
//!      一行是 `window.bounds()` 实时查到的窗口矩形，两者应该一致；
//!   2. 窗口颜色按位置区分：上方两个红、下方两个蓝、其余五个黑。
//!
//! 关键概念（详见第 21 章）：
//!   1. `cx.displays()` 枚举所有显示器，`display.bounds()` 是屏幕矩形
//!      （全局坐标系，含菜单栏/任务栏），`display.id()` 开窗时指定目标屏幕（见 21.1）；
//!   2. `Bounds` 的方位方法 `origin`/`top_right()`/`bottom_left()`/`bottom_right()`/
//!      `center()`/`top()`/`bottom()`/`left()`/`right()` 直接返回矩形上的关键点（见 21.3）；
//!   3. 点之间可以加减，定位就是「角点 + 偏移」或「角点 - 偏移」；
//!   4. `WindowOptions { window_bounds, display_id, ... }` 里逐个字段
//!      摆出开窗配置的完整写法（字段含义见 20.1）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, DisplayId, Hsla, Pixels, SharedString, Size, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, div, point, prelude::*,
    px, rgb,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、窗口内容
// ===========================================================================

/// 定位标记窗口的内容：除了标签文字，还把「创建时的 bounds」和
/// 「实时查到的 bounds」都显示出来，验证窗口真的开在算好的位置上。
struct WindowContent {
    /// 标签文字：位置名 + 显示器 id，如 "Top Left DisplayId(0)"。
    text: SharedString,
    /// 开窗时传入的矩形（算出来的目标位置），原样存下来展示。
    bounds: Bounds<Pixels>,
    /// 窗口背景色：按位置区分（上红、下蓝、中间黑）。
    bg: Hsla,
}

impl Render for WindowContent {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        // 每帧实时查询窗口当前矩形（用户拖动后会和创建时的 bounds 不同）
        let window_bounds = window.bounds();

        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 改成纵向排列：flex-direction: column
            .bg(self.bg) // 背景色：background-color:（按位置取红/蓝/黑）
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .text_color(rgb(0xffffff)) // 文字颜色：color: #ffffff
            .child(self.text.clone())
            .child(
                div()
                    .flex() // 弹性布局：display: flex
                    .flex_col() // 改成纵向排列：flex-direction: column
                    .text_sm() // 小字号：font-size: 0.875rem; line-height: 1.25rem
                    .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
                    .size_full() // 撑满剩余空间：width: 100%; height: 100%
                    // 第一行：开窗时传入的 bounds（目标位置）
                    .child(format!(
                        "origin: {}, {} size: {}, {}",
                        self.bounds.origin.x,
                        self.bounds.origin.y,
                        self.bounds.size.width,
                        self.bounds.size.height
                    ))
                    // 第二行：window.bounds() 实时查到的窗口矩形（标签文字里的
                    // "cx.bounds()" 是历史遗留，实际取自 window.bounds()）
                    .child(format!(
                        "cx.bounds() origin: {}, {} size {}, {}",
                        window_bounds.origin.x,
                        window_bounds.origin.y,
                        window_bounds.size.width,
                        window_bounds.size.height
                    )),
            )
    }
}

// ===========================================================================
// 二、开窗配置
// ===========================================================================

/// 造一份把窗口钉在指定屏幕指定位置的 `WindowOptions`。
/// 这里逐个字段写全而不是 `..Default::default()` 一笔带过，方便对照 20.1 的字段表。
fn build_window_options(display_id: DisplayId, bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        // 窗口位置尺寸：屏幕坐标系下的矩形（见 21.2）
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // 目标显示器：不写会开到主屏，算好的坐标就可能落在别的屏上（见 21.1）
        display_id: Some(display_id),
        titlebar: None, // 不要系统标题栏，这些标记窗口不需要
        window_background: WindowBackgroundAppearance::Transparent, // 窗口底色：透明（见 20.1）
        focus: false,   // 创建后不抢焦点：一屏 9 个窗口，抢焦点会互相打架
        show: true,     // 创建后立即显示
        kind: WindowKind::PopUp, // 置顶弹出窗口：始终浮在普通窗口之上（见 20.2）
        is_movable: false,       // 用户拖不动：这些是钉死的定位标记
        app_id: None,            // 应用标识用默认（Linux 桌面环境分组用，见 20.1）
        window_min_size: None,   // 不限制最小尺寸
        window_decorations: None, // 窗口装饰用平台默认
        tabbing_identifier: None, // 不加入 macOS 原生标签页分组
        ..Default::default()
    }
}

// ===========================================================================
// 三、应用入口
// ===========================================================================

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 造 9 个定位窗口，摆到每块屏幕的四角、四边中点和中心
        let size = Size {
            width: px(350.),
            height: px(75.),
        };
        let margin_offset = px(150.); // 窗口离屏幕边缘的间距

        // 遍历所有显示器（见 21.1）：单屏时循环只跑一遍，多屏每块屏都摆一套
        for screen in cx.displays() {
            // ---- 左上角：屏幕左上角向右下偏移一个间距 ----
            let bounds = Bounds {
                origin: screen.bounds().origin + point(margin_offset, margin_offset),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Top Left {:?}", screen.id()).into(),
                    bg: gpui::red(),
                    bounds,
                })
            })
            .unwrap(); // 简单示例里直接 unwrap，失败就 panic

            // ---- 右上角：从右上角向左退「窗口宽 + 间距」，向下进一个间距 ----
            let bounds = Bounds {
                origin: screen.bounds().top_right()
                    - point(size.width + margin_offset, -margin_offset),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Top Right {:?}", screen.id()).into(),
                    bg: gpui::red(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 左下角：从左下角向右进一个间距，向上退「窗口高 + 间距」 ----
            let bounds = Bounds {
                origin: screen.bounds().bottom_left()
                    - point(-margin_offset, size.height + margin_offset),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Bottom Left {:?}", screen.id()).into(),
                    bg: gpui::blue(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 右下角：从右下角向左上退「窗口尺寸 + 间距」 ----
            let bounds = Bounds {
                origin: screen.bounds().bottom_right()
                    - point(size.width + margin_offset, size.height + margin_offset),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Bottom Right {:?}", screen.id()).into(),
                    bg: gpui::blue(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 上边中点：x 取屏幕中心减去窗口半宽（居中），y 从顶边进一个间距 ----
            // size.center() 返回 (宽/2, 高/2)，正好当「半个窗口」用（见 21.3）
            let bounds = Bounds {
                origin: point(
                    screen.bounds().center().x - size.center().x,
                    screen.bounds().top() + margin_offset,
                ),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Top Center {:?}", screen.id()).into(),
                    bg: gpui::black(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 左边中点：x 从左边进一个间距，y 垂直居中 ----
            let bounds = Bounds {
                origin: point(
                    screen.bounds().left() + margin_offset,
                    screen.bounds().center().y - size.center().y,
                ),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Left Center {:?}", screen.id()).into(),
                    bg: gpui::black(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 正中心：屏幕中心减去窗口半宽半高，就是居中窗口的左上角 ----
            let bounds = Bounds {
                origin: point(
                    screen.bounds().center().x - size.center().x,
                    screen.bounds().center().y - size.center().y,
                ),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Center {:?}", screen.id()).into(),
                    bg: gpui::black(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 右边中点：x 从右边退「窗口宽 + 间距」，y 垂直居中 ----
            let bounds = Bounds {
                origin: point(
                    screen.bounds().right() - size.width - margin_offset,
                    screen.bounds().center().y - size.center().y,
                ),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Right Center {:?}", screen.id()).into(),
                    bg: gpui::black(),
                    bounds,
                })
            })
            .unwrap();

            // ---- 下边中点：x 居中，y 从底边退「窗口高 + 间距」 ----
            let bounds = Bounds {
                origin: point(
                    screen.bounds().center().x - size.center().x,
                    screen.bounds().bottom() - size.height - margin_offset,
                ),
                size,
            };

            cx.open_window(build_window_options(screen.id(), bounds), |_, cx| {
                cx.new(|_| WindowContent {
                    text: format!("Bottom Center {:?}", screen.id()).into(),
                    bg: gpui::black(),
                    bounds,
                })
            })
            .unwrap();
        }
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
