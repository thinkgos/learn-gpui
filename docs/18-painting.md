# 18. 底层绘图

布局系统(div、图片、文本)覆盖不了的图形 —— 曲线、饼图、自由涂鸦 —— 用 `canvas()` 进行底层绘图: 在元素的绘制阶段直接向场景提交图形, 由 `PathBuilder` 构建矢量路径, 再用 `window.paint_path`/`window.paint_quad` 画上去. 

## 18.1 canvas() 函数

`canvas()` 创建一个"自己动手画"的元素, 它收**两个闭包**: `prepaint` 和 `paint`: 

```rust
use gpui::*;

// 签名: canvas(prepaint, paint)
//   prepaint: FnOnce(Bounds<Pixels>, &mut Window, &mut App) -> T
//   paint:   FnOnce(Bounds<Pixels>, T, &mut Window, &mut App)
canvas(
    // prepaint: 布局完成后先执行, 可以做测量、预构建路径, 返回值 T 传给 paint
    move |bounds, window, cx| {
        // ...
    },
    // paint: 真正绘制, 第二个参数就是 prepaint 的返回值 T
    move |bounds, t, window, cx| {
        // ...
    },
)
```

- `bounds` 是元素在布局中的实际矩形(窗口坐标系), 所有绘图坐标都是绝对坐标. 
- 两个闭包都是 `FnOnce`, 每帧渲染时随元素一起传入. 
- 不需要 prepaint 做事就返回 `()`, 示例里常写 `move |_, _, _| {}`. 
- `canvas()` 是**自由函数**(不是 `div()` 的方法), 一般包在 `.child(...)` 里;  返回的 `Canvas` 实现了 `Styled`, 可以继续链式设置尺寸: `.size_full()`、`.w(px(200.0))` 等. 

一个最小的圆(取自 [`examples/painting.rs`](../examples/painting.rs) 的画法): 

```rust
canvas(
    move |_, _, _| {}, // prepaint 无事可做
    move |bounds, _, window, _| {
        let center = bounds.center();
        let radius = bounds.size.width * 0.25; // Pixels 只能乘 f32, 没有除 f32

        // 用两段椭圆弧拼一个整圆
        let mut builder = PathBuilder::fill();
        builder.move_to(point(center.x + radius, center.y));
        builder.arc_to(point(radius, radius), px(0.), false, false, point(center.x - radius, center.y));
        builder.arc_to(point(radius, radius), px(0.), false, false, point(center.x + radius, center.y));
        builder.close();
        let path = builder.build().unwrap();
        window.paint_path(path, rgb(0x3b82f6)); // 填充蓝色圆
    },
)
.size(px(120.0)) // 画布自身 120x120: width: 120px; height: 120px
```

**NOTE**:GPUI**没有** `PaintCtx` 这个类型.  "paint 回调获取 PaintCtx" 的说法不成立 —— 绘制阶段拿到的是 `&mut Window`, 绘图方法是 `window.paint_path()`/`window.paint_quad()`. 

💡 提示: canvas 的闭包**每帧执行**, 内容完全由闭包捕获的状态决定. 状态变了(比如鼠标画了新线)要 `cx.notify()` 通知GPUI重绘, 否则画面停在旧内容上;  窗口尺寸变化导致 `bounds` 改变时同理. 

## 18.2 PathBuilder 路径构建

`PathBuilder` 提供类似 SVG path 的 API 构建矢量路径, 构建完 `build()` 得到 `Path<Pixels>` 交给 `paint_path`. 

**基本操作**:

| 方法 | 说明 |
| :--- | :--- |
| `move_to(point)` | 移动画笔到起点(不绘制) |
| `line_to(point)` | 从当前点画直线到目标点 |
| `close()` | 闭合当前子路径(连回起点) |
| `add_polygon(&points, closed)` | 一次性加多边形 |
| `translate(p)`/`scale(f)`/`rotate(deg)` | 对整条路径做几何变换 |
| `dash_array(&[px(4.), px(2.)])` | 虚线样式(描边路径用) |
| `build()` | 完成构建, 返回 `Result<Path<Pixels>>` |

**填充与描边模式**:  在**创建 builder 时**就决定, 不是绘制时传 `Fill`/`Stroke`: 

| 写法 | 模式 |
| :--- | :--- |
| `PathBuilder::fill()` | 填充模式, 填充封闭区域 |
| `PathBuilder::stroke(width)` | 描边模式, `width` 是线宽(如 `px(2.0)`) |

**NOTE**:GPUI**没有** `PathBuilder::new()`, 也**没有** `Fill::new()`/`Stroke::new()` 这类"绘制模式"对象 —— 填充还是描边由 `PathBuilder::fill()`/`PathBuilder::stroke()` 决定, `build()` 的结果直接交给 `window.paint_path`. 

```rust
// 1. 填充三角形
let mut builder = PathBuilder::fill(); // 填充模式
builder.move_to(point(px(60.), px(20.)));
builder.line_to(point(px(20.), px(80.)));
builder.line_to(point(px(100.), px(80.)));
builder.close(); // 闭合路径
let path = builder.build().unwrap();
window.paint_path(path, rgb(0xef4444)); // 红色填充

// 2. 描边菱形
let mut builder = PathBuilder::stroke(px(3.0)); // 描边模式, 线宽 3px
builder.move_to(point(px(160.), px(20.)));
builder.line_to(point(px(200.), px(50.)));
builder.line_to(point(px(160.), px(80.)));
builder.line_to(point(px(120.), px(50.)));
builder.close();
let path = builder.build().unwrap();
window.paint_path(path, rgb(0x22c55e)); // 绿色描边
```

## 18.3 arc_to/curve_to 曲线路径

除直线外, `PathBuilder` 支持圆弧和贝塞尔曲线. 

**arc_to —— 椭圆弧**(签名与 SVG path 的 `A` 命令一致): 

| 参数 | 说明 |
| :--- | :--- |
| `radii: Point<Pixels>` | 椭圆两个半径, `point(px(30.), px(30.))` 即正圆 |
| `x_rotation: Pixels` | 椭圆轴旋转角度(度), 一般传 `px(0.)` |
| `large_arc: bool` | 走大弧还是小弧 |
| `sweep: bool` | 顺时针(`true`)还是逆时针(`false`) |
| `to: Point<Pixels>` | 弧的终点 |

```rust
// 从当前点画半圆弧到直径另一端(配合第二次 arc_to 就是整圆, 见 17.1)
builder.move_to(point(center.x + radius, center.y));
builder.arc_to(point(radius, radius), px(0.), false, false, point(center.x - radius, center.y));
```

**NOTE**: `arc_to` **没有** "圆心 + 半径 + 起始角 + 扫过角" 这种签名. 原文的 `arc_to(center, radius, start_angle, sweep_angle)` 是别的图形库的写法, 在GPUI里编译不过 —— 圆心、角度都要自己换算成"起点 + 终点 + 两个半径"传进去. 

**curve_to —— 贝塞尔曲线**:  `curve_to(to, ctrl)` 是**二次**贝塞尔(一个控制点), 注意**终点在前、控制点在后**;  三次贝塞尔(两个控制点)用 `cubic_bezier_to(to, control_a, control_b)`. 

**NOTE**: 原文的 `curve_to(control1, control2, end_point)` 三次贝塞尔写法**不对** —— `curve_to` 是二次贝塞尔, 且参数顺序相反. 

```rust
// 一条下垂的曲线: 从左下到右下, 控制点把线往下拉
let mut builder = PathBuilder::stroke(px(2.0));
builder.move_to(point(px(40.), px(40.)));
builder.curve_to(
    point(px(200.), px(40.)), // to: 终点
    point(px(120.), px(100.)), // ctrl: 控制点(往下拉)
);
let path = builder.build().unwrap();
window.paint_path(path, gpui::blue());
```

## 18.4 paint_path/paint_quad

构建好的路径和矩形通过 `Window` 的绘制方法提交: 

| API | 用途 | 说明 |
| :--- | :--- | :--- |
| `window.paint_path(path, color)` | 填充/描边任意路径 | 填充还是描边早已由 PathBuilder 决定; `color` 收 `impl Into<Background>` |
| `window.paint_quad(quad)` | 画矩形(圆角/边框/背景) | 收 `PaintQuad` 结构体, 用下面三个函数构造 |

`PaintQuad` 的三个构造函数(都在 `gpui` 根下): 

| 函数 | 用途 |
| :--- | :--- |
| `quad(bounds, corner_radii, background, border_widths, border_color, border_style)` | 全参数版 |
| `fill(bounds, background)` | 纯填充的矩形 |
| `outline(bounds, border_color, border_style)` | 1px 描边的空心矩形 |

**NOTE**: `paint_quad` 收的是 `PaintQuad` 值, **没有** `cx.paint_quad(rect, fill, radius)` 这个签名;  绘图方法也挂在 `Window` 上, `App`/`Context` 上没有. 

```rust
// 一个黑底方块压在半透明红方块下面(painting.rs 的对照实验)
window.paint_quad(quad(
    Bounds { origin: point(px(70.), px(70.)), size: size(px(40.), px(40.)) },
    px(0.),                  // 圆角半径: border-radius: 0
    gpui::black(),           // 背景色: background-color: #000000
    px(0.),                  // 边框宽度: border-width: 0
    gpui::transparent_black(),// 边框颜色(透明)
    Default::default(),      // 边框样式: 实线
));

// 半透明红方块盖上去, 叠加处颜色加深
window.paint_quad(quad(
    Bounds { origin: point(px(50.), px(50.)), size: size(px(80.), px(80.)) },
    px(0.),
    rgb(0xFF0000).alpha(0.5), // 半透明红: background-color: rgb(255 0 0/0.5)
    px(0.),
    gpui::transparent_black(),
    Default::default(),
));
```

💡 提示: `paint_path` 的第二个参数收 `Background`, 所以纯色、渐变(`linear_gradient`)、图案(`pattern_slash`)都能直接填充路径(见第 18 章). 

## 18.5 鼠标绘图交互(核心思路)

[`examples/painting.rs`](../examples/painting.rs) 是一个完整画板: 按住左键拖动涂鸦, 按住 Shift 画水平/垂直直线. 核心是"事件改状态 + `cx.notify()` 触发 canvas 重绘": 

1. `on_mouse_down` 记录起点, 开始一条新笔画;
2. `on_mouse_move` 把当前点追加进笔画, `cx.notify()` 重绘;
3. `on_mouse_up` 结束当前笔画;
4. Shift 吸附: 把当前点钉到过起点的水平或垂直线上. 

```rust
.on_mouse_down(
    gpui::MouseButton::Left,
    cx.listener(|this, ev: &MouseDownEvent, _, _| {
        this._painting = true;
        this.start = ev.position;
        this.lines.push(vec![ev.position]); // 新笔画
    }),
)
.on_mouse_move(cx.listener(|this, ev: &MouseMoveEvent, _, cx| {
    if !this._painting { return; }
    let mut pos = ev.position;
    if ev.modifiers.shift {
        // Shift 吸附到水平/垂直方向
        let dx = pos.x - this.start.x;
        let dy = pos.y - this.start.y;
        if dx.abs() > dy.abs() {
            pos.y = this.start.y;
        } else {
            pos.x = this.start.x;
        }
    }
    if let Some(path) = this.lines.last_mut() {
        path.push(pos);
    }
    cx.notify(); // 通知重绘: canvas 闭包按新数据再画一遍
}))
```

每帧绘制时把所有笔画转成 `PathBuilder::stroke` 路径, 逐段 `line_to`, 一次 `paint_path` 画完;  要虚线就在 builder 上 `.dash_array(&[px(4.), px(2.)])`. 

## 18.6 复杂图形: Rust logo、饼图、波浪线

基本操作组合起来能画相当复杂的图形, [`painting.rs`](../examples/painting.rs) 里有三个经典案例: 

- **Rust logo**:  直接借用 lyon 自带的 logo 路径(`lyon::extra::rust_logo::build_logo_path`), 转成 `PathBuilder` 后 `translate()`/`scale()` 挪到合适位置. 
- **饼图**:  每个扇形 = `move_to(弧起点)` + `arc_to(弧终点)` + `line_to(圆心)` + `close()`, 每段填不同颜色. 
- **波浪线**:  描边路径逐点 `line_to` 采样正弦函数(50 个点), 比贝塞尔更容易精确控制形状;  配合 `StrokeOptions` 还能调连接处样式(`.with_style(PathStyle::Stroke(options))`). 

## 18.7 底层绘图速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **画布** | `canvas(prepaint, paint)` | 双闭包元素: 测量 + 绘制 |
| **路径构建** | `PathBuilder::fill()` `PathBuilder::stroke(width)` | 填充/描边模式(创建时定) |
| **路径基本操作** | `move_to()` `line_to()` `close()` `add_polygon()` | 直线与多边形 |
| **曲线** | `arc_to(radii, x_rotation, large_arc, sweep, to)` `curve_to(to, ctrl)` `cubic_bezier_to(to, a, b)` | 椭圆弧/二次/三次贝塞尔 |
| **路径修饰** | `dash_array()` `translate()` `scale()` `rotate()` `with_style()` | 虚线、变换、描边样式 |
| **构建** | `builder.build()` | 返回 `Result<Path<Pixels>>` |
| **绘制** | `window.paint_path(path, bg)` `window.paint_quad(paint_quad)` | 画路径/画矩形 |
| **PaintQuad 构造** | `quad()` `fill()` `outline()` | 全参数/纯填充/描边矩形 |
| **交互** | `on_mouse_down` `on_mouse_move` `on_mouse_up` + `cx.notify()` | 状态驱动重绘 |

完整示例: [`cargo run --example painting`](../examples/painting.rs)(画板 + logo/饼图/波浪线 + 半透明叠加), 性能对照(2000 个渐变星形)见 [`cargo run --example paths_bench`](../examples/paths_bench.rs). 

## 18.8 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`painting`](../examples/painting.rs) | 画板: 左键涂鸦、Shift 画直线, Solid/Dashed/Clear 三态; 另有 Rust logo、饼图、波浪线和半透明叠加(见 18.1–18.6) |
| [`paths_bench`](../examples/paths_bench.rs) | 2000 个渐变星形的路径渲染性能对照 |
