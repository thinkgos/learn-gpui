//! 图片画廊 + 缓存对照示例：30 张网络图排成网格，上半区用**手动管理**的
//! `RetainAllImageCache`，下半区用**自动管理**的自定义 LRU 缓存，对照两种缓存策略
//! （见 6.5、6.6）。
//!
//! 运行方式：`cargo run --example image_gallery`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! **NOTE**: 本示例目前**编译不过** —— 它用了 zed 内部的 `reqwest_client::ReqwestClient`，
//! 而仓库引的注册表版 `reqwest_client` **没有**这个 API，修复前先跳过它。
//!
//! 界面分块：
//!   1. 上半「Manually managed image cache」：顶栏显示已渲染张数 + "Next Photos"
//!      按钮，下面是 30 张缩略图；
//!   2. 下半「Automatically managed image cache」：同样 30 张图，但缓存由
//!      `image_cache(...)` 元素里的自定义 LRU 自动淘汰。
//!
//! 关键概念：
//!   1. `img("https://...")` 的网络图走 HTTP 异步下载（只有 URL 字符串走网络，见 6.1）；
//!   2. 两种挂缓存的方式：`.image_cache(实体)` 把手动管理的缓存实体挂到子树上，
//!      `image_cache(提供器)` 元素给子树套一个自动管理的缓存；
//!   3. `ImageCacheProvider` 回答「缓存实体放哪」（这里存进元素状态，跟着元素走），
//!      `ImageCache` 回答「怎么淘汰」（这里是最久未用先出），淘汰时用
//!      `cx.drop_image()` 释放 GPU 纹理（见 6.6）；
//!   4. URL 里拼时间戳（`?t=...`）让同一张图绕过资源缓存重新下载，方便压测内存。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, AppContext, Bounds, ClickEvent, Context, ElementId, Entity, ImageCache,
    ImageCacheProvider, KeyBinding, Menu, MenuItem, RetainAllImageCache, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowOptions, actions, div, hash, image_cache, img,
    prelude::*, px, rgb, size,
};
// 桌面的 HTTP 客户端：zed 内部 reqwest_client 的 API，注册表版没有（见文件头 NOTE）
#[cfg(not(target_family = "wasm"))]
use reqwest_client::ReqwestClient;
use std::{collections::HashMap, sync::Arc};

/// 每个画廊一次显示的图片数，也是下面 LRU 缓存的上限。
const IMAGES_IN_GALLERY: usize = 30;

// ===========================================================================
// 一、画廊视图：手动缓存 vs 自动缓存
// ===========================================================================

/// 示例根组件：上下两个画廊对照「手动管理」和「自动管理」两种图片缓存。
struct ImageGallery {
    /// 换图参数：拼进 URL 里，逼图片绕过缓存重新下载
    image_key: String,
    /// 一屏显示几张图
    items_count: usize,
    /// 累计渲染过多少张（内存压测看这个数涨）
    total_count: usize,
    /// 手动管理的图片缓存实体（上半画廊用）
    image_cache: Entity<RetainAllImageCache>,
}

impl ImageGallery {
    /// "Next Photos" 按钮回调：清空手动缓存，换一批 URL 重新下载。
    fn on_next_image(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        // 先把手动缓存里的图全清掉（GPU 纹理跟着释放）
        self.image_cache
            .update(cx, |image_cache, cx| image_cache.clear(window, cx));

        // 用当前毫秒时间戳当新的 URL 参数
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        self.image_key = format!("{}", t);
        self.total_count += self.items_count; // 累计计数
        cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for ImageGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // picsum.photos 的随机图 URL；t= 参数用来绕过缓存（每次点按钮都换一批）
        let image_url: SharedString =
            format!("https://picsum.photos/400/200?t={}", self.image_key).into();

        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .text_color(gpui::white())
            .child("Manually managed image cache:")
            // ---------- 上半：手动管理的缓存（RetainAllImageCache，全都不淘汰） ----------
            .child(
                div()
                    .image_cache(self.image_cache.clone()) // 手动挡：把缓存实体挂到这棵子树上
                    .id("main") // 有 id 才是「有状态元素」，才能用滚动方法（见 4.7）
                    .text_color(gpui::black())
                    .bg(rgb(0xE9E9E9)) // 背景色：background-color: #e9e9e9
                    .overflow_y_scroll() // 纵向可滚动：overflow-y: scroll
                    .p_4() // 内边距：padding: 1rem (16px)
                    .size_full() // 铺满容器：width: 100%; height: 100%
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2() // 子元素间距：gap: 0.5rem (8px)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_row() // 横向排列：flex-direction: row
                            .justify_between() // 两端对齐：justify-content: space-between
                            .child(format!(
                                "Example to show images and test memory usage (Rendered: {} images).",
                                self.total_count
                            ))
                            .child(
                                div()
                                    .id("btn")
                                    .py_1()
                                    .px_4()
                                    .bg(gpui::black()) // 背景色：background-color: #000000
                                    .hover(|this| this.opacity(0.8)) // 悬停变淡（状态样式，见第 7 章）
                                    .text_color(gpui::white())
                                    .text_center() // 文字居中：text-align: center
                                    .w_40() // 宽度：width: 10rem (160px)
                                    .child("Next Photos")
                                    .on_click(cx.listener(Self::on_next_image)),
                            ),
                    )
                    .child(
                        div()
                            .id("image-gallery")
                            .flex()
                            .flex_row()
                            .flex_wrap() // 允许换行：flex-wrap: wrap
                            .gap_x_4() // 横向间距：gap: 1rem (16px)
                            .gap_y_2() // 纵向间距：gap: 0.5rem (8px)
                            .justify_around() // 主轴（横向）均分：justify-content: space-around
                            .children(
                                // 每张图 URL 不同（-0 / -1 / ...），size_20 = 5rem (80px) 缩略图
                                (0..self.items_count)
                                    .map(|ix| img(format!("{}-{}", image_url, ix)).size_20()),
                            ),
                    ),
            )
            .child(
                "Automatically managed image cache:"
            )
            // ---------- 下半：自动管理的缓存（自定义 LRU，超上限就淘汰最久没用的） ----------
            .child(image_cache(simple_lru_cache("lru-cache", IMAGES_IN_GALLERY)).child(
                div()
                    .id("main")
                    .bg(rgb(0xE9E9E9))
                    .text_color(gpui::black())
                    .overflow_y_scroll()
                    .p_4()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .id("image-gallery")
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap_x_4()
                            .gap_y_2()
                            .justify_around()
                            .children(
                                (0..self.items_count)
                                    .map(|ix| img(format!("{}-{}", image_url, ix)).size_20()),
                            ),
                    )
            ))
    }
}

// ===========================================================================
// 二、自定义 LRU 缓存：提供器 + 实现
// ===========================================================================

/// 造一个简单 LRU 缓存提供器的辅助函数。
fn simple_lru_cache(id: impl Into<ElementId>, max_items: usize) -> SimpleLruCacheProvider {
    SimpleLruCacheProvider {
        id: id.into(),
        max_items,
    }
}

/// 图片缓存提供器：回答「缓存实体放哪、放几个」，由 `image_cache(...)` 元素来问。
struct SimpleLruCacheProvider {
    /// 元素状态的键：同一个 id 的子树共用同一份缓存
    id: ElementId,
    /// 缓存上限（超过就按 LRU 淘汰）
    max_items: usize,
}

impl ImageCacheProvider for SimpleLruCacheProvider {
    /// 取出（或新建）缓存实体：把它存在元素状态里，跟着元素生命周期走。
    fn provide(&mut self, window: &mut Window, cx: &mut App) -> gpui::AnyImageCache {
        window
            .with_global_id(self.id.clone(), |global_id, window| {
                // 按元素 id 找到这棵树的元素状态（见第 12 章），第一次进来就建一个实体存进去
                window.with_element_state::<Entity<SimpleLruCache>, _>(
                    global_id,
                    |lru_cache, _window| {
                        let mut lru_cache = lru_cache.unwrap_or_else(|| {
                            cx.new(|cx| SimpleLruCache::new(self.max_items, cx))
                        });
                        // 上限变了就重建（简单处理，不做迁移）
                        if lru_cache.read(cx).max_items != self.max_items {
                            lru_cache = cx.new(|cx| SimpleLruCache::new(self.max_items, cx));
                        }
                        (lru_cache.clone(), lru_cache)
                    },
                )
            })
            .into()
    }
}

/// 一个最简单的 LRU 图片缓存：`usages` 记使用顺序（下标 0 = 最近用过），`cache` 存条目。
struct SimpleLruCache {
    max_items: usize,
    usages: Vec<u64>,
    cache: HashMap<u64, gpui::ImageCacheItem>,
}

impl SimpleLruCache {
    /// 新建缓存，并注册「实体被释放时」的清理回调。
    fn new(max_items: usize, cx: &mut Context<Self>) -> Self {
        cx.on_release(|simple_cache, cx| {
            // 缓存实体被丢弃时，把还占着的 GPU 纹理都还回去（见 6.6）
            for (_, item) in std::mem::take(&mut simple_cache.cache) {
                if let Some(Ok(image)) = item.get() {
                    cx.drop_image(image, None);
                }
            }
        })
        .detach(); // 回调独立存在，不挂在返回值上

        Self {
            max_items,
            usages: Vec::with_capacity(max_items),
            cache: HashMap::with_capacity(max_items),
        }
    }
}

impl ImageCache for SimpleLruCache {
    /// 图片加载入口：命中就挪到最前；没命中且满了就先淘汰最久没用的。
    fn load(
        &mut self,
        resource: &gpui::Resource,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Result<Arc<gpui::RenderImage>, gpui::ImageCacheError>> {
        // 内部不变量自检：usages 和 cache 一样长、都不超上限
        assert_eq!(self.usages.len(), self.cache.len());
        assert!(self.cache.len() <= self.max_items);

        let hash = hash(resource); // 资源的哈希当缓存键

        // 命中：把它挪到 usages 最前（最近用过）
        if let Some(item) = self.cache.get_mut(&hash) {
            let current_ix = self
                .usages
                .iter()
                .position(|item| *item == hash)
                .expect("cache and usages must stay in sync");
            self.usages.remove(current_ix);
            self.usages.insert(0, hash);

            return item.use_image(window);
        }

        // 没命中且满了：淘汰 usages 末尾那张（最久没用），释放它的 GPU 纹理
        if self.usages.len() == self.max_items {
            let oldest = self.usages.pop().unwrap();
            let image = self
                .cache
                .remove(&oldest)
                .expect("cache and usages must be in sync");
            if let Some(Ok(image)) = image.get() {
                cx.drop_image(image, Some(window));
            }
        }
        // 新条目进缓存，插到 usages 最前
        let item = gpui::ImageCacheItem::new(resource, cx);
        let result = item.use_image(window);
        self.cache.insert(hash, item);
        self.usages.insert(0, hash);

        result
    }
}

// ===========================================================================
// 三、启动：HTTP 客户端 + 窗口
// ===========================================================================

// 声明 Quit 动作（见 9.1），菜单和快捷键都分发它
actions!(image, [Quit]);

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // 应用入口：桌面用普通的 application()，wasm 用单线程 Web 入口
    #[cfg(not(target_family = "wasm"))]
    let app = gpui_platform::application();
    #[cfg(target_family = "wasm")]
    let app = gpui_platform::single_threaded_web();
    // HTTP 客户端（img("https://...") 靠它下载）：桌面走 reqwest，wasm 走 fetch
    #[cfg(not(target_family = "wasm"))]
    let http_client =
        ReqwestClient::user_agent("gpui example").expect("failed to create HTTP client");
    #[cfg(target_family = "wasm")]
    let http_client = gpui_web::FetchHttpClient::new(app.foreground_executor().to_send());
    let app = app.with_http_client(Arc::new(http_client)); // 把 HTTP 客户端装进应用

    // 启动主循环，在闭包里做初始化
    app.run(move |cx: &mut App| {
        // 加载字体；失败就直接退出
        if !example_support::load_fonts(cx) {
            return;
        }

        // 把应用弄到前台来（忽略其他应用）
        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit()); // Quit 动作处理器：退出应用
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]); // 快捷键绑定（见第 9 章）
        cx.set_menus([Menu::new("Image Gallery").items([MenuItem::action("Quit", Quit)])]); // 菜单栏挂 Quit（见 9.6）

        // 开窗口：标题栏标题 "Image Gallery"，1100×860 居中
        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("Image Gallery")),
                appears_transparent: false, // 标题栏不透明
                ..Default::default()
            }),

            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1100.), px(860.)),
                cx,
            ))),

            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| {
            cx.new(|ctx| ImageGallery {
                image_key: "".into(),
                items_count: IMAGES_IN_GALLERY,
                total_count: 0,
                image_cache: RetainAllImageCache::new(ctx), // 手动缓存实体：只进不出
            })
        })
        .unwrap();
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main。
#[cfg(not(target_family = "wasm"))]
fn main() {
    env_logger::init(); // 初始化日志（配合 RUST_LOG=info 可看下载日志）
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
