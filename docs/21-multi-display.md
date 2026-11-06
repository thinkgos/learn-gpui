# 21. 多屏定位

现代桌面应用常常要面对多显示器: 窗口开在哪块屏、摆在什么位置、两块屏 DPI 不一致怎么办.GPUI通过 `cx.displays()` 枚举所有显示器, 配合 `Bounds` 的方位方法, 可以在任意屏幕上精确定位窗口.

## 21.1 多显示器支持

`App::displays()` 返回系统当前所有显示器, 每个是 `Rc<dyn PlatformDisplay>`, 从中可以拿到唯一标识 `DisplayId` 和屏幕矩形:

```rust
use gpui::*;

/// 枚举所有显示器并打印位置和尺寸
fn list_displays(cx: &App) {
    for (i, display) in cx.displays().iter().enumerate() {
        let bounds = display.bounds(); // 显示器的矩形(全局坐标系)
        println!(
            "显示器 {}: {:?} 位置=({}, {}) 尺寸={}x{}",
            i,
            display.id(),
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        );
    }
}
```

`PlatformDisplay` 提供的方法:

| 方法 | 说明 |
| :--- | :--- |
| `id()` | 返回 `DisplayId`, 开窗时用来指定目标显示器 |
| `uuid()` | 跨重启稳定的 UUID(`Result<Uuid>`), 适合持久化"上次所在的屏" |
| `bounds()` | 显示器矩形 `Bounds<Pixels>`(整个屏幕, 含菜单栏/任务栏) |
| `visible_bounds()` | 可见区域, 排除 dock/任务栏, 摆窗口用它更合适 |
| `default_bounds()` | 平台建议的默认窗口位置 |
| `refresh_interval()` | 刷新间隔(可变刷新率屏报最大值) |

`DisplayId` 是不透明标识符, 通过 `display.id()` 获取, 放进 `WindowOptions::display_id` 就能把窗口开在指定屏幕上. 另有两个查找入口: `cx.primary_display()` 返回主显示器, `cx.find_display(id)` 按 id 查找;  `window.display(cx)` 则能查出某个窗口当前在哪块屏上.

**NOTE**: 显示器对象上 **没有** `scale_factor()` 和 `is_primary()` 这两个方法. 缩放因子挂在窗口上(`window.scale_factor()`), 要找主屏就用 `cx.primary_display()`, 再比较返回的 `id()`.

## 21.2 Bounds 结构

屏幕和窗口的矩形都用 `Bounds<T>` 描述: 左上角 `origin: Point<T>` 加上尺寸 `size: Size<T>`. 屏幕坐标是 `Bounds<Pixels>`:

```rust
/// 简化版定义(实际是泛型结构体)
struct Bounds<T> {
    origin: Point<T>, // 左上角坐标
    size: Size<T>,   // 宽高
}

struct Point<T> {
    x: T, // X 坐标
    y: T, // Y 坐标
}

struct Size<T> {
    width: T, // 宽度
    height: T, // 高度
}
```

构造矩形最常用的是自由函数 `point(x, y)` 和 `size(w, h)`, 此外还有 `Bounds::new(origin, size)`、`Bounds::from_corners(top_left, bottom_right)`、`Bounds::centered_at(center, size)`、`Bounds::from_anchor_and_size(anchor, origin, size)`(指定角点 + 尺寸). `Point` 之间可以加减, 定位时直接对角点做偏移:

```rust
// 从屏幕左上角向右下偏移 150px, 得到窗口左上角
let origin = screen.bounds().origin + point(px(150.0), px(150.0));
```

**NOTE**: `display.bounds()` 返回的是 `Bounds<Pixels>`(逻辑像素), **不是** `Bounds<DevicePixels>`. 需要物理像素时用 `bounds.to_device_pixels(scale_factor)` 换算, `scale_factor` 来自 `window.scale_factor()`.

## 21.3 屏幕方位方法

`Bounds` 提供一组方位方法, 直接返回矩形上的关键点(都是 `Point<T>`), 窗口定位、弹层对齐时不用手算:

| 方法 | 返回值 | 说明 |
| :--- | :--- | :--- |
| `bounds.origin` | `Point` | 左上角坐标 |
| `bounds.top_right()` | `Point` | 右上角坐标 |
| `bounds.bottom_left()` | `Point` | 左下角坐标 |
| `bounds.bottom_right()` | `Point` | 右下角坐标 |
| `bounds.top_center()` | `Point` | 上边中点 |
| `bounds.bottom_center()` | `Point` | 下边中点 |
| `bounds.left_center()` | `Point` | 左边中点 |
| `bounds.right_center()` | `Point` | 右边中点 |
| `bounds.center()` | `Point` | 中心点 |
| `bounds.corner(Anchor::BottomLeft)` | `Point` | 按 8 个 `Anchor` 取角点/边中点 |

另有 4 个只返回边线坐标的方法 `top()`/`bottom()`/`left()`/`right()`(返回 `T` 而不是点), 以及 `contains(&point)` 判断点是否落在矩形内、`intersects(&other)` 判断两矩形是否相交.

**NOTE**: `Bounds` **没有** `top_left()` 方法, 左上角就是字段 `bounds.origin`;  想和其他角写法保持一致可以用 `bounds.corner(Anchor::TopLeft)`.

```rust
// 取屏幕右下角, 再向左上退 100px 摆一个点
let anchor_point = screen.bounds().bottom_right() - point(px(100.0), px(100.0));
// 判断窗口中心是否落在这块屏上
let on_this_screen = screen.bounds().contains(&window_bounds.center());
```

## 21.4 多屏定位示例

**在指定屏幕上开窗.** 把算好的屏幕坐标和 `DisplayId` 一起塞进 `WindowOptions`. [`window_positioning.rs`](../examples/window_positioning.rs) 遍历所有屏幕, 在每块屏的右上角开一窗:

```rust
use gpui::*;

fn open_on_each_display(cx: &mut App) {
    for screen in cx.displays() {
        let size = Size { width: px(350.0), height: px(75.0) };
        let margin = px(150.0);

        // 窗口摆在屏幕右上角: 横向退一个窗口宽加边距, 纵向进一个边距
        let bounds = Bounds {
            origin: screen.bounds().top_right() - point(size.width + margin, -margin),
            size,
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口位置和大小
                display_id: Some(screen.id()), // 开在哪块屏上
                ..Default::default()
            },
            |_, cx| cx.new(|_| MyApp),
        )
        .unwrap();
    }
}
```

`window_bounds` 收 `WindowBounds` 枚举, 三种取值里的 `Bounds` 都是"还原尺寸":

| 取值 | 说明 |
| :--- | :--- |
| `WindowBounds::Windowed(bounds)` | 普通窗口, 按给定 bounds 摆放 |
| `WindowBounds::Maximized(bounds)` | 打开即最大化(bounds 是还原后的尺寸) |
| `WindowBounds::Fullscreen(bounds)` | 打开即全屏(bounds 是还原后的尺寸) |

`WindowBounds::get_bounds()` 取出内部的 `Bounds<Pixels>`;  `WindowBounds::centered(size, cx)` 一步得到"主屏居中"的取值.

**窗口居中.** 通用算法是"容器原点 + (容器尺寸 - 自身尺寸)/2":

```rust
/// 计算 content 在 container 内居中时的左上角
fn center_origin(content: Size<Pixels>, container: Bounds<Pixels>) -> Point<Pixels> {
    point(
        container.origin.x + (container.size.width - content.width)/2.0,
        container.origin.y + (container.size.height - content.height)/2.0,
    )
}
```

不过GPUI已经内置了居中构造, 通常不必手写:

```rust
// 在主屏可见区域内居中(第一个参数可指定 DisplayId, None 表示主屏)
let bounds = Bounds::centered(None, size(px(600.0), px(600.0)), cx);
// 同类的还有: 生成最大化尺寸的 Bounds::maximized(display_id, cx)
```

[`window_shadow.rs`](../examples/window_shadow.rs) 就是用 `Bounds::centered(None, size(px(600.0), px(600.0)), cx)` 开出居中窗口的.

**跨屏移动窗口.** **NOTE**:GPUI**没有** `move_window()`/`window.move_to()` 这类运行时挪窗口的 API, 窗口位置只在创建时由 `window_bounds` 决定, 之后用户拖动由系统处理. 想把窗口"挪"到另一块屏, 只能用目标屏的 `bounds` 重新开一扇窗再关掉旧的.

💡 提示: [`window_positioning.rs`](../examples/window_positioning.rs) 遍历 `cx.displays()`, 在每块屏的四角、四边中点和中心各开一窗(共 9 个), 是检验方位方法坐标系最直观的例子.

**多屏 DPI.** 不同屏幕的缩放因子可能不同(如 1x 外接屏 + 2x Retina 内置屏). `display.bounds()` 用逻辑像素 `Pixels`;  窗口拖到另一块屏时GPUI自动按 `window.scale_factor()` 重算缩放, 无需手动处理. 要做精确物理像素对齐(如贴边吸附)时, 用 `bounds.to_device_pixels(scale_factor)` 换算后再比较. 需要在窗口换屏时做点什么(比如记住"这块屏上的最大化尺寸"), 订阅 `cx.observe_window_display(window, ...)`(窗口移动到另一块屏或所在屏刷新率变化时触发).

## 21.5 多屏定位速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **枚举显示器** | `cx.displays()` | 返回 `Vec<Rc<dyn PlatformDisplay>>` |
| **查找显示器** | `cx.primary_display()` `cx.find_display(id)` `window.display(cx)` | 主屏/按 id 查/窗口所在屏 |
| **显示器信息** | `display.id()` `display.uuid()` `display.bounds()` `display.visible_bounds()` | 标识与矩形 |
| **Bounds 构造** | `Bounds::new()` `Bounds::from_corners()` `Bounds::centered_at()` `Bounds::centered()` `Bounds::maximized()` | 各种矩形构造与居中 |
| **方位点** | `origin` `top_right()` `bottom_left()` `bottom_right()` `top_center()` `bottom_center()` `left_center()` `right_center()` `center()` `corner(Anchor)` | 矩形关键点 |
| **边线与命中** | `top()` `bottom()` `left()` `right()` `contains()` `intersects()` | 边坐标、点包含、相交 |
| **开窗定位** | `WindowOptions { window_bounds, display_id }` `WindowBounds::Windowed/Maximized/Fullscreen` | 窗口开在哪、多大、什么状态 |

完整示例: [`cargo run --example window_positioning`](../examples/window_positioning.rs).

## 21.6 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`window_positioning`](../examples/window_positioning.rs) | 遍历所有屏幕, 在每块屏的四角、四边中点和中心各开一窗(共 9 个)(见 21.3、21.4) |
| [`window_shadow`](../examples/window_shadow.rs) | 用 `Bounds::centered` 开居中窗口(见 21.4; 窗口阴影见第 22 章) |
