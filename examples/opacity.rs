//! 透明度示例：`.opacity()` 让**整个元素（含所有子元素）**一起变淡，和「颜色自带
//! alpha、只影响自己」形成对照（见 4.6）。点击面板还会用 `request_animation_frame()`
//! 逐帧把透明度从 0 推到 1，做最简单的补间动画。
//!
//! 运行方式：`cargo run --example opacity`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 底层：灰底 + 居中的蓝色文字 "This is background text."（透过面板能看见）；
//!   2. 上层：绝对定位的半透明面板（初始 opacity 0.5，四周缩进 32px），
//!      面板的白底、红边、里面的文字图片阴影，全都跟着面板一起变淡；
//!   3. 面板内含：app 图标、当前 opacity 数值、SVG 箭头、6 个 emoji
//!      （hover 单独变淡）、一张自动播放的 GIF；
//!   4. 点面板任意位置（提示文字 "Opacity Panel (Click to test)"）：
//!      opacity 归 0 后逐帧 +0.005 涨回 1.0。
//!
//! 关键概念：
//!   1. `.opacity(f32)` 影响整个元素树：0.0 全透明 ~ 1.0 完全不透明（见 4.6）；
//!   2. 想只淡一个颜色、不牵连子元素，用颜色自带的 alpha
//!      （`rgba(0x00000080)` 或 `gpui::black().alpha(0.5)`），不是 `.opacity()`；
//!   3. `window.request_animation_frame()`：申请下一帧继续重绘，动画状态直接在
//!      render 里推进（GPUI 没有 CSS animation，见第 17 章）；
//!   4. `.hover(|style| style.opacity(0.5))` 是悬停状态样式（见第 7 章）；
//!   5. `img("image/...")` 里的字符串是 **AssetSource 资源键**而不是文件路径，
//!      本示例把 `examples/` 目录注册成资源根（见 6.4）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use std::{fs, path::PathBuf};

use anyhow::Result;
use gpui::{
    App, AssetSource, Bounds, BoxShadow, ClickEvent, Context, SharedString, Task, Window,
    WindowBounds, WindowOptions, div, hsla, img, prelude::*, px, rgb, size, svg,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、资源加载：把 examples/ 目录当资源根
// ===========================================================================

/// 资源加载器：`img("image/...")` 这类资源键会到这里查（见 6.4）。
struct Assets {
    /// 资源根目录（run_example 里传入 examples/ 的绝对路径）
    base: PathBuf,
}

impl AssetSource for Assets {
    /// 按资源键读文件字节：图片/SVG 都是二进制，不是字符串
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|e| e.into())
    }

    /// 列出某目录下的资源名
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|e| e.into())
    }
}

// ===========================================================================
// 二、示例根组件
// ===========================================================================

/// 示例根组件（名字沿用了 hello_world 的骨架，实际演示的是透明度）。
struct HelloWorld {
    _task: Option<Task<()>>,
    /// 当前面板透明度（点击后从 0 逐帧涨回 1）
    opacity: f32,
    /// 透明度动画是否在播
    animating: bool,
}

impl HelloWorld {
    /// 新建状态：面板初始半透明，动画没在播。
    fn new(_window: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            _task: None,
            opacity: 0.5,
            animating: false,
        }
    }

    /// 点击面板时的回调：把透明度归 0，标记动画开始，通知 GPUI 重绘。
    fn start_animation(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.opacity = 0.0;
        self.animating = true;
        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 动画推进：每帧 +0.005，到 1.0 就停；没到就申请下一帧继续重绘
        if self.animating {
            self.opacity += 0.005;
            if self.opacity >= 1.0 {
                self.animating = false;
                self.opacity = 1.0;
            } else {
                window.request_animation_frame(); // 申请下一帧：render 会被再次调用
            }
        }

        div()
            .flex() // 弹性布局：display: flex
            .flex_row() // 横向排列：flex-direction: row
            .size_full() // 铺满窗口：width: 100%; height: 100%
            .bg(rgb(0xe0e0e0)) // 背景色：background-color: #e0e0e0
            .text_xl() // 字号：font-size: 1.25rem (20px)
            // ---------- 底层：背景文字，透过面板看它变淡的效果 ----------
            .child(
                div()
                    .flex()
                    .size_full()
                    .justify_center() // 主轴（横向）居中：justify-content: center
                    .items_center() // 交叉轴（纵向）居中：align-items: center
                    .border_1() // 边框：border-width: 1px
                    .text_color(gpui::blue()) // 文字颜色：color: #0000ff
                    .child(div().child("This is background text.")),
            )
            // ---------- 上层：半透明面板，盖住大部分背景文字 ----------
            .child(
                div()
                    .id("panel") // 有 id 才是「有状态元素」，才能接下面的 on_click（见第 7 章）
                    .on_click(cx.listener(Self::start_animation)) // 点击 → 重播透明度动画
                    .absolute() // 绝对定位：position: absolute（不占布局空间，浮在底层之上）
                    .top_8() // top: 2rem (32px)
                    .left_8() // left: 2rem (32px)
                    .right_8() // right: 2rem (32px)
                    .bottom_8() // bottom: 2rem (32px)
                    .opacity(self.opacity) // 整体透明度（含所有子元素）：opacity: 0.5
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(gpui::white()) // 白底也跟着面板一起变淡：background-color: #ffffff
                    .border_3() // 边框：border-width: 3px
                    .border_color(gpui::red()) // 边框颜色：border-color: #ff0000
                    .text_color(gpui::yellow()) // 文字颜色：color: #ffff00
                    .child(
                        div()
                            .flex()
                            .flex_col() // 纵向排列：flex-direction: column
                            .gap_2() // 子元素间距：gap: 0.5rem (8px)
                            .justify_center()
                            .items_center()
                            .size(px(300.)) // 300×300 正方形：width: 300px; height: 300px
                            .bg(gpui::blue()) // 背景色：background-color: #0000ff
                            .border_3()
                            .border_color(gpui::black()) // 边框颜色：border-color: #000000
                            .shadow(vec![
                                // 外阴影：box-shadow: 10px 10px 1px 5px rgb(0 0 0/0.5)
                                BoxShadow::new(px(10.0), px(10.0), hsla(0.0, 0.0, 0.0, 0.5)) // x/y 偏移 10px，50% 黑
                                    .blur_radius(px(1.0)) // 模糊半径 1px
                                    .spread_radius(px(5.0)), // 扩散半径 5px
                            ])
                            .child(img("image/app-icon.png").size_8()) // 资源键读图（见 6.1）：8 号尺寸的正方形 (32px)
                            .child("Opacity Panel (Click to test)")
                            .child(
                                div()
                                    .id("deep-level-text")
                                    .flex()
                                    .justify_center()
                                    .items_center()
                                    .p_4() // 内边距：padding: 1rem (16px)
                                    .bg(gpui::black()) // 背景色：background-color: #000000
                                    .text_color(gpui::white()) // 文字颜色：color: #ffffff
                                    .text_decoration_2() // 下划线粗细：text-decoration-thickness: 2px
                                    .text_decoration_wavy() // 波浪下划线：text-decoration-style: wavy
                                    .text_decoration_color(gpui::red()) // 下划线颜色：text-decoration-color: #ff0000
                                    .child(format!("opacity: {:.1}", self.opacity)), // 实时显示当前透明度
                            )
                            .child(
                                svg()
                                    .path("image/arrow_circle.svg") // SVG 资源键（见 6.2）
                                    .text_color(gpui::black()) // SVG 是单色蒙版，着色来自 text_color（不设则不可见）
                                    .text_2xl()
                                    .size_8(), // 8 号尺寸的正方形：width: 2rem; height: 2rem (32px)
                            )
                            .child(
                                div()
                                    .flex()
                                    .children(["🎊", "✈️", "🎉", "🎈", "🎁", "🎂"].map(|emoji| {
                                        div()
                                            .child(emoji.to_string())
                                            // 悬停时单独变淡（这里只淡 emoji 自己，不影响兄弟元素）（见第 7 章）
                                            .hover(|style| style.opacity(0.5))
                                    })),
                            )
                            .child(img("image/black-cat-typing.gif").size_12()), // GIF 自动播放（见 6.3）：12 号尺寸 (48px)
                    ),
            )
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // 启动主循环，在闭包里做初始化
    application()
        // 注册资源加载器：资源根 = examples/ 目录（img 的资源键从这里查，见 6.4）
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples"),
        })
        .run(|cx: &mut App| {
            // 加载字体；失败就直接退出
            if !example_support::load_fonts(cx) {
                return;
            }
            // 开一个 500×500 的窗口，位置在屏幕中央
            let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| HelloWorld::new(window, cx)),
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
