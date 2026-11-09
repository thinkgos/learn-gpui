# 4. 样式与外观 - 颜色、阴影、渐变与透明度

一个好的 UI 离不开精致的视觉样式. GPUI提供了完整的样式 API, 涵盖颜色、边框、圆角、阴影、渐变、透明度等各个方面, 全部采用与 CSS 一致的命名约定, 定义在 `Styled` trait 上, `div()` 等元素直接链式调用.

## 4.1 颜色系统

GPUI 里颜色有两个底层类型: `Hsla` 和 `Rgba`.

- `rgb()`/`rgba()` 返回 `Rgba`.
- `hsla()` 和具名颜色函数返回 `Hsla`.
- 所有接收颜色的样式方法都接受 `impl Into<Hsla>`, 两种类型可以混用.

### 4.1.1 `rgb()` - 不透明颜色

```rust
// rgb(u32) — 接受 0xRRGGBB 格式的十六进制数, alpha 恒为 1.0
rgb(0xff5733)  // 红色 0xff, 绿色 0x57, 蓝色 0x33
rgb(0x505050)  // 深灰色
rgb(0xffffff)  // 白色
rgb(0x000000)  // 黑色

// 在链式调用中使用
div()
.bg(rgb(0x7c3aed))         // 紫色背景：background-color: #7c3aed
.text_color(rgb(0xffffff)) // 白色文字：color: #ffffff
```

### 4.1.2 `rgba()` - 带透明度的颜色

```rust
// rgba(u32) — 0xRRGGBBAA 格式, 最后两位是 alpha 通道(0x00 全透明 ~ 0xff 不透明)
rgba(0x00000080)  // 半透明黑色(遮罩常用, 0x80 ≈ 50%)
rgba(0xff000033)  // 低透明度红色
rgba(0xffffff1a)  // 10% 白色(轻微高光)

// 示例: 半透明遮罩层
div()
.absolute()
.top(px(0.0))
.left(px(0.0))
.right(px(0.0))
.bottom(px(0.0))
.bg(rgba(0x00000080)) // 半透明黑色背景：background-color: rgb(0 0 0/0.5)
```

**NOTE**: `rgba()` 只收**一个**参数(alpha编码在十六进制数里), 不是 `rgba(color, alpha)` 两个参数. 想"拿一个颜色再调透明度"用 `.alpha()`, 见 4.1.4.

### 4.1.3 HSL颜色模型

```rust
// hsla(h: f32, s: f32, l: f32, a: f32)
// 四个分量都是 0.0~1.0 的小数 —— 色相 h 也是 0~1, 不是 0~360 度!
// 色相换算: 红 = 0.0, 绿 ≈ 0.333, 蓝 ≈ 0.667, 绕一圈回到红 = 1.0
hsla(0.667, 0.8, 0.5, 1.0)  // 不透明蓝色(≈ 240°)
hsla(0.333, 0.6, 0.4, 0.8)  // 80% 不透明绿色(≈ 120°)

// HSL 比 RGB 更直观: 调整亮度即可获得同色系的深浅变体
let base_hue = 0.667;  // 蓝色色相 = 240 ÷ 360
let light_blue  = hsla(base_hue, 0.8, 0.7, 1.0);  // 浅蓝
let dark_blue   = hsla(base_hue, 0.8, 0.3, 1.0);  // 深蓝
```

### 4.1.4 具名颜色与颜色工具

```rust
// 具名颜色函数(返回 Hsla)
gpui::black()   // #000000
gpui::white()   // #ffffff
gpui::red()     // #ff0000
gpui::green()   // #008000
gpui::blue()    // #0000ff
gpui::yellow()  // #ffff00
gpui::transparent_black()  // 完全透明的黑
gpui::transparent_white()  // 完全透明的白

// 十六进制字符串解析成 Rgba(支持 #rgb #rgba #rrggbb #rrggbbaa)
let c: Rgba = "#7c3aed".try_into().unwrap();
```

颜色值自带一组修改方法, 都**返回新值**而不改动原值:

```rust
let brand = rgb(0x7c3aed);

brand.alpha(0.5)      // 直接把 alpha 设成 0.5
brand.opacity(0.5)    // 把 alpha 乘以 0.5(再淡一半)
brand.grayscale()     // 去掉饱和度, 变灰
brand.blend(other)    // 把 other 按它自己的 alpha 叠加到自己上面(混色)
```

**NOTE**: `grayscale()` 只在 `Hsla` 上有, `Rgba` **没有**这个方法. `rgb()` 返回的是 `Rgba`, 上面那行 `brand.grayscale()` 编译不过, 要灰度化得先转 `Hsla`(如 `Hsla::from(brand).grayscale()`); `alpha()`/`opacity()`/`blend()` 两个类型都有.

`Hsla::to_rgb()` 从 `Hsla` 转 `Rgba`, 反向用 `Rgba -> Hsla` 的 `From` 转换.

## 4.2 边框与圆角

### 4.2.1 边框

```rust
div()
.border(px(1.0))             // 四边 1 像素边框：border-width: 1px
.border_color(rgb(0x555555)) // 边框颜色：border-color: #555555

// 按边/成对设置宽度: 
.border_t(px(1.0))   // border-top-width: 1px
.border_b(px(0.0))   // border-bottom-width: 0
.border_l(px(1.0))   // border-left-width: 1px
.border_r(px(0.0))   // border-right-width: 0
.border_x(px(1.0))   // 左右两边：border-left-width + border-right-width: 1px
.border_y(px(1.0))   // 上下两边

// 数值简写(同 Tailwind 刻度, 不用传 px()): 
.border_1()    // 四边 1px：border-width: 1px
.border_t_2()  // 顶边 2px：border-top-width: 2px

// 虚线边框(实线是默认值)
.border_dashed()  // border-style: dashed
```

### 4.2.2 圆角

```rust
div()
.rounded(px(4.0))   // 四角 4px(自定义值)：border-radius: 4px
.rounded_lg()       // 四角 8px(预设, 不收参数)
.rounded_full()     // 完全圆形：border-radius: 9999px

// 按边/按角设置(收自定义值): 
.rounded_t(px(4.0))    // 左上 + 右上：border-top-left-radius + border-top-right-radius: 4px
.rounded_b(px(4.0))    // 左下 + 右下
.rounded_l(px(4.0))    // 左上 + 左下
.rounded_r(px(4.0))    // 右上 + 右下
.rounded_tl(px(4.0))   // 只有左上角：border-top-left-radius: 4px
.rounded_tr(px(2.0))   // 只有右上角
.rounded_bl(px(2.0))   // 左下角
.rounded_br(px(2.0))   // 右下角
```

每个前缀都配一套 Tailwind 风格的**预设后缀**(不收参数):

| 预设 | 半径 |
| :-- | --: |
| `rounded_none()` | 0px |
| `rounded_xs()` | 2px |
| `rounded_sm()` | 4px |
| `rounded_md()` | 6px |
| `rounded_lg()` | 8px |
| `rounded_xl()` | 12px |
| `rounded_2xl()` | 16px |
| `rounded_3xl()` | 24px |
| `rounded_full()` | 9999px |

**NOTE**:  预设后缀可以和任意前缀组合, 例如 `rounded_tl_full()` 做"单角胶囊". |

卡片样式示例:

```rust
div()
.p(px(16.0))                 // 内边距：padding: 16px
.bg(rgb(0x2a2a2a))           // 背景色：background-color: #2a2a2a
.border(px(1.0))             // 边框：border-width: 1px
.border_color(rgb(0x444444)) // 边框颜色：border-color: #444444
.rounded_lg()                // 圆角：border-radius: 8px
.shadow_md()                 // 中等阴影(见下一节)
.child("这是一个卡片")
```

## 4.3 BoxShadow 阴影系统

GPUI 的阴影通过 `BoxShadow` 结构体定义, 并提供了多个预设级别供快速使用. 阴影是提升 UI 层次感的关键手段.

`BoxShadow` 结构体定义与构造:

```rust
pub struct BoxShadow {
    pub color: Hsla,           // 阴影颜色(通常带透明度)
    pub offset: Point<Pixels>, // 阴影偏移量 (x, y)
    pub blur_radius: Pixels,   // 模糊半径
    pub spread_radius: Pixels, // 扩散半径(可为负)
    pub inset: bool,           // true 表示内阴影
}

// 构造是"固定参数 + 链式补可选项"的建造者模式
BoxShadow::new(px(0.0), px(4.0), hsla(0.0, 0.0, 0.0, 0.1))
  .blur_radius(px(6.0))    // 模糊半径：box-shadow 的 blur 部分
  .spread_radius(px(-1.0)) // 扩散半径：box-shadow 的 spread 部分
  .inset()                 // 内阴影：box-shadow: inset ...
```

`.shadow()` 收的是 `Vec<BoxShadow>`, **可以一次叠多层阴影**(列表里后面的画在更上层), `.shadow_none()` 清除全部阴影:

```rust
div()
.shadow(vec![
    // 同一个元素叠两层阴影: 外层大阴影 + 带颜色的小阴影
    BoxShadow::new(px(0.0), px(4.0), hsla(0.0, 0.0, 0.0, 0.3)).blur_radius(px(8.0)),
    BoxShadow::new(px(0.0), px(1.0), hsla(0.667, 0.8, 0.5, 0.4)).blur_radius(px(2.0)),
])
```

GPUI 内置了 7 个阴影预设, 从极轻微到非常明显, 数值与 Tailwind 的 shadow 刻度一致:

| 预设方法 | 典型用途 | 等价 CSS box-shadow |
| -- | -- | -- |
| `.shadow_2xs()` | 细分隔线 | `0 1px 0 0 rgb(0 0 0/0.05)` |
| `.shadow_xs()` | 轻微浮起 | `0 1px 2px 0 rgb(0 0 0/0.05)` |
| `.shadow_sm()` | 按钮悬停、小卡片 | `0 1px 3px 0 rgb(0 0 0/0.1), 0 1px 2px -1px rgb(0 0 0/0.1)` |
| `.shadow_md()` | 卡片、下拉菜单 | `0 4px 6px -1px rgb(0 0 0/0.1), 0 2px 4px -2px rgb(0 0 0/0.1)` |
| `.shadow_lg()` | 弹出面板、对话框 | `0 10px 15px -3px rgb(0 0 0/0.1), 0 4px 6px -4px rgb(0 0 0/0.1)` |
| `.shadow_xl()` | 模态框、重要提示 | `0 20px 25px -5px rgb(0 0 0/0.1), 0 8px 10px -6px rgb(0 0 0/0.1)` |
| `.shadow_2xl()` | 全局拖拽预览、最高层级 | `0 25px 50px -12px rgb(0 0 0/0.25)` |

> 💡 提示: 阴影跟随圆角裁剪. 同一个阴影打在直角/小圆角/大圆角/圆形上, 阴影轮廓始终贴着元素形状. 想看各级别、多层阴影、内阴影的实拍效果, 跑 [`cargo run --example shadow`](../examples/shadow.rs).

## 4.4 渐变背景

`.bg()` 不只收纯色, 渐变也是背景"填充"的一种: 用 `linear_gradient()` 构造, 直接传给 `.bg()`, 和纯色完全平权.

```rust
use gpui::{linear_color_stop, linear_gradient};

div()
.bg(linear_gradient(
    45.0, // 角度(度)：0 = 向上, 顺时针增大, 与 CSS linear-gradient 一致
    linear_color_stop(gpui::red(), 0.0),  // 起始色标：颜色 + 位置(0.0~1.0)
    linear_color_stop(gpui::blue(), 1.0), // 结束色标
))
```

- 角度取 0~360 的**度数**: 0 是"从下往上", 90 是"从左往右", 180 是"从上往下".
- 目前**只支持两个色标**(from/to), 不支持多色标, 也没有径向(radial)/锥形(conic)渐变.
- 两个色标位置相同(比如都写 0.5)得到的是硬边分界, 而不是渐变过渡.
- 色标位置不必铺满 0~1, 写 0.05~0.95 会把渐变带压缩到中间, 两端各留一段纯色.

插值颜色空间:

```rust
let bg = linear_gradient(90.0, linear_color_stop(gpui::red(), 0.0), linear_color_stop(gpui::blue(), 1.0))
.color_space(ColorSpace::Oklab); // 在 Oklab 空间插值, 过渡更自然(默认 ColorSpace::Srgb)
div().bg(bg)
```

💡 提示: 为什么需要 `color_space()`?  sRGB 线性插值的中段容易发灰发暗(红蓝渐变中间会出"脏紫色"), Oklab 在感知均匀的空间里插值, 中间色更接近人眼预期. 对比两种模式跑 [`cargo run --example gradient`](../examples/gradient.rs).

渐变值还能直接用在 `canvas` 自绘里(`window.paint_path(path, gradient)`), 与 `.bg()` 共用同一套类型, 一份渐变两处用.

## 4.5 图案背景

`.bg()` 还接受两种图案填充, 斜线网纹和棋盘格:

```rust
div().bg(pattern_slash(gpui::red(), 4.0, 4.0)) // 斜线图案：线宽 4, 间隔 4
div().bg(checkerboard(gpui::red(), 8.0))       // 棋盘格：格子边长 8
```

`pattern_slash(color, width, interval)` 的 `width`/`interval` 是普通 `f32`(不套 `px()`), 分别是斜线的线宽和间距; `checkerboard(color, size)` 的 `size` 是格子边长. 示例见 [`cargo run --example pattern`](../examples/pattern.rs).

> 💡 提示: `.bg()` 是唯一的背景入口 —— 纯色(`rgb()`)、渐变(`linear_gradient()`)、图案(`pattern_slash()`/`checkerboard()`)三类值都走它, 可以随时互相替换.

## 4.6 透明度

`.opacity()` 修饰符控制整个元素的透明度(包括其子元素).  取值范围为 0.0(完全透明)到 1.0(完全不透明).

```rust
div().opacity(0.5) // 整个元素(含子元素)半透明：opacity: 0.5
```

与 `rgba()` 的区别:

- `rgba(0x00000080)` 只影响背景色的透明度, 文字和子元素不受影响
- `.opacity(0.5)` 影响整个元素及其所有子元素的透明度

想"只改某个颜色的透明度、不牵连子元素", 用颜色自身的 alpha: `rgba(0x00000080)` 或 `gpui::black().alpha(0.5)`. 交互状态下的悬停变淡常用状态样式回调 `.hover(|style| style.opacity(0.5))`(见第 7 章), 整组效果见 [`cargo run --example opacity`](../examples/opacity.rs).

## 4.7 溢出控制

当元素内容超出其限定尺寸时, 溢出控制决定如何显示多余内容.

```rust
// 隐藏溢出内容(任何元素都有)
.overflow_hidden()    // overflow: hidden(x 和 y 方向)
.overflow_x_hidden()  // overflow-x: hidden
.overflow_y_hidden()  // overflow-y: hidden

// 滚动(需要"有状态元素", 见下方 NOTE)
.overflow_scroll()    // overflow: scroll(x 和 y 都可滚)
.overflow_x_scroll()  // overflow-x: scroll
.overflow_y_scroll()  // overflow-y: scroll

// 示例: 固定高度的可滚动列表
div()
.id("list")               // 有 id 才是"有状态元素", 才能用滚动方法
.h(px(300.0))             // 固定高度：height: 300px
.overflow_y_scroll()      // 内容超过 300px 时出现纵向滚动条
.children(/* 列表项 */)
```

**NOTE**: `overflow_hidden()` 系列定义在 `Styled` 上, 任何元素都能用; `overflow_scroll()` 系列定义在 `StatefulInteractiveElement` 上, 只有调用过 `.id(...)` 的元素才有这些方法.  没有 CSS 里 `overflow: auto` 的对应方法 —— 只有 hidden 和 scroll 两档.

> 💡 提示:  `overflow_hidden` 的常见用途
> `.overflow_hidden()` 不仅用于隐藏溢出内容, 还常用于裁剪圆角内的子元素. 当一个元素设置了 `.rounded_lg()` 但子元素有背景色时, 需要给父元素加`.overflow_hidden()` 才能让子元素不溢出圆角. 还可以配合 `.scrollbar_width(px(6.0))` 设置滚动条占的宽度.

## 4.8 可见性、光标与其他

### 4.8.1 显示与可见性

```rust
.hidden()    // display: none(不渲染, 也不占布局空间)
.invisible() // visibility: hidden(不渲染, 但保留占位)
.visible()   // 恢复可见
```

两者就是 CSS `display: none` 与 `visibility: hidden` 的区别: 前者彻底从布局里消失, 后者位置还留着.

### 4.8.2 鼠标光标

```rust
.cursor_pointer()     // cursor: pointer(手型, 可点击元素常用)
.cursor_text()        // cursor: text(I 型)
.cursor_grab()        // cursor: grab
.cursor_not_allowed() // cursor: not-allowed
```

预设和 CSS `cursor` 一一对应, 另有 `cursor_default`/`cursor_move`/`cursor_crosshair`/`cursor_copy`/`cursor_*_resize` 等. 鼠标交互(见第 7 章)时配合悬停效果使用.

### 4.8.3 其他外观方法

```rust
.aspect_ratio(1.5)  // 宽高比：aspect-ratio: 1.5
.aspect_square()    // 正方形(宽高比 1:1)
.debug()            // 调试: 给元素画红色轮廓(仅 debug 构建生效)
```

## 4.9 样式方法速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **颜色构造** | `rgb()` `rgba()` `hsla()` `red()` `white()` ... | 创建 `Rgba`/`Hsla` 颜色值 |
| **颜色工具** | `.alpha()` `.opacity()` `.blend()` `.grayscale()` | 修改颜色的透明度/混色/灰度 |
| **背景** | `.bg(color)` `.bg(linear_gradient(...))` `.bg(pattern_slash(...))` | 纯色/渐变/图案背景 |
| **渐变** | `linear_gradient()` `linear_color_stop()` `.color_space()` | 两色标线性渐变 |
| **文字颜色** | `.text_color(color)` | 设置文字颜色 |
| **边框** | `.border()` `.border_1()` `.border_t()` `.border_color()` `.border_dashed()` | 边框宽度与颜色、虚线 |
| **圆角** | `.rounded()` `.rounded_lg()` `.rounded_tl()` `.rounded_full()` | 边框圆角(预设不收参数) |
| **阴影** | `.shadow_md()` `.shadow(vec![...])` `.shadow_none()` `BoxShadow::new()` | 盒子阴影(支持多层/内阴影) |
| **透明度** | `.opacity(f32)` | 元素整体透明度 |
| **溢出** | `.overflow_hidden()` `.overflow_y_scroll()` | 溢出控制 |
| **可见性** | `.hidden()` `.invisible()` `.visible()` | 显示/隐藏/占位隐藏 |
| **光标** | `.cursor_pointer()` `.cursor_text()` ... | 鼠标指针样式 |
| **其他** | `.aspect_ratio()` `.aspect_square()` `.debug()` | 宽高比、调试轮廓 |

## 4.10 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`shadow`](../examples/shadow.rs) | 7 级阴影预设(2XS~2XL)× 五种形状(直角/圆角 4/8/16/圆形)对照, 另有多层阴影和内阴影(见 4.3) |
| [`gradient`](../examples/gradient.rs) | 多组线性渐变, 顶部按钮切换 Srgb/Oklab, 对比插值空间差异(见 4.4) |
| [`pattern`](../examples/pattern.rs) | 斜线和棋盘格的参数对照, 同高元素的斜线密度保持一致(见 4.5) |
| [`opacity`](../examples/opacity.rs) | 元素整体半透明和颜色自带 alpha 的差异, 可点击面板实测(见 4.6) |
