//! GIF 动图示例：`img()` 直接收 `PathBuf` 播 GIF，**没有任何专用 API** ——
//! 帧动画自动播放，速度取每帧自带的延时（见 6.3）。
//!
//! 运行方式：`cargo run --example gif_viewer`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 窗口里只有一张铺满的 GIF（`examples/image/black-cat-typing.gif`），
//!      等比缩放、完整显示（可能有留白）。
//!
//! 关键概念：
//!   1. `img()` 的参数决定图片来源：`PathBuf` 直接读文件系统；
//!      **字符串不等于文件路径**（字符串是 URL 或 AssetSource 资源键，见 6.1）；
//!   2. GIF（以及带动画的 WebP）自动播放：窗口失焦自动暂停，系统开了
//!      「减少动态效果」也不动（见 6.3）；
//!   3. `ObjectFit::Contain`：等比缩放完整显示，`object-fit: contain`（默认值，见 6.7）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{App, Context, Render, Window, WindowOptions, div, img, prelude::*};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;
use std::path::PathBuf;

/// GIF 查看器：只有「GIF 文件路径」一个字段。
struct GifViewer {
    /// GIF 文件路径
    gif_path: PathBuf,
}

impl GifViewer {
    /// 新建查看器，记住要播的 GIF 路径。
    fn new(gif_path: PathBuf) -> Self {
        Self { gif_path }
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for GifViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            // img() 收 PathBuf 直接读文件，GIF 每帧自动播放（见 6.3）
            img(self.gif_path.clone())
                .size_full() // 铺满容器：width: 100%; height: 100%
                .object_fit(gpui::ObjectFit::Contain) // 等比缩放完整显示：object-fit: contain
                .id("gif"), // 元素 id
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
        // 拼出 GIF 的绝对路径（以 Cargo 清单目录为基准，任何工作目录下都能跑）
        let gif_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/image/black-cat-typing.gif");

        // 开窗口：内容就是那个 GIF 查看器
        cx.open_window(
            WindowOptions {
                focus: true, // 打开后自动拿键盘焦点
                ..Default::default()
            },
            |_, cx| cx.new(|_| GifViewer::new(gif_path)),
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
    env_logger::init(); // 初始化日志（配合 RUST_LOG=info 可看加载日志）
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
