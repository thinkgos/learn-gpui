# 3. 布局系统

GPUI的布局引擎是**Taffy**(CSS Flexbox/Grid 规范的Rust实现), 布局概念与CSS基本一致, 但API是**Tailwind**风格的链式方法(定义在 `Styled` trait 上, `div()` 天然支持).

## 3.1 长度单位

布局 API 接受 4 种长度值:

| 函数 | 类型 | 含义 | 对应 CSS |
| :-- | :-- | :-- | :-- |
| `px(n)` | 绝对 | 逻辑像素, 自动适配 DPI 缩放 | `px` |
| `rems(n)` | 绝对 | 根字号倍数, 默认 1rem = 16px, 跟随GPUI持有的一个基准字号设置`window.set_rem_size(...)` | `rem` |
| `relative(n)` | 相对 | 父容器对应轴的 0.0-1.0 比例 | `%` |
| `auto()` | 自动 | 由内容/上下文决定 | `auto` |

类型层次决定哪些方法收哪些值:

- `AbsoluteLength` = `px` | `rems` (纯绝对值)
- `DefiniteLength` = 绝对值 | `relative(...)` (确定值, 不含 auto)
- `Length` = `DefiniteLength` | `auto()` (最宽)

规则: **padding/gap 收 `DefiniteLength`(不能 auto), margin/宽高/定位偏移收 `Length`(可 auto)**.

```rust
div()
    .w(px(500.0))          // 固定 500px
    .h(rems(2.0))          // 2rem(默认 32px)
    .min_w(relative(0.5))  // 最小宽度为父容器的 50%
    .max_w(auto())         // 最大宽度 auto
```

**NOTE**:GPUI**没有** `fr()` 函数. 比例单位是 `relative(0.0-1.0)`; `fr` 语义由 `flex_1()`/`grid_cols(n)` 这类专用 API 承担. 其他教程里出现的 `fr(1.0)`/`repeat()`/`minmax()` 写法在 zed 的GPUI中编译不过.

## 3.2 Flexbox布局

Flexbox 是GPUI中最常用的布局方式. 通过`.flex()`启用弹性布局, 然后用方向, 对齐, 间距等修饰方法控制子元素的排列.

### 3.2.1 Flexbox 方向API

```rust
div()
  .flex()              // display: flex(默认 row 方向)
  .flex_row()          // flex-direction: row(水平排列, 默认值)
  .flex_col()          // flex-direction: column(垂直排列)
  .flex_row_reverse()  // flex-direction: row-reverse
  .flex_col_reverse()  // flex-direction: column-reverse

// 换行
.flex_nowrap()         // flex-wrap: nowrap(默认)
.flex_wrap()           // flex-wrap: wrap
.flex_wrap_reverse()   // flex-wrap: wrap-reverse
```

### 3.2.2 Flexbox 对齐API

```rust
// 交叉轴对齐(align-items)
.items_start()      // align-items: flex-start
.items_center()     // align-items: center
.items_end()        // align-items: flex-end
.items_baseline()   // align-items: baseline
.items_stretch()    // align-items: stretch

// 单个子元素覆盖交叉轴对齐(align-self)
.self_start()       // align-self: flex-start
.self_center()      // align-self: center
.self_end()         // align-self: flex-end
.self_baseline()    // align-self: baseline
.self_stretch()     // align-self: stretch

// 主轴对齐(justify-content)
.justify_start()    // justify-content: flex-start
.justify_center()   // justify-content: center
.justify_end()      // justify-content: flex-end
.justify_between()  // justify-content: space-between
.justify_around()   // justify-content: space-around
.justify_evenly()   // justify-content: space-evenly

// 多行对齐(align-content, 只在多行 flex 中有意义)
.content_normal()   // align-content: normal
.content_start()    // align-content: flex-start
.content_center()   // align-content: center
.content_end()      // align-content: flex-end
.content_between()  // align-content: space-between
.content_around()   // align-content: space-around
.content_evenly()   // align-content: space-evenly
.content_stretch()  // align-content: stretch
```

### 3.2.3 Flexbox 间距与弹性 API

```rust
// 子元素间距
.gap(px(8.0))       // gap: 8px(行和列)
.gap_x(px(8.0))     // column-gap: 8px
.gap_y(px(8.0))     // row-gap: 8px

// 弹性简写(子元素上使用)
.flex_1()           // flex: 1 1 0%(等分剩余空间, 忽略自身内容宽度)
.flex_auto()        // flex: 1 1 auto(可伸可缩, 基准为内容宽度)
.flex_initial()     // flex: 0 1 auto(默认, 可收缩不可拉伸)
.flex_none()        // flex: 0 0 auto(完全固定)

// 弹性细分(子元素上使用)
.flex_basis(px(120.0))  // flex-basis: 120px
.flex_grow(1.0)         // flex-grow: 1
.flex_grow_0()          // flex-grow: 0
.flex_grow_1()          // flex-grow: 1
.flex_shrink(1.0)       // flex-shrink: 1
.flex_shrink_0()        // flex-shrink: 0(禁止收缩)
.flex_shrink_1()        // flex-shrink: 1
```

> 💡 提示: `flex_1()` 和 `flex_auto()` 的区别
> 两者都会拉伸占满剩余空间, 但基准不同: `flex_1()` 的 `flex-basis: 0%` 表示**无视自身内容宽度**, 空间完全按 grow 系数等分(做等宽导航栏用它); `flex_auto()` 的 `flex-basis: auto` 表示**先按内容宽度分配**, 剩余空间再按 grow 系数分(做内容不等长的工具栏用它).

## 3.3 CSS Grid 布局

CSS Grid 适用于二维布局(同时控制行和列). 本版GPUI的 Grid API 是简化版: **轨道按数量统一生成(默认等分)**, 不支持自定义轨道列表.

```rust
div()
    .grid()             // display: grid
    .grid_rows(3)       // grid-template-rows: repeat(3, minmax(0, 1fr)) -> 3 行等分
    .grid_cols(2)       // grid-template-columns: repeat(2, minmax(0, 1fr)) -> 2 列等分
    .gap(px(16.0))      // gap: 16px
```

轨道生成规则(内部映射到 taffy):

| 方法 | 内部语义 | 说明 |
| :-- | :-- | :-- |
| `grid_rows(n)` | `repeat(n, minmax(0, 1fr))` | n 行等分 |
| `grid_rows_min_content(n)` | `repeat(n, minmax(min-content, 1fr))` | 行不收缩到小于内容高度 |
| `grid_rows_max_content(n)` | `repeat(n, minmax(0, max-content))` | 行按内容高度撑开 |
| `grid_cols(n)` | `repeat(n, minmax(0, 1fr))` | n 列等分 |
| `grid_cols_min_content(n)` | `repeat(n, minmax(min-content, 1fr))` | 列不收缩到小于内容宽度 |
| `grid_cols_max_content(n)` | `repeat(n, minmax(0, max-content))` | 列按内容宽度撑开 |

放置API(在 grid 子项上使用):

| 方法 | 说明 |
| :-- | :-- |
| `col_start(n)`/`col_start_auto()` | 起始列线 |
| `col_end(n)`/`col_end_auto()` | 结束列线 |
| `col_span(n)`/`col_span_full()` | 跨 n 列/跨满整行 |
| `row_start(n)`/`row_start_auto()` | 起始行线 |
| `row_end(n)`/`row_end_auto()` | 结束行线 |
| `row_span(n)`/`row_span_full()` | 跨 n 行/跨满整列 |

示例: 2x2 仪表盘, 右侧卡片跨两行:

```rust
div()
    .size_full()
    .grid()
    .grid_rows(2) // 2 行等分
    .grid_cols(2) // 2 列等分
    .gap(px(16.0))
    .child(card("CPU"))
    .child(card("内存").row_span(2)) // 右侧跨两行
    .child(card("磁盘"))
```

**NOTE**: "圣杯布局"(顶栏 auto + 主区域 1fr + 底栏 auto)这类**不等高行**用 Grid 表达不了(轨道整组同一规则, 无法逐条混排 auto/1fr/定高), 应该用 Flex 纵向嵌套:

```rust
div()
    .size_full()
    .flex()
    .flex_col()
    .child(topbar.h(px(48.0)))    // 定高顶栏
    .child(
        div()
            .flex()
            .flex_1()             // 主区域吃掉剩余空间
            .child(sidebar.w(px(240.0)))  // 定宽侧栏
            .child(main.flex_1()),       // 主内容弹性
    )
    .child(bottombar.h(px(32.0))) // 定高底栏
```

## 3.4 定位系统

GPUI 支持 CSS 的 position 模型中的两种: `.relative()`(相对定位)和 `.absolute()`(绝对定位). 绝对定位的元素相对于最近的 `.relative()`/`.absolute()` 祖先定位.

**NOTE**:GPUI**没有** `.fixed()` 方法(底层 taffy 支持, 但 `Styled` trait 未导出). 全屏遮罩/浮层的常规做法是 `absolute` + 四边偏移拉满.

```rust
// 定位模式
.relative()            // position: relative
.absolute()            // position: absolute

// 偏移量(配合 absolute/relative 使用, 可为负值)
.inset(px(0.0))       // 同时设置 top/right/bottom/left
.top(px(0.0))         // top: 0
.left(px(0.0))        // left: 0
.right(px(0.0))       // right: 0
.bottom(px(0.0))      // bottom: 0
.top_auto()           // top: auto
.left(px(-4.0))       // 负偏移: 常用于徽标溢出角(简写 .left_neg_1())
```

典型用法: 全屏覆盖层

```rust
div()
    .relative()             // 作为定位参考
    .size_full()
    .child(
        div()
            .absolute()         // 绝对定位
            .inset(px(0.0))     // 四边拉满
            .bg(rgba(0x00000080)), // 半透明黑色遮罩
    )
```

**NOTE**: `rgba()` **没有** `rgba(色值, 透明度)` 这种两参数写法, 它只收一个 8 位十六进制常量(红绿蓝 alpha 各 8 位), 如 `rgba(0x00000080)` 是 50% 透明的黑. 要单独调透明度用 `color.alpha(0.5)`.

## 3.5 间距系统

GPUI 提供完整的 `padding`(内边距)和 `margin`(外边距)控制, 与 CSS 的简写规则一致.

```rust
// Padding(内边距, 不支持 auto)
.p(px(16.0))     // padding: 16px(四边)
.px(px(16.0))    // padding-left + padding-right: 16px
.py(px(8.0))     // padding-top + padding-bottom: 8px
.pt(px(8.0))     // padding-top: 8px
.pr(px(8.0))     // padding-right: 8px
.pb(px(8.0))     // padding-bottom: 8px
.pl(px(8.0))     // padding-left: 8px

// Margin(外边距, 支持 auto)
.m(px(16.0))     // margin: 16px
.mx(px(16.0))    // margin-left + margin-right: 16px
.my(px(8.0))     // margin-top + margin-bottom: 8px
.mt(px(8.0))     // margin-top: 8px
.mr(px(8.0))     // margin-right: 8px
.mb(px(8.0))     // margin-bottom: 8px
.ml(px(8.0))     // margin-left: 8px
.mx(auto())      // margin-left/right: auto(水平居中经典手法)

// Gap(子元素间距, Flex/Grid 容器上使用)
.gap(px(8.0))     // gap: 8px
```

**NOTE**: `.px()` 方法是"水平 padding", 而 `px()` 函数是"像素单位" -- 同名不同物. `.px(px(16.0))` 里两个 `px` 含义完全不同, 初学容易看晕.

## 3.6 尺寸控制

```rust
// 同时设置宽高
.size(px(500.0))       // width: 500px; height: 500px
.size(relative(1.0))   // width: 100%; height: 100%

// 单独设置宽/高
.w(px(500.0))          // width: 500px
.h(px(300.0))          // height: 300px
.w(auto())             // width: auto
.min_w(px(100.0))      // min-width: 100px
.min_h(px(100.0))      // min-height: 100px
.max_w(px(800.0))      // max-width: 800px
.max_h(px(600.0))      // max-height: 600px

// 同时设置最小/最大宽高
.min_size(px(100.0))   // min-width + min-height
.max_size(px(800.0))   // max-width + max-height

// 百分比/全屏
.w_full()              // width: 100%(等价于 .w(relative(1.0)))
.h_full()              // height: 100%
.min_w_full()          // min-width: 100%
.min_h_full()          // min-height: 100%
```

> 💡 提示: `px()` vs `relative()` 的区别
> `px(n)` 表示固定像素值, 在任何布局中都生效. `relative(n)` 表示父容器对应轴的比例(0.0-1.0), `relative(1.0)` = 100%. 而 flex 的"按比例分配剩余空间"不走这两个函数, 用 `flex_1()`/`flex_grow(n)` 或 Grid 的 `grid_cols(n)` 表达.

## 3.7 Tailwind 数值简写

所有 padding/margin/gap/宽高/偏移方法都有一套**数值后缀**简写, 遵循 Tailwind 的间距刻度(数值 n 约等于 4n px), 不用传 `px()`:

| 简写 | 实际值 | 简写 | 实际值 |
| :-- | :-- | :-- | :-- |
| `*_0()` | 0px | `*_6()` | 24px |
| `*_0p5()` | 2px | `*_8()` | 32px |
| `*_1()` | 4px | `*_10()` | 40px |
| `*_1p5()` | 6px | `*_12()` | 48px |
| `*_2()` | 8px | `*_16()` | 64px |
| `*_2p5()` | 10px | `*_20()` | 80px |
| `*_3()` | 12px | `*_24()` | 96px |
| `*_4()` | 16px | `*_32()` | 128px |
| `*_5()` | 20px | | |

此外还有三个特殊后缀和负值变体:

| 简写 | 实际值 |
| :-- | :-- |
| `*_auto()` | auto(仅 margin/宽高/偏移, padding/gap 不支持) |
| `*_full()` | 100%(宽高), 9999px(圆角) |
| `*_px()` | 1px(仅 padding/margin/gap/宽高/偏移等 box 方法, 边框宽度不走这套) |
| `*_neg_2()` 等 | 对应负值 -8px |

```rust
div()
    .p_4()             // padding: 16px
    .px_2()            // padding-left/right: 8px
    .gap_2()           // gap: 8px
    .mt_neg_1()        // margin-top: -4px
    .min_w_24()        // min-width: 96px
```

## 3.8 布局方法速查表

| 类别 | 方法 | 对应 CSS |
| :--- | :--- | :--- |
| **长度单位** | `px()` `rems()` `relative()` `auto()` | `px` `rem` `%` `auto` |
| **Flex 方向** | `.flex()` `.flex_row()` `.flex_col()` `.flex_wrap()` | `display`, `flex-direction`, `flex-wrap` |
| **对齐** | `.items_center()` `.justify_between()` `.self_center()` `.content_start()` | `align-items`, `justify-content`, `align-self`, `align-content` |
| **弹性** | `.flex_1()` `.flex_auto()` `.flex_none()` `.flex_grow()` `.flex_shrink_0()` | `flex`, `flex-grow`, `flex-shrink` |
| **Grid** | `.grid()` `.grid_rows()` `.grid_cols()` `.col_span()` `.row_span()` | `display:grid`, `grid-template-*`, `grid-column`, `grid-row` |
| **定位** | `.absolute()` `.relative()` `.inset()` `.top()` `.left()` | `position`, `top`, `left` |
| **间距** | `.p()` `.px()` `.py()` `.pt()` `.m()` `.mt()` `.gap()` | `padding`, `margin`, `gap` |
| **尺寸** | `.size()` `.w()` `.h()` `.w_full()` `.min_w()` `.max_h()` | `width`, `height`, `min/max-*` |
| **数值简写** | `.p_4()` `.gap_2()` `.mt_neg_1()` `.w_full()` | Tailwind 间距刻度 |

## 3.9 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`grid_layout`](../examples/grid_layout.rs) | 经典圣杯布局(顶部/底部 + 三栏); 拖动窗口宽度体验 `container_query` 响应式折叠(宽于 400px 三栏, 窄于 400px 单列), 同时演示 `grid_cols`/`grid_rows`/`col_span`/`row_span` |
