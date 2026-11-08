# 6. 图像与媒体

图像是丰富界面表达的重要元素. GPUI提供了对静态图片、SVG 、GIF 动图等多种媒体格式的支持, 并通过 `AssetSource` `trait` 提供灵活的资源加载机制.

## 6.1 静态图片

使用 `img()` 函数加载图片, 参数决定图片来源, **字符串并不等于文件路径**, 这是初学者最容易搞错的地方:

| 写法 | 来源 | 说明 |
| :--- | :--- | :--- |
| `img("https://...")` | HTTP 下载 | 能解析成 URL 的字符串走网络请求 |
| `img("assets/logo.png")` | `AssetSource` 资源键 | 非 URL 字符串按资源键查找(见 6.4), 不是读磁盘 |
| `img(PathBuf::from("assets/logo.png"))` | 文件系统 | `PathBuf`/`&Path` 直接读文件 |

```rust
use gpui::*;

fn render_images() -> impl IntoElement {
    div()
    .flex()  // 弹性布局：display: flex
    .gap_4() // 间距：gap: 16px
    .p_4()   // 内边距：padding: 16px
    // 1. URL 字符串: HTTP 异步下载
    .child(
        img("https://example.com/image.png")
        .size(px(80.0))      // 宽高 80px：width: 80px; height: 80px
        .rounded_full()      // 圆形裁剪：border-radius: 9999px
    )
    // 2. PathBuf: 直接读文件系统
    .child(
        img(PathBuf::from("assets/logo.png"))
        .size(px(100.0))
        .rounded_lg()        // 圆角：border-radius: 8px
    )
    // 3. 非 URL 字符串: AssetSource 里的资源键(见 6.4)
    .child(
        img("assets/banner.jpg")
        .w(px(200.0))        // 宽：width: 200px
        .h(px(150.0))        // 高：height: 150px
    )
}
```

`img()` 还有两个状态钩子和一个渲染开关:

```rust
img(PathBuf::from("assets/banner.jpg"))
    .grayscale(true) // 灰度渲染：filter: grayscale(100%)
    .with_loading(|| div().child("加载中...").into_any_element()) // 加载超过 200ms 时显示
    .with_fallback(|| div().child("加载失败").into_any_element())  // 加载失败时显示
```

- `.with_loading()` 的占位元素**只在加载超过 200 毫秒后**才出现(常量 `LOADING_DELAY`), 快图不会闪占位符.
- `.with_fallback()` 在加载出错时替换整个元素.
- 两者都收一个返回 `AnyElement` 的闭包, 可以画任意占位/错误界面.

## 6.2 SVG图像

GPUI 通过 `svg()` 元素渲染 SVG. `svg()` 不收参数, 用链式方法指定来源:

| 方法 | 来源 |
| :--- | :--- |
| `.path(key)` | `AssetSource` 资源键(见 6.4) |
| `.external_path(path)` | 文件系统路径, 异步读取 |
| `.data(bytes)` | 原始 SVG 字节 |

```rust
use gpui::*;

fn render_svg() -> impl IntoElement {
    div()
    .flex()  // 弹性布局：display: flex
    .gap_4() // 间距：gap: 16px
    // 1. path(): 资源键, 通过 AssetSource 查找
    .child(
        svg()
        .path("assets/icons/settings.svg")
        .size(px(24.0))              // 宽高 24px
        .text_color(rgb(0x333333))   // 图标着色：color: #333333
    )
    // 2. external_path(): 直接读磁盘上的 SVG 文件
    .child(
        svg()
        .external_path("assets/icons/close.svg")
        .size(px(16.0))
        .text_color(gpui::red())     // 图标着色：color: #ff0000
    )
    // 3. data(): 内联的原始 SVG 字节
    .child(
        svg()
        .data(br#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="10" fill="currentColor"/></svg>"#)
        .size(px(32.0))
        .text_color(rgb(0x7c3aed))   // 图标着色：color: #7c3aed
    )
}
```

**NOTE**: SVG 在GPUI里被渲染成**单色蒙版**再按颜色着色, 颜色来自 `.text_color()`. **不设 `text_color` 的 SVG 什么都不画(不可见)**, 这和网页里 SVG 保留原配色的行为不同.

SVG 还支持只影响渲染的几何变换(不影响布局和命中测试):  `.with_transformation(Transformation::scale(...)/::translate(...)/::rotate(...))`.

## 6.3 GIF动图

GPUI 原生支持 GIF 动图(以及带动画的 WebP), **不需要任何专用 API** —— 帧动画自动播放:

```rust
use gpui::*;

pub struct GifViewer {
    /// GIF 文件路径
    gif_path: PathBuf,
}

impl Render for GifViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        //GPUI自动按每帧的延时播放动画
        img(self.gif_path.clone())
        .size_full()                      // 铺满容器：width: 100%; height: 100%
        .object_fit(ObjectFit::Contain)   // 完整显示：object-fit: contain
    }
}
```

- 播放速度取 GIF 每帧自带的延时.
- 窗口失焦时自动暂停, 系统开启"减少动态效果"时也不动.
- 同一视图里别放太多 GIF, 见本章末尾的性能提示.

完整示例: [`cargo run --example gif_viewer`](../examples/gif_viewer.rs).

## 6.4 AssetSource trait 自定义资源加载策略

`img("资源键")` 和 `svg().path("资源键")` 查找的资源来自 `AssetSource`. 通过实现这个 trait, 可以自定义资源加载策略(文件系统、嵌入内存、网络都行):

```rust
use gpui::{AssetSource, SharedString};
use std::borrow::Cow;
use std::path::PathBuf;

/// 从文件系统加载资源的 AssetSource 实现
pub struct FsAssetSource {
    /// 资源根目录
    root: PathBuf,
}

impl AssetSource for FsAssetSource {
    // 加载资源: 返回文件字节(图片/SVG 都是二进制, 不是字符串)
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let data = std::fs::read(self.root.join(path))?;
        Ok(Some(Cow::Owned(data)))
    }

    // 列出某目录下的资源名
    fn list(&self, path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(self.root.join(path))?
            .filter_map(|entry| entry.ok())
            .map(|entry| SharedString::from(entry.file_name().to_string_lossy().to_string()))
            .collect())
    }
}
```

在应用启动时注册:

```rust
fn main() {
    application()
    .with_assets(FsAssetSource { root: PathBuf::from("assets") }) // 注册资源加载器
    .run(|cx: &mut App| {
        // 之后 img("assets/logo.png") 就会从 assets/ 目录读
    });
}
```

**NOTE**: `load()` 收 `&str` 返回**字节的 `Cow`**, 不是 `SharedString` —— 图片和 SVG 都是二进制. 没注册 `AssetSource` 时资源键一律解析失败(图片报"资源不存在"错误, SVG 什么都不画), 但 `img(PathBuf)` 和 `img("https://...")` 不受影响.

完整示例: [`cargo run --example image_loading`](../examples/image_loading.rs)(资源加载 + 自定义异步 Asset + 加载/失败占位), [`cargo run --example opacity`](../examples/opacity.rs)(以示例目录为资源根).

## 6.5 图片画廊

展示如何使用流式布局创建缩略图画廊:

```rust
use gpui::*;

pub struct ImageGallery {
    images: Vec<SharedString>,
}

impl Render for ImageGallery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
        .size_full()          // 铺满窗口：width: 100%; height: 100%
        .id("gallery")        // 有 id 才能用滚动方法
        .overflow_y_scroll()  // 纵向可滚动：overflow-y: scroll
        .p_4()                // 内边距：padding: 16px
        .flex()               // 弹性布局：display: flex
        .flex_wrap()          // 允许换行：flex-wrap: wrap
        .gap_2()              // 缩略图间距：gap: 8px
        .children(self.images.iter().map(|path| {
            img(path.clone())
            .w(px(150.0))                    // 宽：width: 150px
            .h(px(150.0))                    // 高：height: 150px
            .rounded_lg()                    // 圆角：border-radius: 8px
            .object_fit(ObjectFit::Cover)    // 覆盖并裁剪：object-fit: cover
        }))
    }
}
```

进阶版画廊(30 张网络图 + 两种缓存策略对照)见 [`cargo run --example image_gallery`](../examples/image_gallery.rs).

## 6.6 图片缓存

GPUI 的图片缓存分三层, 一般用默认配置就够:

| 层 | 机制 | 何时需要关心 |
| :--- | :--- | :--- |
| 资源缓存 | `img()` 加载结果自动缓存, 同一来源不重复解码; 手动管理用 `cx.remove_asset::<ImgResourceLoader>(&source)` 强制重新加载 | 想"重新加载"时 |
| 图片缓存 | `RetainAllImageCache::new(cx)` 手动管理, 或用 `image_cache(...)` 元素给整棵子树套一个自定义缓存(如 LRU) | 大量缩略图、要控制内存时 |
| GPU 纹理 | 图片解码后进各窗口的精灵图集, `cx.drop_image(image, Some(window))` 释放 | 手写自定义缓存时 |

- 自动缓存: 通过 `img()` 加载的图片会自动缓存.
- 异步加载: 图片异步解码, 不阻塞 UI 线程; 加载完成后自动重绘.
- 内存管理: 手动缓存(`RetainAllImageCache`/自定义 `ImageCache`)负责决定何时淘汰, 淘汰时记得 `cx.drop_image()` 释放纹理.

## 6.7 ObjectFit, 图片适配模式

`ObjectFit` 控制图片在容器内的适配方式, 通过 `.object_fit()` 设置(默认 `Contain`): 

| 取值 | 说明 |
| :--- | :--- |
| `ObjectFit::Cover` | 等比缩放并裁剪, 填满容器(可能有裁剪) |
| `ObjectFit::Contain` | 等比缩放, 完整显示图片(可能有留白, **默认**) |
| `ObjectFit::Fill` | 拉伸填满容器(可能变形) |
| `ObjectFit::ScaleDown` | 仅当原图比容器大时缩小, 否则保持原尺寸居中 |
| `ObjectFit::None` | 保持原始尺寸 |

⚠️ 注意: 大图加载性能注意事项

加载大尺寸图片时需要注意以下几点:

- 异步解码:GPUI会异步解码图片, 但超大图片(>10MB)仍可能导致内存峰值.
- 缩略图优先: 在列表或网格中显示时, 优先使用缩略图而非原图.
- 延迟加载: 对于不可见区域的图片, 考虑延迟加载以减少启动时的内存开销.
- GIF 优化: GIF 动图会持续占用内存和 CPU 资源, 避免在同一视图中加载过多 GIF.

## 6.8 媒体方法速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **图片来源** | `img("https://...")` `img("资源键")` `img(PathBuf)` | URL/AssetSource 键/文件路径 |
| **图片构建** | `.object_fit()` `.grayscale()` `.with_loading()` `.with_fallback()` | 适配、灰度、加载/失败占位 |
| **SVG 来源** | `svg().path()` `.external_path()` `.data()` | 资源键/文件路径/原始字节 |
| **SVG 着色** | `.text_color(color)` | SVG 单色蒙版的着色(不设则不可见) |
| **GIF/动图** | 直接 `img(...)` | 帧动画自动播放 |
| **资源加载** | `impl AssetSource` + `application().with_assets()` | 自定义资源加载策略 |
| **适配模式** | `ObjectFit::Cover/Contain/Fill/ScaleDown/None` | 图片适配方式 |
| **缓存** | `RetainAllImageCache` `image_cache()` `cx.remove_asset()` `cx.drop_image()` | 缓存与释放 |

## 6.9 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`gif_viewer`](../examples/gif_viewer.rs) | 播放 `examples/image/black-cat-typing.gif`, `img()` 直接收路径(见 6.3) |
| [`image_loading`](../examples/image_loading.rs) | 本地图片 + 自定义异步 Asset + 加载中/失败占位(见 6.4) |
| [`opacity`](../examples/opacity.rs) | 以示例目录为 `AssetSource` 资源根加载图片和 SVG(见 6.4) |
| [`image_gallery`](../examples/image_gallery.rs) | 30 张网络图 + 两种缓存策略对照(见 6.5、6.6) |

**NOTE**: image_gallery 目前编译不过(它用了 zed 内部的 `reqwest_client::ReqwestClient`, 而本仓库引的注册表版没有这个 API), 修复前先跳过它, 其他示例不受影响.
