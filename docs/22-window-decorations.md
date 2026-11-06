# 22. 自定义窗口装饰

原生标题栏千篇一律, 很多应用(VS Code、Spotify 风格)选择自己画标题栏和边框.GPUI用 `WindowDecorations` 决定装饰由谁绘制, 配合拖拽、缩放、阴影等 API, 可以做出完全定制的窗口外观.

## 22.1 WindowDecorations 枚举

`WindowDecorations` 是开窗选项, 放在 `WindowOptions::window_decorations` 里, 只有两个取值:

| 枚举值 | 行为 | 适用场景 |
| :--- | :--- | :--- |
| `WindowDecorations::Server` | 系统(窗口管理器)绘制原生标题栏和边框 | 标准应用窗口(默认) |
| `WindowDecorations::Client` | 应用自己渲染标题栏和边框 | 自定义 UI 风格 |

```rust
use gpui::*;

let options = WindowOptions {
    window_decorations: Some(WindowDecorations::Client), // 装饰由应用自己画
    ..Default::default()
};
```

💡 提示: 平台不一定满足装饰请求(源码原话: "The platform may ignore requests it cannot satisfy"), 这个选项主要在 Linux(X11/Wayland)生效. macOS/Windows 隐藏系统标题栏走的是 `TitlebarOptions::appears_transparent: true`, 四种标题栏组合的对照见 `cargo run --example window_movable`.

**运行时查询用的是另一个枚举.** `window.window_decorations()` 返回 `Decorations`, 告诉你当前实际生效的装饰形态:

```rust
match window.window_decorations() {
    Decorations::Server => {
        // 系统装饰, 标题栏和边框都不用自己画
    }
    Decorations::Client { tiling } => {
        // tiling: Tiling { top, left, right, bottom, .. }
        // 被平铺贴靠的边不用画圆角和阴影
    }
}
```

`Tiling` 有 `top`/`left`/`right`/`bottom` 四个布尔字段和 `is_tiled()` 方法, 表示窗口哪些边正被系统平铺(贴靠屏幕边缘或被平铺器接管)——这些边上圆角和阴影应该关掉.

**NOTE**: 查询装饰状态用 `window.window_decorations()`, 返回的是 `Decorations`(带 `tiling` 信息), **不是** `WindowDecorations`;  后者只出现在 `WindowOptions` 里.

## 22.2 自定义标题栏实现

自定义标题栏就是一个普通 `div()`: 摆上标题文字和按钮, 在鼠标按下时调 `window.start_window_move()` 把拖拽交给系统:

```rust
div()
    .flex()                  // 弹性布局: display: flex
    .items_center()          // 垂直居中: align-items: center
    .h(px(36.0))             // 标题栏高度: height: 36px
    .bg(rgb(0x1a1b2e))       // 背景色: background-color: #1a1b2e
    .px_3()                  // 左右内边距: padding-left: 0.75rem; padding-right: 0.75rem
    .on_mouse_down(MouseButton::Left, |_e, window, _cx| {
        window.start_window_move(); // 按住标题栏拖动窗口(交给系统移动)
    })
    .child("自定义标题栏")
```

右键呼出系统窗口菜单(Wayland/X11), 常常就绑在标题栏上:

```rust
.on_click(|e, window, _| {
    if e.is_right_click() {
        window.show_window_menu(e.position()); // 原生窗口菜单(移动、最小化等)
    }
})
```

标题栏右侧的三个按钮直接调 `Window` 上的窗口控制方法:

| 按钮 | 调用 | 说明 |
| :--- | :--- | :--- |
| 最小化 | `window.minimize_window()` | 最小化窗口 |
| 最大化/还原 | `window.zoom_window()` | 在放大和还原之间切换 |
| 关闭 | `window.remove_window()` | 关闭当前窗口 |

**NOTE**: 这些方法都挂在 `window: &mut Window` 上, **没有** `cx.start_window_move()`/`cx.maximize_window()` 这类写法;  放大对应 `zoom_window()`, 而 `cx.hide()` 是隐藏**整个应用**, 不是关窗口.

**NOTE**: 鼠标/点击事件的监听闭包都是 `(event, window, cx)` 三个参数, 不是两个.

**标记系统控件区域.** 元素级的 `.window_control_area(WindowControlArea::...)` 告诉平台这块区域是窗口控件, 系统接管命中测试(拖拽、双击等行为保持原生):

| 取值 | 语义 |
| :--- | :--- |
| `WindowControlArea::Drag` | 标题栏拖拽区 |
| `WindowControlArea::Close` | 关闭按钮 |
| `WindowControlArea::Max` | 最大化按钮 |
| `WindowControlArea::Min` | 最小化按钮 |

## 22.3 ResizeEdge 枚举

Client 装饰下边框缩放也要自己实现:  判断鼠标落在哪条边/角, 调 `window.start_window_resize(edge)` 交给系统. `ResizeEdge` 共 8 个取值, 与光标样式的对应关系(`window_shadow.rs` 的 `resize_edge()` 就是这么映射的):

| 枚举值 | 位置 | 光标样式 |
| :--- | :--- | :--- |
| `ResizeEdge::Top` | 上边 | `CursorStyle::ResizeUpDown` |
| `ResizeEdge::TopRight` | 右上角 | `CursorStyle::ResizeUpRightDownLeft` |
| `ResizeEdge::Right` | 右边 | `CursorStyle::ResizeLeftRight` |
| `ResizeEdge::BottomRight` | 右下角 | `CursorStyle::ResizeUpLeftDownRight` |
| `ResizeEdge::Bottom` | 下边 | `CursorStyle::ResizeUpDown` |
| `ResizeEdge::BottomLeft` | 左下角 | `CursorStyle::ResizeUpLeftDownRight` |
| `ResizeEdge::Left` | 左边 | `CursorStyle::ResizeLeftRight` |
| `ResizeEdge::TopLeft` | 左上角 | `CursorStyle::ResizeUpRightDownLeft` |

把边缘命中判断和移动/缩放串起来的典型写法(完整版见 `cargo run --example window_shadow`):

```rust
.on_mouse_down(MouseButton::Left, move |e, window, _cx| {
    let size = window.window_bounds().get_bounds().size; // 当前窗口尺寸
    match resize_edge(e.position, shadow_size, size) {
        // 落在边上: 系统接管缩放
        Some(edge) => window.start_window_resize(edge),
        // 落在内部: 系统接管移动
        None => window.start_window_move(),
    };
})
```

`resize_edge()` 就是拿鼠标坐标和四边/四角的命中宽度比大小:  两轴都贴边是角, 只有一轴贴边是边, 都不贴则返回 `None`. 命中宽度通常取自绘阴影/边框的宽度(见 21.4).

**NOTE**: 缩放入口是 `window.start_window_resize(edge)`, **没有** `cx.start_window_resize(...)` 这个方法.

## 22.4 set_client_inset() 设置客户区内边距

`window.set_client_inset(inset)` 声明窗口边缘到内容区之间那段"不可见装饰"的宽度, 平台据此把这段区域当装饰处理(Wayland/X11), 让外圈的阴影和缩放手柄不被内容挡住:

```rust
// 渲染开头声明: 四周留 10px 给自绘阴影/边框
window.set_client_inset(shadow_size);
```

- 参数是**一个** `Pixels`, 四边同宽;  读回用 `window.client_inset()`.
- `window_shadow.rs` 里它就是阴影宽度:  内容区外的 10px 既留给圆角阴影, 又是缩放手柄的命中宽度.
- 配套的 `window.set_client_corner_radii(corners)` 把客户端画的圆角告诉平台, 让系统效果(如 macOS 阴影)贴合窗口形状.

**NOTE**: `set_client_inset()` 收单个 `Pixels`, **没有** `Inset { top, left, right, bottom }` 这种结构体, 也不存在按边分别设置的版本.

## 22.5 窗口阴影

阴影是样式 API, 和 CSS `box-shadow` 对应. 预设档位 `shadow_2xs()` 到 `shadow_2xl()` 、清空用 `shadow_none()`;  自定义则用 `shadow(vec![...])` 装一组 `BoxShadow`:

```rust
// 自定义阴影
div()
    .shadow(vec![
        // x/y 偏移和颜色: box-shadow: 0 0 #00000066
        gpui::BoxShadow::new(px(0.0), px(0.0), gpui::hsla(0.0, 0.0, 0.0, 0.4))
            .blur_radius(px(5.0)), // 模糊半径: blur 5px
    ])

// 或者直接用预设档位(和 shadow() 二选一)
div().shadow_lg() // 大阴影: box-shadow: 0 10px 15px -3px rgb(0 0 0/0.1), ...
```

`BoxShadow::new(offset_x, offset_y, color)` 对应 CSS `box-shadow` 的偏移和颜色, 构造后还有三个可调属性:  `.blur_radius(px)` 模糊半径、`.spread_radius(px)` 扩散半径、`.inset()` 改成内阴影.

Client 装饰下系统往往不画窗口外阴影, `window_shadow.rs` 的做法是自己给窗口容器画一道, 且被平铺贴靠时关掉:

```rust
.when(!tiling.is_tiled(), |div| {
    div.shadow(vec![
        gpui::BoxShadow::new(px(0.), px(0.), Hsla { h: 0., s: 0., l: 0., a: 0.4 })
            .blur_radius(shadow_size/2.), // 模糊半径: blur 5px
    ])
})
```

装饰状态会变(拖到屏幕边缘被平铺、系统外观切换), 两条常用钩子:  `window.refresh()` 标脏触发重绘;  `cx.observe_window_appearance(window, |_, window, _| window.refresh())` 订阅外观变化(返回 `Subscription`, 记得 `.detach()` 或保存住).

## 22.6 窗口装饰速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **装饰选择** | `WindowOptions::window_decorations` `WindowDecorations::Server/Client` | 开窗时决定由谁画装饰 |
| **标题栏选项** | `TitlebarOptions { title, appears_transparent, traffic_light_position }` | 标题、透明标题栏(macOS/Windows) |
| **运行时查询** | `window.window_decorations()` `Decorations::Server/Client { tiling }` | 实际装饰形态与平铺边 |
| **拖动/缩放** | `window.start_window_move()` `window.start_window_resize(edge)` | 交给系统移动/缩放 |
| **缩放方向** | `ResizeEdge::Top/TopRight/Right/BottomRight/Bottom/BottomLeft/Left/TopLeft` | 8 个边/角 |
| **窗口控制** | `window.minimize_window()` `window.zoom_window()` `window.remove_window()` `window.show_window_menu()` | 最小化/放大/关闭/系统菜单 |
| **控件区域** | `.window_control_area(WindowControlArea::Drag/Close/Max/Min)` | 标记原生控件命中区 |
| **客户区** | `window.set_client_inset(px)` `window.client_inset()` `window.set_client_corner_radii()` | 不可见装饰宽度与圆角 |
| **阴影** | `.shadow()` `.shadow_lg()` … `BoxShadow::new().blur_radius()` | 元素阴影与窗口阴影 |

完整示例: `cargo run --example window_shadow`(自绘标题栏 + 边框阴影 + 8 向缩放手柄);  标题栏选项对照见 `cargo run --example window_movable`, 窗口选项总览见 `cargo run --example window`.
