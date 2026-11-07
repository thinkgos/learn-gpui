//! 鼠标压力示例：实时显示压感设备（如 macOS 力度触控板）的压力档位和压力值。
//!
//! 运行方式：`cargo run --example mouse_pressure`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：500x500 的灰色方块居中显示两行文字：
//!   1. Pressure stage：压力档位（Zero / Normal / Force）；
//!   2. Pressure amount：压力值（0.00~1.00）。
//!
//! 关键点（见 7.8）：压力是**独立事件** `MousePressureEvent`，
//! 不是鼠标按下 / 移动事件的字段，用 `.on_mouse_pressure(listener)` 订阅。
//! 目前仅 macOS 力度触控板实现，触屏的压力在 `TouchEvent.force` 里。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, MousePressureEvent, PressureStage, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例组件：记录最近一次压感事件的档位和压力值。
struct MousePressureExample {
    /// 压力档位：Zero（无压力）/ Normal（普通按压）/ Force（触发 force click 的重压）。
    pressure_stage: PressureStage,
    /// 当前档位下的压力值：0.0~1.0 的浮点数。
    pressure_amount: f32,
}

/// 实现 `Render`：一个居中的灰色方块，实时显示档位和压力值。
impl Render for MousePressureExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
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
            .child(format!("Pressure stage: {:?}", self.pressure_stage)) // 档位：{:?} 按 Debug 格式打出枚举名
            .child(format!("Pressure amount: {:.2}", self.pressure_amount)) // 压力值：{:.2} 保留两位小数
            .on_mouse_pressure(cx.listener(Self::on_mouse_pressure)) // 订阅压感事件，回调写在下面
    }
}

impl MousePressureExample {
    /// 压感事件回调：记下档位和压力值，通知 GPUI 重绘。
    fn on_mouse_pressure(
        &mut self,
        pressure_event: &MousePressureEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pressure_amount = pressure_event.pressure; // 压力值 0.0~1.0
        self.pressure_stage = pressure_event.stage; // 档位：Zero / Normal / Force

        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
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
        // 计算窗口位置：让 500x500 的窗口出现在屏幕中央。
        // None 表示不指定显示在哪个显示器上，用默认的。
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default()                                 // 其余选项都用默认值
            },
            |_, cx| {
                // 根视图：档位从 Zero（无压力）起步
                cx.new(|_| MousePressureExample {
                    pressure_stage: PressureStage::Zero,
                    pressure_amount: 0.0,
                })
            },
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
