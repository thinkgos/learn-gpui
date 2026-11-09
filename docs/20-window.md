# 20. 窗口管理

GPUI 的窗口由 `WindowOptions` 配置、`cx.open_window()` 创建, 之后通过 `Window` 句柄做调整/对话框/关闭, 通过 `Context<T>` 的观察者订阅窗口变化.

## 20.1 WindowOptions 完整字段

`WindowOptions` 是创建窗口的配置结构体, 配合 `..Default::default()` 只填需要的字段:

| 字段 | 类型 | 说明 |
| :--- | :--- | :--- |
| `window_bounds` | `Option<WindowBounds>` | 位置/状态: `None` 继承已有边界; `Some(WindowBounds::Windowed(bounds))` 指定位置尺寸 |
| `titlebar` | `Option<TitlebarOptions>` | `None` = 无标题栏(自绘); `Some(...)` 配置标题、透明标题栏、macOS 红绿灯位置 |
| `kind` | `WindowKind` | 窗口类型(见 20.2) |
| `focus` | `bool` | 创建后是否聚焦(默认 `true`) |
| `show` | `bool` | 创建后是否显示(默认 `true`, 传 `false` 创建隐藏窗口) |
| `is_movable` | `bool` | 用户能否拖动窗口(默认 `true`) |
| `is_resizable` | `bool` | 用户能否调整大小(默认 `true`) |
| `is_minimizable` | `bool` | 用户能否最小化(默认 `true`) |
| `app_owns_titlebar_drag` | `bool` | 自绘标题栏时由应用接管拖动(仅 macOS) |
| `display_id` | `Option<DisplayId>` | 显示在哪个显示器(`None` = 主显示器) |
| `window_background` | `WindowBackgroundAppearance` | 窗口底色材质: `Opaque`/`Transparent`/`Blurred`/`MicaBackdrop`/`MicaAltBackdrop` |
| `window_min_size` | `Option<Size<Pixels>>` | 窗口最小尺寸 |
| `window_decorations` | `Option<WindowDecorations>` | X11/Wayland 客户端或服务端装饰 |
| `icon` | `Option<Arc<image::RgbaImage>>` | 窗口图标(仅 X11) |
| `app_id` | `Option<String>` | 应用标识, 供桌面环境分组 |
| `tabbing_identifier` | `Option<String>` | macOS 原生标签页分组 |
| `use_display_p3` | `bool` | 用 P3 色域渲染(仅 macOS) |
| `*_frame_interval` | `Option<Duration>` | 非活跃/低电量/过热时的帧率上限(三个字段) |

`TitlebarOptions` 的字段: `title`(初始标题)、`appears_transparent`(隐藏系统标题栏以便自绘, macOS/Windows)、`traffic_light_position`(macOS 红绿灯按钮位置).

```rust
// 取自 examples/window.rs: 居中开一个 800x600 的普通窗口
let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
cx.open_window(
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口化 + 初始位置尺寸
        ..Default::default()
    },
    |window, cx| cx.new(|cx| WindowDemo {}),
)
.unwrap();
```

**NOTE**: `WindowOptions` **没有** `center: bool` 字段. 居中要在创建前自己算好位置:  `Bounds::centered(None, size, cx)` 或 `WindowBounds::centered(size, cx)`, 再放进 `window_bounds`.

**NOTE**: `window_bounds` 和 `titlebar` 都是 `Option` —— `window_bounds: None` 表示继承已有边界, `titlebar: None` 表示**不要系统标题栏**(自绘标题栏场景, 见 [`window.rs`](../examples/window.rs) 的 Custom Titlebar). `WindowBounds` 的变体是 `Windowed`/`Maximized`/`Fullscreen`, **没有** `Fixed`;  `Maximized`/`Fullscreen` 里包的 `Bounds` 是恢复用的尺寸.

## 20.2 WindowKind 枚举

`kind` 决定窗口的层级和交互方式:

| 变体 | 说明 | 典型用途 |
| :--- | :--- | :--- |
| `WindowKind::Normal` | 普通应用窗口(默认) | 主窗口 |
| `WindowKind::PopUp` | 置顶的弹出窗口 | 警示、弹层 |
| `WindowKind::AnchoredPopup(options)` | 相对父窗口定位的原生弹出层, 尺寸取 `window_bounds` | 下拉菜单、右键菜单、工具提示 |
| `WindowKind::Floating` | 浮动在父窗口之上的窗口 | 工具面板、调色板 |
| `WindowKind::Dialog` | 挡住父窗口交互直到关闭的模态对话框 | 确认框 |
| `WindowKind::LayerShell(options)` | Wayland LayerShell 覆盖层(仅 Linux/Wayland) | dock、壁纸、通知层 |

💡 提示: 平台差异较大 —— 同一个 `kind` 在 macOS/Windows/Linux 上的原生行为(聚焦策略、动画、装饰)不完全一致, 多平台实测为准. [`examples/window.rs`](../examples/window.rs) 把几种 `kind` 各开一个窗口, 可以直观对比.

## 20.3 窗口操作

**window.prompt() —— 系统对话框**:

```rust
pub fn prompt<T: Clone + Into<PromptButton>>(
    &mut self,
    level: PromptLevel,       // Info/Warning/Critical
    message: &str,            // 主提示文字
    detail: Option<&str>,     // 补充说明, 可 None
    answers: &[T],            // 按钮列表
    cx: &mut App,
) -> oneshot::Receiver<usize>  // 异步返回被点按钮的下标
```

按钮既可以直接传 `&str`(仅 "ok"/"cancel" 会映射成 Ok/Cancel 按钮, 其他文字都算 `Other`), 也可以用 `PromptButton` 显式指定语义 —— 中文标签建议用后者, 保留"确定/取消"的按钮类型:

```rust
// 简单版: &str 里的 "ok"/"cancel" 自动映射成 OK/Cancel 按钮
let answer = window.prompt(PromptLevel::Info, "Are you sure?", None, &["OK", "Cancel"], cx);

// 中文按钮: 显式构造 PromptButton
let answer = window.prompt(
    PromptLevel::Info,
    "Are you sure?",
    None,
    &[PromptButton::ok("确定"), PromptButton::cancel("取消")],
    cx,
);

// 结果是异步的: 在任务里 await 出按钮下标
cx.spawn(async move |_| {
    if answer.await.unwrap() == 0 {
        println!("You have clicked Ok");
    }
})
.detach();
```

**NOTE**: `window.prompt()` 返回的是 `oneshot::Receiver<usize>`, **没有** 回调参数 —— 原文 `window.prompt(level, text, &["确定","取消"], cx, |result, cx| ...)` 的签名编译不过. 真实签名还多一个 `detail: Option<&str>`, 按钮下标用 `answer.await` 取回. `PromptLevel` 是 `Info`/`Warning`/`Critical` 三档, `PromptButton` 另有 `::new(label)` 建普通按钮.

**window.resize() —— 调整内容区尺寸**(单位是逻辑像素 `Pixels`):

```rust
window.resize(size(px(1024.0), px(768.0))); // 设置内容区尺寸: 1024x768
```

**窗口显示与状态控制**:

| 方法 | 说明 |
| :--- | :--- |
| `window.remove_window()` | 关闭(销毁)这个窗口 |
| `window.minimize_window()` | 最小化 |
| `window.zoom_window()` | 缩放(等价于标题栏的最大化按钮) |
| `window.toggle_fullscreen()`/`window.is_fullscreen()` | 切换/查询全屏 |
| `window.activate_window()` | 聚焦并把窗口带到前台 |
| `window.request_attention()` | 请求系统注意力(如 Dock 闪烁) |
| `window.bounds()`/`window.window_bounds()` | 查询窗口矩形/状态(含恢复尺寸) |
| `cx.hide()` | 隐藏**整个应用** |
| `cx.activate(ignoring_other_apps)` | 激活应用(如恢复后重新前置) |

**NOTE**: `Window` 上**没有** `maximize_window()`、`fullscreen()`、`exit_fullscreen()` —— 对应能力是 `zoom_window()` 和 `toggle_fullscreen()`. 也**没有**隐藏单个窗口的 `window.hide()`:  `cx.hide()` 隐藏的是整个应用(App 级);  想"临时不显示"用 `WindowOptions { show: false, ... }` 创建, 想"关掉"用 `window.remove_window()`.

## 20.4 监听窗口变化

窗口变化通过**实体的 `Context<T>`** 注册观察者, 统一返回 `Subscription`(保存住或 `.detach()` 才持续有效), 回调签名都是 `|this: &mut T, window: &mut Window, cx: &mut Context<T>|`:

| 方法 | 触发时机 |
| :--- | :--- |
| `cx.observe_window_bounds(window, cb)` | 窗口移动或缩放 |
| `cx.observe_window_activation(window, cb)` | 窗口激活/失活 |
| `cx.observe_window_visibility(window, cb)` | 可见性变化 |
| `cx.observe_window_appearance(window, cb)` | 外观(亮色/暗色)变化 |
| `cx.observe_window_display(window, cb)` | 移到别的显示器/刷新率变化 |
| `cx.observe_button_layout_changed(window, cb)` | 标题栏按钮布局变化 |

```rust
// 取自 examples/window.rs: 在实体创建时注册一次
cx.new(|cx| {
    cx.observe_window_bounds(window, move |_, window, _| {
        println!("Window bounds changed: {:?}", window.bounds());
    })
    .detach();
    WindowDemo {}
})
```

外观用 `window.appearance()` 查询, 返回 `WindowAppearance`(`Light`/`VibrantLight`/`Dark`/`VibrantDark`, 由操作系统决定), App 级查询是 `cx.window_appearance()`. 需要跟随系统亮暗切换配色, 订阅 `observe_window_appearance` 即可.

窗口关闭的回调在 **App** 上: `cx.on_window_closed(|cx, window_id| ...)`. 典型用法是"最后一个窗口关了就退出":

```rust
cx.on_window_closed(|cx, _window_id| {
    if cx.windows().is_empty() {
        cx.quit();
    }
})
.detach();
```

**NOTE**: 原文的 `cx.observe_window_bounds(|cx| ...)` 签名**不对** —— 真实签名第一个参数是 `window`, 回调收 `(&mut T, &mut Window, &mut Context<T>)`, 注册在实体的 `Context<T>` 上(不是全局 `App`). 另外别在 `render()` 里注册观察者(每帧都会新建订阅), 要在实体创建时(`cx.new(|cx| ...)` 里)注册一次.

## 20.5 窗口管理速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **创建** | `cx.open_window(options, build)` | 返回 `Result`, options 用 `..Default::default()` |
| **配置** | `WindowOptions` `TitlebarOptions` `WindowBounds` | 位置、标题栏、行为开关 |
| **类型** | `WindowKind::Normal/PopUp/AnchoredPopup/Floating/Dialog/LayerShell` | 窗口层级与交互 |
| **对话框** | `window.prompt(level, message, detail, answers, cx)` | 返回 `oneshot::Receiver<usize>` |
| **按钮** | `PromptButton::ok/cancel/new` `PromptLevel::Info/Warning/Critical` | 中文标签用 `PromptButton` |
| **尺寸** | `window.resize(size)` `window.bounds()` `window.window_bounds()` | 设置内容尺寸/查询 |
| **状态** | `window.minimize_window()` `window.zoom_window()` `window.toggle_fullscreen()` `window.activate_window()` | 最小化/缩放/全屏/前置 |
| **关闭/隐藏** | `window.remove_window()` `cx.hide()` `cx.activate(b)` | 关窗/隐藏应用/激活应用 |
| **观察者** | `cx.observe_window_bounds/activation/visibility/appearance/display(...)` | 返回 `Subscription`, 在实体创建时注册 |
| **关闭事件** | `cx.on_window_closed(\|cx, id\| ...)` | App 级, 常配合 `cx.windows().is_empty()` 判断退出 |

完整示例: [`cargo run --example window`](../examples/window.rs)(各种 WindowKind、标题栏、prompt、resize、隐藏应用). 关窗即退出的最小示例见 [`cargo run --example on_window_close_quit`](../examples/on_window_close_quit.rs), 系统通知(带操作按钮)见 [`cargo run --example system_notifications`](../examples/system_notifications.rs).

## 20.6 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`window`](../examples/window.rs) | 各种 `WindowKind`、标题栏、`window.prompt()`、resize、隐藏应用(见 20.1–20.3) |
| [`on_window_close_quit`](../examples/on_window_close_quit.rs) | 关窗即退出的最小示例(见 20.4) |
| [`system_notifications`](../examples/system_notifications.rs) | 系统通知的发送、替换、关闭和操作按钮响应 |
