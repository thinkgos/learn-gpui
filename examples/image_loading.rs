//! 图片加载示例：自定义异步 `Asset` 模拟「快加载 / 慢加载 / 加载失败」几种情况，
//! 对照 `.with_loading()`（加载中占位）和 `.with_fallback()`（失败占位）的行为
//! （见 6.1、6.4）。
//!
//! 运行方式：`cargo run --example image_loading`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：一行 4 个 48px 的图片位（红边框标出），点任意一个都会清缓存重来：
//!   1. 快加载（比 200ms 阈值还早 25ms 完成）→ 不闪加载占位，直接出图；
//!   2. 慢加载（等 5 秒）→ 先显示黑色呼吸块，出图后消失；
//!   3. 5 秒后故意失败 → 显示 "?" 灰框失败占位；
//!   4. 指向不存在的路径（走普通 `img(PathBuf)` 加载器）→ 失败占位。
//!
//! 关键概念：
//!   1. `.with_loading()` 的占位元素**只在加载超过 200 毫秒**（常量 `LOADING_DELAY`）
//!      后才出现，快图不会闪占位符；`.with_fallback()` 在加载出错时替换整个元素，
//!      两者都收返回 `AnyElement` 的闭包（见 6.1）；
//!   2. 自定义 `Asset`：`Source` 是缓存键、`Output` 是加载结果，`load()` 返回异步
//!      future —— 可以在内置加载器外面再包一层自己的逻辑（这里包了延时和失败）；
//!   3. `window.use_asset::<加载器>(&键, cx)` 在渲染里发起/复用加载；
//!      `cx.remove_asset::<加载器>(&键)` 清掉资源缓存强制重新加载（见 6.6）；
//!   4. 加载中占位是一段 `with_animation` 呼吸动画（见第 17 章）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use std::{path::Path, sync::Arc, time::Duration};

use gpui::{
    Animation, AnimationExt, App, Asset, AssetLogger, AssetSource, Bounds, Context, Hsla,
    ImageAssetLoader, ImageCacheError, ImgResourceLoader, LOADING_DELAY, Length, RenderImage,
    Resource, SharedString, Window, WindowBounds, WindowOptions, black, div, img, prelude::*,
    pulsating_between, px, red, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、资源加载：资源键就是文件路径
// ===========================================================================

/// 最简单的 `AssetSource`：不设资源根，把资源键本身当文件路径直接读（见 6.4）。
struct Assets {}

impl AssetSource for Assets {
    /// 按路径读文件字节
    fn load(&self, path: &str) -> anyhow::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(path)
            .map(Into::into)
            .map_err(Into::into)
            .map(Some)
    }

    /// 列出目录下的资源路径
    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(
                    entry.ok()?.path().to_string_lossy().into_owned(),
                ))
            })
            .collect::<Vec<_>>())
    }
}

// ===========================================================================
// 二、自定义异步 Asset：模拟延时和失败
// ===========================================================================

/// 要加载的图片绝对路径（编译期用 `CARGO_MANIFEST_DIR` 拼出来）。
const IMAGE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/image/app-icon.png");

/// 自定义 `Asset` 的参数（也就是它的缓存键）：等多久、要不要故意失败。
#[derive(Copy, Clone, Hash)] // 参数要能当缓存键，所以派生 Hash
struct LoadImageParameters {
    /// 加载前要等多久
    timeout: Duration,
    /// 是否故意报错（演示失败占位）
    fail: bool,
}

/// 自定义异步图片加载器：等一个定时器，再决定返回图片还是报错。
struct LoadImageWithParameters {}

impl Asset for LoadImageWithParameters {
    type Source = LoadImageParameters; // 缓存键：参数不同就是不同的缓存条目

    type Output = Result<Arc<RenderImage>, ImageCacheError>; // 加载结果：图片或错误

    /// 启动加载：后台等 timeout，到点后按 fail 决定成功还是失败。
    fn load(
        parameters: Self::Source,
        cx: &mut App,
    ) -> impl std::future::Future<Output = Self::Output> + Send + 'static {
        let timer = cx.background_executor().timer(parameters.timeout); // 后台定时器
        let data = AssetLogger::<ImageAssetLoader>::load(
            // 真正干活的还是内置图片加载器，AssetLogger 只是包一层日志
            Resource::Path(Path::new(IMAGE).to_path_buf().into()),
            cx,
        );
        async move {
            timer.await; // 先把「延时」走完
            if parameters.fail {
                log::error!("Intentionally failed to load image"); // 故意失败
                Err(anyhow::anyhow!("Failed to load image").into())
            } else {
                data.await // 正常返回图片
            }
        }
    }
}

// ===========================================================================
// 三、示例根组件：一行四个图片位
// ===========================================================================

/// 示例根组件：没有自己的状态，看点全在四个 `img()` 的加载/失败占位上。
struct ImageLoadingExample {}

impl ImageLoadingExample {
    /// 加载中占位：黑色背景在 4%~24% 透明度之间来回呼吸（动画见第 17 章）。
    fn loading_element() -> impl IntoElement {
        div().size_full().flex_none().p_0p5().rounded_xs().child(
            div().size_full().with_animation(
                "loading-bg", // 动画 id（同一个 id 共用动画进度）
                Animation::new(Duration::from_secs(3)) // 一轮 3 秒
                    .repeat() // 无限循环
                    .with_easing(pulsating_between(0.04, 0.24)), // 在 0.04~0.24 之间来回
                move |this, delta| this.bg(black().opacity(delta)), // 每帧拿 delta 当黑色的透明度
            ),
        )
    }

    /// 加载失败占位：灰框里一个问号。
    fn fallback_element() -> impl IntoElement {
        let fallback_color: Hsla = black().opacity(0.5); // 50% 黑

        div().size_full().flex_none().p_0p5().child(
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_xs()
                .text_sm() // 字号：font-size: 0.875rem (14px)
                .text_color(fallback_color)
                .border_1() // 边框：border-width: 1px
                .border_color(fallback_color)
                .child("?"),
        )
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for ImageLoadingExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().size_full().justify_around().child(
            div().flex().flex_row().w_full().justify_around().child(
                div()
                    .flex()
                    .bg(gpui::white()) // 背景色：background-color: #ffffff
                    .size(Length::Definite(px(300.0).into())) // 300×300：显式给定尺寸
                    .justify_center()
                    .items_center()
                    .child({
                        let image_source = LoadImageParameters {
                            // 比 LOADING_DELAY（200ms）还早 25ms 加载完：
                            // 不会触发加载占位，直接出图
                            timeout: LOADING_DELAY.saturating_sub(Duration::from_millis(25)),
                            fail: false,
                        };

                        // img() 收闭包 = 自定义资源来源：渲染时用 use_asset 发起/复用加载
                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-1")
                        .border_1() // 边框：border-width: 1px
                        .size_12() // 12 号尺寸的正方形：width: 3rem; height: 3rem (48px)
                        .with_fallback(|| Self::fallback_element().into_any_element()) // 加载失败时换成问号框
                        .border_color(red()) // 边框颜色：border-color: #ff0000
                        .with_loading(|| Self::loading_element().into_any_element()) // 加载超 200ms 时显示呼吸块
                        .on_click(move |_, _, cx| {
                            // 点一下清掉缓存，图片重新走一遍加载流程（可反复观察占位）
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .child({
                        // 慢加载：5 秒后才出图，中间一直显示呼吸占位
                        let image_source = LoadImageParameters {
                            timeout: Duration::from_secs(5),
                            fail: false,
                        };

                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-2")
                        .with_fallback(|| Self::fallback_element().into_any_element())
                        .with_loading(|| Self::loading_element().into_any_element())
                        .size_12()
                        .border_1()
                        .border_color(red())
                        .on_click(move |_, _, cx| {
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .child({
                        // 5 秒后故意失败：显示 "?" 失败占位
                        let image_source = LoadImageParameters {
                            timeout: Duration::from_secs(5),
                            fail: true,
                        };

                        img(move |window: &mut Window, cx: &mut App| {
                            window.use_asset::<LoadImageWithParameters>(&image_source, cx)
                        })
                        .id("image-3")
                        .with_fallback(|| Self::fallback_element().into_any_element())
                        .with_loading(|| Self::loading_element().into_any_element())
                        .size_12()
                        .border_1()
                        .border_color(red())
                        .on_click(move |_, _, cx| {
                            cx.remove_asset::<LoadImageWithParameters>(&image_source);
                        })
                    })
                    .child({
                        // 普通 img(PathBuf) 加载器指向一个不存在的路径：立刻走失败占位；
                        // 顺带确认普通加载器加载坏路径也不会刷屏日志
                        let image_source = Path::new(
                            "this/file/really/shouldn't/exist/or/won't/be/an/image/I/hope",
                        )
                        .to_path_buf();
                        img(image_source.clone())
                            .id("image-4")
                            .border_1()
                            .size_12()
                            .with_fallback(|| Self::fallback_element().into_any_element())
                            .border_color(red())
                            .with_loading(|| Self::loading_element().into_any_element())
                            .on_click(move |_, _, cx| {
                                // 这张图走的是默认加载器 ImgResourceLoader，清缓存也是清它
                                cx.remove_asset::<ImgResourceLoader>(&image_source.clone().into());
                            })
                    }),
            ),
        )
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // 启动主循环，在闭包里做初始化
    application().with_assets(Assets {}).run(|cx: &mut App| {
        // 加载字体；失败就直接退出
        if !example_support::load_fonts(cx) {
            return;
        }
        // 开一个 300×300 的窗口，位置在屏幕中央
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(300.), px(300.)),
                cx,
            ))),
            ..Default::default()
        };
        cx.open_window(options, |_, cx| {
            // 激活应用，但不强行压过别的应用（ignoring_other_apps = false）
            cx.activate(false);
            cx.new(|_| ImageLoadingExample {})
        })
        .unwrap();
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
