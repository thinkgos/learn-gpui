# 1. Hello World — 你的第一个GPUI程序

每个GPUI应用都遵循同一套骨架: 创建应用、打开窗口、运行事件循环. 本章带你写完并跑通第一个程序, 认识 `Application`、`App`、`Window`、`Entity`、`Render` 这几个绕不开的概念, 后面所有章节都建立在这套骨架之上.

## 1.1 最小GPUI程序

一个最小的GPUI程序分三步:

1. **创建应用**: `gpui_platform::application()` 返回当前平台的 `Application`.
2. **启动事件循环**: `.run(|cx: &mut App| { ... })` 启动应用, 闭包在应用就绪后执行一次, 里面的`cx`是全局上下文.
3. **打开窗口**: 在闭包里 `cx.open_window(...)` 指定窗口配置和根组件, 窗口内容就是根组件 `render` 出来的元素树.

```rust
use gpui::{
    App, Bounds, Context, SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    rgb, size,
};
use gpui_platform::application;

/// 我们要展示的"应用状态".GPUI组件就是普通 Rust 结构体,
/// 实现 `Render` trait 后, 就能描述自己长什么样.
struct HelloWorld {
    /// 要显示的文本内容. `SharedString` 是GPUI里的字符串类型, clone 很便宜.
    text: SharedString,
}

/// 实现 `Render`: 告诉GPUI这个组件的界面由哪些元素组成.
impl Render for HelloWorld {
    /// 返回值是"任意可以转换成 Element 的东西", 通常就是一棵 div() 布局树.
    /// `_window` 和 `_cx` 这里用不到, 所以用下划线开头.
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div() // 创建一个 div 元素, 类似 HTML 里的 <div>
            .flex() // 弹性布局: display: flex
            .flex_col() // 改成纵向排列: flex-direction: column
            .gap_3() // 子元素之间的间距: gap: 0.75rem (12px)
            .bg(rgb(0x505050)) // 背景色: background-color: #505050
            .size(px(500.0)) // 宽高都是 500 像素: width: 500px; height: 500px
            .justify_center() // 主轴居中(这里是垂直居中): justify-content: center
            .items_center() // 交叉轴居中(这里是水平居中): align-items: center
            .shadow_lg() // 大阴影: box-shadow: 0 10px 15px -3px rgb(0 0 0/0.1), ...
            .border_1() // 1 像素边框: border-width: 1px
            .border_color(rgb(0x0000ff)) // 边框颜色: border-color: #0000ff
            .text_xl() // 字号加大: font-size: 1.25rem; line-height: 1.75rem
            .text_color(rgb(0xffffff)) // 文字颜色: color: #ffffff
            // 子元素: 一行文字. String 直接就能当元素用
            .child(format!("Hello, {}!", self.text))
    }
}

/// 程序入口
fn main() {
    // application().run(...) 启动应用主循环, 并在闭包里做初始化
    application().run(|cx: &mut App| {
        // 计算窗口位置: 让 500x500 的窗口出现在屏幕中央(None 表示用默认显示器)
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        // 打开一个新窗口, 第二个参数的闭包负责创建窗口的根组件
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 用上面算好的位置和尺寸
                ..Default::default() // 其余选项都用默认值(默认自带系统标题栏)
            },
            |_, cx| {
                // cx.new(...) 创建一个实体: 状态的所有权交给 GPUI, 返回强类型句柄 Entity<T>
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap(); // 简单示例里直接 unwrap, 失败就 panic
        // 让这个窗口成为活动窗口(抢到输入焦点)
        cx.activate(true);
    });
}
```

(真实示例 [`hello_world.rs`](../examples/hello_world.rs) 还做了加载字体、兼容 wasm 两件事, 与GPUI骨架无关, 这里省略.)

逐块看下来:

- `struct HelloWorld` 就是组件本体, 字段就是它要显示的状态.
- `impl Render for HelloWorld` 把状态翻译成界面: `render` 返回一棵元素树,GPUI拿去布局和绘制; 状态没变时直接跳过, 不会白跑(见 1.5).
- `application().run(...)` 之后控制权就交给事件循环, 闭包只在启动时执行一次; 窗口一直开着, 直到用户退出.

> 💡 提示: `SharedString` 是GPUI的共享字符串, 内部是 `SmolStr`(本质为 `Arc<str>` 与 `&'static str` 的抽象), clone 只增加引用计数. `render` 每次重建元素树都会复制文本, 所以GPUI的文本 API 普遍收 `SharedString`(见 1.4).

**NOTE**: 应用入口是 `Application` 加 `.run(...)`, **没有** `App::new()` 这个构造函数 —— `Application` 只能通过 `Application::with_platform(...)` 构造, `gpui_platform::application()` 就是"当前平台"版本的封装.

**NOTE**: `open_window` 的第二个参数是 `FnOnce(&mut Window, &mut App) -> Entity<V>` —— 闭包收 `window` 和`cx`**两个** 参数, 返回值必须是 `cx.new(...)` 创建出来的 `Entity<V>`, **不是**直接返回组件本体.

**NOTE**: `WindowOptions` 的标题栏字段类型是 `Option<TitlebarOptions>`, **没有** `WindowTitlebarOptions` 这个类型. `WindowOptions::default()` 已经带了默认标题栏, 只改窗口位置/尺寸时 `..Default::default()` 就够了.

## 1.2 关键概念: 一段话速认

- **Application**: 应用的"启动器". 在 `main` 里创建和配置(资源加载、退出策略等), 调 `.run()` 把控制权交给事件循环, 之后你几乎不再直接碰它.
- **App**: 全局上下文, 也就是 `run` 闭包手里的 `cx`. 事件循环、所有实体、窗口、全局状态都归它管, 严格跑在主线程(见 1.1).
- **Window**: 一个操作系统窗口, 也是一块独立的渲染表面. `render` 的第一个参数 `window` 就是它, 焦点、尺寸这类"窗口的事"都找它(见 1.2).
- **Entity**: 被GPUI托管的状态容器. 你的结构体经 `cx.new(...)` 注册后成为实体, 拿到强类型句柄 `Entity<T>`, 谁持有句柄谁就能读写、观察它(见 1.4).
- **Render**: 把"数据"升级成"视图"的 trait. 实现了 `Render` 的实体就能画上屏: `render` 按当前状态返回元素树(`impl IntoElement`)(见 1.5).
- 顺带两个: **Context** 是传给你的 `cx`, 负责安全读写实体、通知重绘(见 1.3); **Element** 是 `div()` 的产物, 即界面上一个个单元.

## 1.3 div() 链式 API: 像写 Tailwind 一样搭界面

GPUI 的界面描述是**链式调用**风格: 每个修饰方法都返回 `self`, 一路点下去把样式、布局、子元素攒成一棵元素树. 方法名就是 Tailwind 的类名, 完全类型安全; 注释里写上等价 CSS 就能两边对照.

常用修饰方法分几类(详细清单见第 2 ~ 4 章):

| 分类 | 方法示例 | 效果 |
| :--- | :--- | :--- |
| 布局 | `.flex()` `.flex_row()` `.flex_col()` | 弹性布局及排列方向 |
| 对齐 | `.items_center()` `.justify_center()` `.justify_between()` | 子元素对齐: align-items/justify-content |
| 尺寸 | `.size()` `.w()` `.h()` `.flex_1()` | 宽高与弹性伸缩 |
| 间距 | `.p()` `.m()` `.gap()` | 内外边距与子元素间距 |
| 颜色 | `.bg()` `.text_color()` `.border_color()` | 背景、文字、边框颜色 |
| 圆角 | `.rounded_md()` `.rounded_lg()` `.rounded_full()` | border-radius |
| 子元素 | `.child()` `.children()` | 添加单个/多个子元素 |

**NOTE**: 大多数样式方法的调用顺序不影响结果(各自设置独立属性), 但 `.child()`/`.children()` 的调用顺序决定子元素的排列顺序.

**NOTE**: 文字颜色用 `.text_color(...)`, **没有** `text_white()` 这种按颜色命名的方法.

布局系统见第 2 章, 颜色阴影等样式见第 3 章, 文字排版见第 4 章; 给元素绑点击、键盘等交互见第 6 章. 本章先把骨架跑通, 这些只需点到为止.

## 1.4 跑起来

本仓库 [`examples/`](../examples/) 目录下每个 `.rs` 文件都是独立可运行的示例, 依赖已在 `Cargo.toml` 里配好. 在仓库根目录执行:

```bash
cargo run --example hello_world
```

一切正常的话, 你会看到一个 500x500 的窗口: 深灰背景、蓝色 1 像素边框、大阴影, 中央是白色的 "Hello, World!" 文字, 下方还有一排彩色小方块(完整示例顺手多画的第二个子元素).

## 1.5 速查表

| 类别 | 写法 | 说明 |
| :--- | :--- | :--- |
| 创建应用 | `gpui_platform::application()` | 当前平台的 `Application`(另有 `headless()` 等) |
| 启动 | `application().run(\|cx\| ...)` | 启动事件循环, 闭包收 `&mut App`, 启动时执行一次 |
| 打开窗口 | `cx.open_window(WindowOptions {...}, \|window, cx\| ...)` | 配置窗口, 闭包返回根组件 `Entity<V>` |
| 窗口选项 | `WindowOptions { window_bounds, titlebar, ..Default::default() }` | 位置尺寸、标题栏等, 默认值即可开箱 |
| 组件 | `struct Xxx` + `impl Render for Xxx` | 状态 + 渲染逻辑, `render` 返回 `impl IntoElement` |
| 创建实体 | `cx.new(\|cx\| Xxx { ... })` | 状态交给GPUI托管, 返回句柄 `Entity<Xxx>` |
| 搭界面 | `div().flex()...().child(...)` | 链式搭出元素树, 样式方法名同 Tailwind |

完整示例: [`cargo run --example hello_world`](../examples/hello_world.rs)

## 1.6 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`hello_world`](../examples/hello_world.rs) | 窗口显示 "Hello, World!" 和一排彩色方块; 源码在最小骨架之外还做了加载字体、兼容 wasm 两件事 |
