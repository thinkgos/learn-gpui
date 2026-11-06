# 19. 渐变与图案

渐变和图案本质都是 `Background` 值, 既能做元素背景(见 3.4、3.5), 也能填充 canvas 自绘的任意路径. 本章重点讲**绘图场景里的用法**和**色彩空间**, 与第 3 章互补不重复. 

## 19.1 linear_gradient 线性渐变

`linear_gradient()` 构造线性渐变, 与纯色一样直接塞进 `.bg()` 或 `paint_path`/`paint_quad`: 

```rust
use gpui::{linear_color_stop, linear_gradient};

// linear_gradient(角度, 起始色标, 结束色标)
let gradient = linear_gradient(
    45.0, // 角度(度): 0 = 向上, 顺时针增大
    linear_color_stop(gpui::red(), 0.0), // 起始色标: 颜色 + 位置(0.0~1.0)
    linear_color_stop(gpui::blue(), 1.0), // 结束色标
);

div().bg(gradient) // 渐变当背景: background-image: linear-gradient(45deg, ...)
```

角度取 0~360 的度数: 0 是"从下往上", 90 是"从左往右", 180 是"从上往下", 与 CSS `linear-gradient` 的方向约定一致. 渐变**只收两个色标**(from/to), 中间不能插更多. 

在 canvas 里填充路径(取自 [`examples/gradient.rs`](../examples/gradient.rs), 梯形内红蓝渐变): 

```rust
canvas(move |_, _, _| {}, move |bounds, _, window, _| {
    // square_bounds/horizontal_offset/vertical_offset 由 bounds 算出(见完整示例)
    let mut builder = PathBuilder::fill();
    builder.move_to(square_bounds.bottom_left());
    builder.line_to(square_bounds.origin + point(horizontal_offset, vertical_offset));
    builder.line_to(square_bounds.top_right() + point(-horizontal_offset, vertical_offset));
    builder.line_to(square_bounds.bottom_right());
    builder.close();
    let path = builder.build().unwrap();

    // 渐变直接当"颜色"用, 填充任意路径
    window.paint_path(
        path,
        linear_gradient(
            180., // 从上往下
            linear_color_stop(gpui::red(), 0.),
            linear_color_stop(gpui::blue(), 1.),
        ),
    );
})
```

**NOTE**: `linear_gradient` **没有** "起止点 + 色标列表" 的签名. 原文的 `linear_gradient(start_point, end_point, color_stops, color_space)` 编译不过 —— 真实签名是 `linear_gradient(angle, from, to)`:  收一个角度 + **恰好两个色标**, 没有多色标, 也没有径向/锥形渐变. 

**NOTE**: 色标函数是 `linear_color_stop(color, percentage)` —— **颜色在前, 位置在后**;  原文的 `linear_color_stop(offset, color)` 顺序写反了. 

渐变填充任意形状都没问题, 比如 [`painting.rs`](../examples/painting.rs) 里给星形填黄橙渐变: 

```rust
// 十角星路径(略): 逐点 line_to 后 close
let path = builder.build().unwrap();
window.paint_path(
    path,
    linear_gradient(
        180., // 从上往下
        linear_color_stop(rgb(0xFACC15), 0.7), // 黄色铺到 0.7 处, 之后过渡到橙
        linear_color_stop(rgb(0xD56D0C), 1.), // 橙色收尾
    )
    .color_space(ColorSpace::Oklab), // 感知均匀插值, 黄橙过渡更顺
);
```

💡 提示: 渐变条带的位置不必铺满 0~1. 两个色标写 0.05 和 0.95, 渐变带压缩到中间, 两端各留一段纯色;  两个色标位置相同(比如都是 0.5)得到的是硬边分界. 渐变还支持 `.opacity(0.5)` 整体调透明度. 

## 19.2 ColorSpace 色彩空间

色彩空间决定两个色标之间**怎么插值**.GPUI支持两种, 用 `.color_space()` 链式设置: 

| 取值 | 特点 | 适用场景 |
| :--- | :--- | :--- |
| `ColorSpace::Srgb` | 在屏幕 RGB 空间线性插值, 与 CSS 默认行为一致;  中段容易发灰发暗 | 要和网页/设计稿像素级对齐时 |
| `ColorSpace::Oklab` | 在感知均匀的空间插值, 过渡自然, 中间色接近人眼预期 | UI 渐变、品牌色过渡(想要好看就选它) |

```rust
let gradient = linear_gradient(
    90.0,
    linear_color_stop(gpui::red(), 0.0),
    linear_color_stop(gpui::blue(), 1.0),
)
.color_space(ColorSpace::Oklab); // 在 Oklab 空间插值: 红蓝之间会出紫色, 而不是脏灰色
```

为什么需要它:  从红到蓝的渐变在 sRGB 里线性插值, 中段是偏暗的"脏紫色";  Oklab 在感知均匀的空间里插值, 中段呈现自然的紫色过渡. [`examples/gradient.rs`](../examples/gradient.rs) 顶部有个切换按钮, 点一下就能对比两种空间下的所有渐变. 想自己做对比实验, 同一组色标画两根条即可: 

```rust
// 同一对色标, 只换插值空间
let stops = |space: ColorSpace| {
    linear_gradient(
        90.0,
        linear_color_stop(gpui::red(), 0.0),
        linear_color_stop(gpui::blue(), 1.0),
    )
    .color_space(space)
};

div().flex().gap_3() // 弹性布局: display: flex; gap: 0.75rem (12px)
    .child(div().flex_1().h_10().bg(stops(ColorSpace::Srgb)))  // 左: Srgb, 中段发灰
    .child(div().flex_1().h_10().bg(stops(ColorSpace::Oklab))) // 右: Oklab, 中段是自然的紫
```

**NOTE**: 默认值是 `ColorSpace::Srgb`, **不是** `Oklab` —— 想要感知均匀的过渡必须显式 `.color_space(ColorSpace::Oklab)`. "GPUI 推荐默认使用 Oklab"的说法不准确, 推荐与否是审美选择, 默认行为跟 CSS 一致. 

💡 提示: 色彩空间只影响 `LinearGradient` 的插值. 纯色、图案(`pattern_slash`/`checkerboard`)没有插值过程, 设置 `color_space` 对它们无效. 

## 19.3 pattern_slash 图案填充

图案也是 `Background` 的一种,GPUI内置两种: 

| 函数 | 效果 | 参数 |
| :--- | :--- | :--- |
| `pattern_slash(color, width, interval)` | 斜线网纹 | 线宽、间距(普通 `f32`, 不套 `px()`) |
| `checkerboard(color, size)` | 棋盘格 | 格子边长 |

```rust
// 斜线图案: 红色斜线, 线宽 4, 间距 4
div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(gpui::red(), 4.0, 4.0))

// 棋盘格: 红色与透明相间的方格, 格子边长 8
div().size(px(64.0)).bg(checkerboard(gpui::red(), 8.0))
```

在 canvas 里同样能用 —— 图案就是 `paint_quad` 的背景参数: 

```rust
canvas(move |_, _, _| {}, move |bounds, _, window, _| {
    window.paint_quad(quad(
        bounds,
        px(12.0),                       // 圆角: border-radius: 12px
        pattern_slash(gpui::red(), 4.5, 4.5), // 斜线图案当背景
        px(0.),
        gpui::transparent_black(),
        Default::default(),
    ));
})
```

**NOTE**: 图案只有 `pattern_slash` 和 `checkerboard` 两种, **没有** `pattern_grid`/`pattern_dots`/`pattern_checker` 这些函数. 想要网格或点阵, 在 canvas 里用 `paint_path` 自己画. 

参数怎么取:  `width` 和 `interval` 是普通 `f32`(不套 `px()`), 分别是斜线的线宽和相邻斜线的间距. 要让不同尺寸的元素斜线密度一致, 按元素高度等比取值, [`pattern.rs`](../examples/pattern.rs) 里就是这么做的: 

```rust
// 高 56px 的条: 线宽 = 间距 = 56/6, 斜线粗细跟着元素走
div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(gpui::red(), 56.0/6.0, 56.0/6.0))

// 同样线宽间距, 换个颜色就是另一种底纹
div().w(px(256.0)).h(px(56.0)).bg(pattern_slash(gpui::blue(), 56.0/6.0, 56.0/6.0))
```

💡 提示: 斜线图案常用来表达"禁用/加载中/装饰底纹". 要"绿底 + 斜线"的叠加效果, 把纯色背景和图案放在父子两层元素上(外层 `.bg(gpui::green().opacity(0.16))`, 内层 `.bg(pattern_slash(...))`), 图案本身只有一种颜色, 不带底色. 

## 19.4 渐变与图案功能对比

四类 `Background` 各有所长, 且**同一个值可以同时用于 `.bg()`、`paint_path`、`paint_quad`** —— 一份填充三处用: 

| 填充 | 构造 | 参数 | 典型用途 |
| :--- | :--- | :--- | :--- |
| 纯色 | `rgb()`/`gpui::red()` | 颜色 | 常规背景、路径填充 |
| 线性渐变 | `linear_gradient()` | 角度 + 两个色标 + 可选色彩空间 | 按钮、卡片装饰、渐变形状 |
| 斜线图案 | `pattern_slash()` | 颜色、线宽、间距 | 禁用态、加载态、底纹 |
| 棋盘格 | `checkerboard()` | 颜色、格子边长 | 透明底衬、对齐检查 |

**渐变 vs 图案**:  渐变是连续的颜色过渡(受色彩空间影响), 图案是平铺的重复纹理(与色彩空间无关). 渐变适合做"一块色面", 图案适合做"提示性的纹理". 

**元素背景 vs canvas 填充**:  `.bg()` 走样式系统, 跟随布局尺寸自动铺满;  canvas 里的 `paint_path`/`paint_quad` 是自由绘图, 坐标、形状、叠放次序全由自己控制(见第 17 章). 渐变的角度语义两边完全一致(0 向上, 顺时针), 但铺满的是各自绘制内容所在的区域, 也不能像 CSS 那样单独指定渐变线的起止点 —— 能调的只有角度和两个色标的位置. 

## 19.5 渐变与图案速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **渐变** | `linear_gradient(angle, from, to)` | 线性渐变, 角度制, 恰好两个色标 |
| **色标** | `linear_color_stop(color, percentage)` | 颜色在前, 位置(0.0~1.0)在后 |
| **色彩空间** | `.color_space(ColorSpace::Srgb/::Oklab)` | 插值空间, 默认 `Srgb` |
| **渐变调整** | `.opacity(factor)` | 整体调透明度 |
| **斜线图案** | `pattern_slash(color, width, interval)` | 线宽/间距是普通 `f32` |
| **棋盘格** | `checkerboard(color, size)` | 格子边长 |
| **使用位置** | `.bg(...)` `window.paint_path(path, bg)` `window.paint_quad(quad(..., bg, ...))` | 同一 `Background` 三处通用 |

完整示例: [`cargo run --example gradient`](../examples/gradient.rs)(色彩空间对比 + canvas 渐变路径), [`cargo run --example pattern`](../examples/pattern.rs)(斜线/棋盘格参数对照). 

## 19.6 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`gradient`](../examples/gradient.rs) | 色彩空间对比 + canvas 里的渐变路径(见 19.1、19.2) |
| [`pattern`](../examples/pattern.rs) | 斜线/棋盘格的参数对照(见 19.3) |
| [`painting`](../examples/painting.rs) | 星形路径填黄橙渐变, 演示渐变填充任意形状(见 19.1; 画板本体见第 18 章) |
