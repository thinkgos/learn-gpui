//! 「关掉最后一个窗口就退出应用」示例：开两个并排的窗口，
//! 不管用 cmd-w 还是系统红绿灯关窗，只要窗口都关完了，整个应用就退出。
//!
//! 运行方式：`cargo run --example on_window_close_quit`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 两个 500x500 的深灰窗口左右并排，各占一个屏幕位置；
//!   2. 每个窗口里只有一行提示文字，说明「关窗应该连应用一起退出」。
//!
//! 关键概念（详见第 20 章，动作和焦点见第 8、9 章）：
//!   1. `cx.on_window_closed(...)`：App 级的窗口关闭回调，
//!      在里面用 `cx.windows().is_empty()` 判断是不是最后一个窗口（见 20.4）；
//!   2. `actions!` + `KeyBinding`：把 cmd-w 绑到 CloseWindow 动作上，
//!      动作由 `on_action` 处理，效果是 `window.remove_window()`（见第 9 章）；
//!   3. `FocusHandle` + `track_focus()`：让元素收到键盘动作的前提（见第 8 章）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, Window, WindowBounds, WindowOptions, actions,
    div, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// actions! 宏声明一个「关闭窗口」动作类型（见第 9 章）。
actions!(example, [CloseWindow]);

/// 窗口的根实体：界面只是提示文字，真正干活的是动作处理和焦点跟踪。
struct ExampleWindow {
    /// 焦点句柄：绑定到根元素上，cmd-w 才能派发到这个窗口的 on_action（见第 8 章）。
    focus_handle: FocusHandle,
}

impl Render for ExampleWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            // 处理 CloseWindow 动作：关掉当前窗口。cmd-w、菜单、其他地方都能触发同一个动作
            .on_action(|_: &CloseWindow, window, _| {
                window.remove_window(); // 关掉「自己这个」窗口（见 20.3）
            })
            .track_focus(&self.focus_handle) // 跟踪焦点：让这个元素收得到键盘动作（见第 8 章）
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
            .child(
                "Closing this window with cmd-w or the traffic lights should quit the application!",
            )
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 第一个窗口的初始位置尺寸：屏幕中央 500x500。mut 是因为下面要把它往右挪一格
        let mut bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);

        cx.bind_keys([KeyBinding::new("cmd-w", CloseWindow, None)]); // cmd-w 触发 CloseWindow（见第 9 章）
        // 窗口关闭回调注册在 App 上（不是某个窗口上），每关一个窗口就触发一次（见 20.4）。
        // 这就是本示例的主角：窗口清空 → 退出应用。
        cx.on_window_closed(|cx, _window_id| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach(); // 订阅句柄丢出去，回调一直有效

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default() // 其余选项都用默认值
            },
            |window, cx| {
                cx.activate(false); // 让应用激活（false = 不强行抢别的应用的焦点）
                cx.new(|cx| {
                    let focus_handle = cx.focus_handle(); // 新建焦点句柄（见第 8 章）
                    focus_handle.focus(window, cx); // 打开就聚焦，不用用户先点一下窗口
                    ExampleWindow { focus_handle }
                })
            },
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        bounds.origin.x += bounds.size.width; // 第二个窗口平移到第一个右边，并排摆放

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 平移后的位置尺寸
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let focus_handle = cx.focus_handle();
                    focus_handle.focus(window, cx);
                    ExampleWindow { focus_handle }
                })
            },
        )
        .unwrap();
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
