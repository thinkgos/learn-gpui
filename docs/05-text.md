# 4. 文字和排版

文字样式和布局样式一样, 都是 `Styled` trait 上的 Tailwind 风格方法. 与 CSS 一致, 文字属性(字号、颜色、字重等)写在容器上会**继承**给子元素, 所以常见做法是在父 div 上设一套基础排版, 子元素只写差异部分. 

文字内容本身有三种常见写法: 

| 写法 | 适用场景 |
| :--- | :--- |
| `.child("字符串")` | 普通文字, 最简单 |
| `text!("...")` | 需要无障碍(accessibility)语义的文字节点 |
| `StyledText::new(...)` | 富文本: 一段文字里混多样式(见 4.4) |

## 5.1 基本文字属性

GPUI 提供了一组简洁的修饰符来控制文本的外观: 

| 修饰符 | 说明 | 等价 CSS |
| :--- | :--- | :--- |
| `.text_size()` | 设置字号 | `font-size` |
| `.text_color()` | 设置文字颜色 | `color` |
| `.font_weight()` | 设置字重 | `font-weight` |
| `.italic()` | 斜体 | `font-style: italic` |
| `.line_height()` | 行高 | `line-height` |
| `.text_bg()` | 文字自身的底色 | 文字行的 `background-color` |

```rust
div()
    .flex()      // 弹性布局：display: flex
    .flex_col()  // 纵向排列：flex-direction: column
    .child(
        div()
            .text_size(px(24.0))               // 字号：font-size: 24px
            .font_weight(FontWeight::BOLD)     // 粗体：font-weight: 700
            .text_color(rgb(0x7c3aed))         // 文字颜色：color: #7c3aed
            .child("大号加粗文字"),            // 普通字符串子元素
    )
    .child(
        div()
            .italic()                          // 斜体：font-style: italic
            .text_size(px(16.0))               // 字号：font-size: 16px
            .child("斜体文字"),
    )
    .child(
        div()
            .text_size(px(14.0))               // 字号：font-size: 14px
            .text_color(rgb(0x666666))         // 文字颜色：color: #666666
            .child("普通文字"),
    )
```

### 5.1.1 字号简写

`text_size()` 收任意长度, 另有一套 Tailwind 风格字号简写(不收参数): 

| 简写 | rem | 像素(1rem = 16px) | 简写 | rem | 像素 |
| :--- | --: | --: | :--- | --: | --: |
| `text_xs()` | 0.75rem | 12px | `text_xl()` | 1.25rem | 20px |
| `text_sm()` | 0.875rem | 14px | `text_2xl()` | 1.5rem | 24px |
| `text_base()` | 1rem | 16px | `text_3xl()` | 1.875rem | 30px |
| `text_lg()` | 1.125rem | 18px | | | |

### 5.1.2 字重 FontWeight

`FontWeight` 是取值 100~900 的数值类型(不是枚举), 提供九个常用常量: 

| 常量 | 值 | 常量 | 值 |
| :--- | --: | :--- | --: |
| `FontWeight::THIN` | 100 | `FontWeight::SEMIBOLD` | 600 |
| `FontWeight::EXTRA_LIGHT` | 200 | `FontWeight::BOLD` | 700 |
| `FontWeight::LIGHT` | 300 | `FontWeight::EXTRA_BOLD` | 800 |
| `FontWeight::NORMAL` | 400(默认) | `FontWeight::BLACK` | 900 |
| `FontWeight::MEDIUM` | 500 | | |

任意数值都能直接构造: `FontWeight::from(650.0)`(可变字重字体才有意义). 

### 5.1.3 行高 line_height

```rust
div()
    .line_height(relative(1.5)) // 行高是字号的 1.5 倍：line-height: 1.5
    .line_height(px(24.0))      // 或固定 24 像素：line-height: 24px
```

`relative(1.5)` 这种无单位倍数和 CSS 的 `line-height: 1.5` 是一回事; 传 `px()`/`rems()` 则是绝对行高. 

### 5.1.4 文字底色 text_bg

```rust
div().text_bg(gpui::yellow()) // 给文字行本身上底色(语法高亮常用)
```

`text_bg()` 作用于**文字行**, `.bg()` 作用于**整个盒子**, 两者不同. 

💡 提示: 文字样式会继承 —— `.text_color()`/`.font_weight()` 等写在容器 div 上, 里面所有文字子元素都跟着变. 但注意这是**样式继承**, 不是 CSS 的层叠(cascade): 写在子元素上的样式只会覆盖父容器同名项. 

## 5.2 文本溢出处理

当文本超出容器边界时,GPUI提供了多种溢出处理方式: 

| 方法 | 说明 | 等价 CSS |
| :--- | :--- | :--- |
| `text_ellipsis()` | 单行省略, 超出部分显示为 … | `text-overflow: ellipsis` |
| `text_ellipsis_start()` | 省略开头、保留结尾(路径类文本) |GPUI扩展 |
| `text_ellipsis_middle()` | 省略中间、保留头尾(文件名类) |GPUI扩展 |
| `truncate()` | 单行截断 + 省略号, 三合一 | `overflow: hidden` + `white-space: nowrap` + `text-overflow: ellipsis` |
| `line_clamp(n)` | 多行省略, 最多 n 行 | `-webkit-line-clamp: n` |
| `text_overflow(overflow)` | 自定义截断标记 |GPUI扩展(见下) |
| `whitespace_nowrap()` | 禁止换行, 强制单行 | `white-space: nowrap` |
| `whitespace_normal()` | 恢复自动换行(默认) | `white-space: normal` |

几个组合的典型用法: 

```rust
// 1. 三合一快捷方式: 单行截断 + 省略号
div().truncate().child(long_text)
// 等价于依次调用 overflow_hidden() + whitespace_nowrap() + text_ellipsis()

// 2. 多行省略: line_clamp(n) 只负责"最多 n 行 + 裁掉"
//    想在截断处出现 …, 还要配一个 text_ellipsis()
div().line_clamp(2).text_ellipsis().child(long_text)

// 3. 自定义截断标记(截断标记是字符串)
div().text_overflow(TextOverflow::Truncate("".into()))       // 空标记 = 直接裁掉, 什么都不显示
div().text_overflow(TextOverflow::TruncateStart("…".into())) // 从头部截, 标记补在开头
div().text_overflow(TextOverflow::TruncateMiddle("…".into())) // 从中间截, 头尾都保留
```

**NOTE**: 和 CSS 不同, `text_ellipsis()` 在GPUI里**自带"截成单行"**的效果, 不需要再配 `whitespace_nowrap()`. 另外 `line_clamp(n)` 本身只裁不加省略号, 配合 `text_ellipsis()` 才会在截断处显示 …. 

**NOTE**: `TextOverflow` 是枚举, 三个变体 `Truncate(marker)`/`TruncateStart(marker)`/`TruncateMiddle(marker)` 分别是尾部、头部、中间截断, 其中 `marker` 就是截断处显示的字符串(`"…"` 是省略号, `""` 是硬裁切). 

💡 提示: flex 布局里的截断

flex 子项默认会被压缩, 演示/使用截断时给盒子加 `.flex_shrink_0()`, 否则"被压扁"会盖过"被截断". 不同类型文本的断行表现也值得知道: 超长英文单词不会从词中间断开, 中日韩逐字断行, URL 会在内部标点处断开. 完整对照跑 `cargo run --example text_wrapper`. 

## 5.3 文本装饰与对齐

### 5.3.1 下划线与删除线

```rust
div().underline()    // 下划线：text-decoration-line: underline
div().line_through() // 删除线：text-decoration-line: line-through

// 下划线的粗细、颜色、线型可以继续调: 
.underline()
.text_decoration_2()                 // 线宽 2px(0/1/2/4/8 五档)：text-decoration-thickness: 2px
.text_decoration_color(gpui::red())  // 线的颜色：text-decoration-color: #ff0000
.text_decoration_wavy()              // 波浪线(拼写检查风格)：text-decoration-style: wavy
.text_decoration_solid()             // 实线(默认)：text-decoration-style: solid

.text_decoration_none()              // 去掉下划线：text-decoration: none
```

**NOTE**: `text_decoration_0()`/`_1()`/`_2()`/`_4()`/`_8()` 编码的是**下划线粗细**;  `text_decoration_none()` 只清除下划线, 不影响 `line_through()` 的删除线. 

### 5.3.2 文本对齐

```rust
div().text_left()   // 左对齐(默认)：text-align: left
div().text_center() // 居中：text-align: center
div().text_right()  // 右对齐：text-align: right

// 通用形式
div().text_align(TextAlign::Center) // text-align: center
```

对齐作用于整个文本块, 多行文本每行都按同一规则对齐. 完整示例(三种对齐 × 下划线/删除线)跑 `cargo run --example text_layout`. 

## 5.4 富文本 StyledText

一段文字里混不同样式(语法高亮、搜索命中、局部加粗), 不用拆成一堆 span, 用 `StyledText` 给区间套样式: 

```rust
StyledText::new("ABCD").with_highlights([
    (0..1, FontWeight::EXTRA_BOLD.into()), // "A"(第 0 字节起): 特粗字重
    (2..3, FontStyle::Italic.into()),     // "C"(第 2 字节起): 斜体
])
```

`with_highlights()` 收一组 `(Range<usize>, HighlightStyle)`;  `FontWeight`/`FontStyle`/`Hsla`/`Rgba` 都能 `.into()` 变成 `HighlightStyle`, 也可以直接构造 `HighlightStyle` 一次改多样: 

```rust
HighlightStyle {
    color: Some(gpui::red()),          // 文字颜色
    background_color: Some(gpui::yellow()), // 底色
    font_weight: Some(FontWeight::BOLD),   // 字重
    fade_out: Some(0.2),                   // 淡化 20%
    ..Default::default()                // 其余字段还有 font_style/underline/strikethrough
}
```

**NOTE**: 区间是**UTF-8 字节偏移**(`Range<usize>`), 且必须落在字符边界上. ASCII 文本可以心算, 中文/emoji 要用 `s.char_indices()` 之类按字节算位置, 否则会 panic. 

其他构造方式: 

```rust
// 1. with_runs(): 手工给每段配 TextRun(所有 run 的字节数必须正好覆盖整个字符串)
let base = window.text_style();              // 当前继承到的文字样式
let bold = base.clone().highlight(FontWeight::BOLD); // 叠加一个高亮, 变成粗体样式
StyledText::new("Hello").with_runs(vec![
    base.to_run(2), // "He": 2 个字节, 用 base 样式
    bold.to_run(3), // "llo": 3 个字节, 用 bold 样式
])

// 2. with_default_highlights(): 按给定基准样式立即算好 run
//    (与 with_highlights 二选一: 后者延迟到布局时按继承样式解析)

// 3. with_font_family_overrides(): 按区间换字体族(行内代码常见)
//    "求和 sum()" 里每个汉字占 3 字节, "sum" 落在第 7~9 字节
StyledText::new("求和 sum()").with_font_family_overrides([
    (7..10, ".ZedMono".into()), // "sum" 这 3 个字节换成等宽字体
])
```

`HighlightStyle` 的字段(全部 `Option`, 只覆盖列出的维度): `color`/`font_weight`/`font_style`/`background_color`/`underline`/`strikethrough`/`fade_out`. 

💡 提示: 需要可点击的富文本(链接、协议勾选文本)用 `InteractiveText::new(id, styled_text)`, 配 `.on_click(ranges, listener)` 按区间响应点击. 搜索框命中标黄的现成组件是 `HighlightedLabel`(zed `ui` crate). 

## 5.5 底层文本布局 API

对于更复杂的文本渲染需求,GPUI提供了底层的文本布局 API. 这些底层 API 通常在编辑器、终端模拟器等需要精确控制字形位置的场景中使用, 一般应用场景用字符串子元素 + 文字样式即可满足需求. 

```rust
/// TextRun: 一段连续的、样式相同的文本(按 UTF-8 字节数切分)
pub struct TextRun {
    /// 这段文本的 UTF-8 字节数(不含文本内容本身)
    pub len: usize,
    /// 字体(族 + 字重 + 样式 + OpenType 特性)
    pub font: Font,
    /// 颜色
    pub color: Hsla,
    /// 文字底色
    pub background_color: Option<Hsla>,
    /// 下划线
    pub underline: Option<UnderlineStyle>,
    /// 删除线
    pub strikethrough: Option<StrikethroughStyle>,
}
```

`TextRun` 通常不用手拼, 由 `TextStyle::to_run(len)` 生成(见 4.4). 

```rust
/// ShapedLine: 已完成字形塑形(连字、字距调整等)的一行文本, 可直接绘制
pub struct ShapedLine { /* 排版结果 + 原始文本, text 是 pub 字段, 排版数据经 Deref 读 LineLayout */ }

impl ShapedLine {
    pub fn width(&self) -> Pixels;   // 行宽
    pub fn split_at(&self, byte_index: usize) -> (ShapedLine, ShapedLine); // 按字节切成两段
    pub fn paint(&self, origin: Point<Pixels>, line_height: Pixels,
                 align: TextAlign, align_width: Option<Pixels>,
                 window: &mut Window, cx: &mut App) -> anyhow::Result<()>; // 绘制
}
```

通过窗口的文本系统做塑形: 

```rust
// 单行塑形(文本里不能带 \n), 第 4 个参数是强制行宽, 一般传 None
let line = window.text_system().shape_line(text, px(16.0), &runs, None);
line.paint(point(px(10.0), px(20.0)), px(24.0), TextAlign::Left, None, window, cx)?;

// 多行塑形: 第 4 个参数是换行宽度, 第 5 个是最大行数(都可选)
let lines = window.text_system().shape_text(text, px(16.0), &runs, Some(px(200.0)), Some(3))?;
```

`shape_text` 返回每个软换行后的 `WrappedLine`, 同样有 `paint()`/`paint_background()` 可用. 渲染出来的文本元素还能拿到 `TextLayout`, 用 `index_for_position()`/`position_for_index()` 在"鼠标坐标 ↔ 字符下标"之间换算(编辑器选区、光标定位用). 

## 5.6 字体

### 5.6.1 指定字体族

```rust
div()
    .font_family(".ZedMono")       // 字体族：font-family: ".ZedMono"
    .font_weight(FontWeight::BOLD) // 字重：font-weight: 700
```

字体族名字有两类: 以 `.` 开头的是GPUI内置字体(`.SystemUIFont` 平台 UI 字体, 是默认值; `.ZedMono` 等宽字体), 其余是系统安装的字体(`"Menlo"`、`"Courier New"` 等). `cx.text_system().all_font_names()` 列出当前全部可用的字体名. 

### 5.6.2 Font 整体设置

`Font` 把族名、字重、样式、OpenType 特性、回退链打包在一起, 适合一次换整套: 

```rust
let f = font("Menlo")   // 以族名构造 Font
    .bold()             // 字重 700
    .italic();          // 斜体
div().font(f)           // 一次设置 family + weight + style + features + fallbacks
```

单项设置用 `.font_family()`/`.font_weight()`/`.italic()`, 和 `.font()` 是并列入口. 

### 5.6.3 OpenType 特性与字体回退

```rust
// 关闭连字/上下文替换(calt), 代码字体常用: font-variant-ligatures: none
div().font_features(FontFeatures::disable_ligatures())
```

`font_features()` 接受任意 OpenType 特性标签到值的映射. 字体回退链没有独立的链式方法, 要通过 `Font` 的 `fallbacks` 字段设置(`FontFallbacks::from_fonts(vec![...])`), 再用 `.font()` 一次设入, 当首选字体缺字时按序找替代. 

### 5.6.4 加载自定义字体

GPUI 允许通过文本系统加载自定义字体, 注意收的是**字体文件的字节**, 不是路径: 

```rust
use std::borrow::Cow;

fn load_custom_fonts(cx: &mut App) {
    // 把 ttf 在编译期嵌进二进制, 再以字节形式注册给文本系统
    let fonts = [include_bytes!("../assets/fonts/lilex/Lilex-Regular.ttf")]
        .iter()
        .map(|b| Cow::Borrowed(&b[..]))
        .collect();

    cx.text_system().add_fonts(fonts).unwrap();
    // 之后 font_family() 里就能用这个字体的族名了
}
```

**NOTE**: `add_fonts()` 的签名是 `add_fonts(Vec<Cow<'static, [u8]>>) -> Result<()>`, 只接受字体文件的字节内容;  从磁盘加载要自己 `std::fs::read` 再转 `Cow::Owned`. 当前版本**没有** `FontSource` 类型, 其他教程里 `FontSource::File(PathBuf)` 的写法编译不过. 

💡 提示:  完整的字体示例见 `cargo run --example text` —— 启动时嵌入 Lilex 字体, 点顶部按钮在 `.ZedMono`/`.SystemUIFont`/`Menlo`/`Monaco`/`Courier New` 之间切换, 同时展示多语言字形覆盖和模块化字号阶梯. 

## 5.7 字符网格

字符网格在网格中渲染单个字符, 常用于字体预览和调试: 

```rust
use gpui::*;

/// CharacterGrid 在网格中展示字符渲染效果
pub struct CharacterGrid {
    /// 要展示的字符列表
    characters: Vec<char>,
    /// 单元格大小
    cell_size: Pixels,
    /// 字体族
    font_family: SharedString,
}

/// 无状态展示组件, 用 RenderOnce 一次性消费掉自己即可
impl RenderOnce for CharacterGrid {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
        .flex()       // 弹性布局：display: flex
        .flex_wrap()  // 允许换行：flex-wrap: wrap
        .gap(px(4.0)) // 单元格间距：gap: 4px
        .children(self.characters.iter().map(|&ch| {
            div()
            .size(self.cell_size)          // 单元格大小：width/height: <cell_size>
            .border(px(1.0))               // 边框：border-width: 1px
            .border_color(rgb(0xdddddd))   // 边框颜色：border-color: #dddddd
            .flex()                        // 弹性布局：display: flex
            .items_center()                // 交叉轴居中：align-items: center
            .justify_center()              // 主轴居中：justify-content: center
            .font_family(self.font_family.clone()) // 字体族：font-family: ...
            .child(ch.to_string())         // 单个字符作为文字子元素
        }))
    }
}
```

## 5.8 文字方法速查表

| 类别 | 方法 | 对应 CSS |
| :--- | :--- | :--- |
| **字号** | `.text_size()` `.text_xs()` ~ `.text_3xl()` | `font-size` |
| **颜色** | `.text_color()` `.text_bg()` | `color`, 文字底色 |
| **字重/字形** | `.font_weight()` `.italic()` `.not_italic()` | `font-weight`, `font-style` |
| **行高** | `.line_height(relative(1.5))` `.line_height(px(24.0))` | `line-height` |
| **字体** | `.font_family()` `.font()` `.font_features()` | `font-family`, `font-feature-settings` |
| **装饰** | `.underline()` `.line_through()` `.text_decoration_2()` `.text_decoration_color()` `.text_decoration_wavy()` | `text-decoration-*` |
| **对齐** | `.text_left()` `.text_center()` `.text_right()` | `text-align` |
| **溢出** | `.text_ellipsis()` `.text_ellipsis_start()` `.text_ellipsis_middle()` `.truncate()` `.line_clamp()` `.text_overflow()` | `text-overflow`, `-webkit-line-clamp` |
| **换行** | `.whitespace_nowrap()` `.whitespace_normal()` | `white-space` |
| **富文本** | `StyledText::new()` `.with_highlights()` `.with_runs()` `.with_font_family_overrides()` | (GPUI 特有) |
| **底层** | `text_system().shape_line()` `.shape_text()` `ShapedLine::paint()` | (GPUI 特有) |
| **字体加载** | `cx.text_system().add_fonts()` `.all_font_names()` | (GPUI 特有) |
