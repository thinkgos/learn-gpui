//! 文字 / 排版示例：演示 GPUI 的字体切换、字号阶梯、行高、以及字符覆盖情况。
//!
//! 运行方式：`cargo run --example text`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面从上到下分几块：
//!   1. 顶部工具条：一个按钮，点击可循环切换字体族
//!   2. 多语言文本块：CJK / 假名 / 谚文 / Emoji 的渲染效果
//!   3. 字符网格：把常用字符逐个排成格子，方便检查某个字体覆盖了哪些字形
//!   4. 字号阶梯（Specimen）：同一句话按模数化字号（type scale）逐级放大/缩小，
//!      并配一份反色版本，用来检查对比度
//!   5. 右侧一条 240px 的色带，用来看容器配色
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。
//! 但这个文件里更多是 GPUI 特有的概念（全局状态、RenderOnce 等），
//! 那些地方就直接用中文解释清楚。

#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）
#[path = "example_support/fonts.rs"]
mod example_support;

use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use gpui::{
    AbsoluteLength, App, Context, DefiniteLength, ElementId, Global, Hsla, Menu, SharedString,
    TextStyle, TitlebarOptions, Window, WindowBounds, WindowOptions, bounds, colors::DefaultColors,
    div, point, prelude::*, px, relative, rgb, size,
};
use gpui_platform::application;
use std::iter;

// ===========================================================================
// 一、全局排版设置 TextContext
//    整个应用共用的一套字号 / 行高 / 字号比例，类似 CSS 里的设计令牌（design token）
// ===========================================================================

/// 一组全局排版参数。
#[derive(Clone, Debug)]
pub struct TextContext {
    /// 基础字号（像素）。对应 font-size: 16px
    font_size: f32,
    /// 行高倍数。对应 line-height: 1.3（无单位，表示字号的倍数）
    line_height: f32,
    /// 字号阶梯比例（模数化比例，modular scale）。
    /// 每升一级字号就乘以它，比如 1.33 就是常见的“完全四度”比例。
    type_scale: f32,
}

impl Default for TextContext {
    fn default() -> Self {
        TextContext {
            font_size: 16.0,
            line_height: 1.3,
            type_scale: 1.33,
        }
    }
}

impl TextContext {
    /// 从应用上下文里取出全局排版设置（只读）。
    pub fn get_global(cx: &App) -> &Arc<TextContext> {
        &cx.global::<GlobalTextContext>().0
    }
}

/// 把 `TextContext` 包一层，好注册成 GPUI 的全局状态（Global）。
/// `Arc` 让多处共享同一份设置，不用反复克隆。
#[derive(Clone, Debug)]
pub struct GlobalTextContext(pub Arc<TextContext>);

/// 实现 `Deref`：让 `GlobalTextContext` 用起来就像 `Arc<TextContext>`，
/// 可以直接写 `global.font_size` 而不用 `global.0.font_size`。
impl Deref for GlobalTextContext {
    type Target = Arc<TextContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 同上，可变版本。
impl DerefMut for GlobalTextContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// 空实现，只为满足 `Global` trait 的标记：声明这个类型可以作为全局状态存进 `App`。
impl Global for GlobalTextContext {}

/// 定义一个便捷 trait，让 `App` 能直接写 `cx.text_context()`。
pub trait ActiveTextContext {
    fn text_context(&self) -> &Arc<TextContext>;
}

impl ActiveTextContext for App {
    fn text_context(&self) -> &Arc<TextContext> {
        &self.global::<GlobalTextContext>().0
    }
}

// ===========================================================================
// 二、样张主题 SpecimenTheme：只有一组前景 / 背景色
// ===========================================================================

/// 字符样张的配色：背景色 + 前景色。
/// 对应 background-color / color
#[derive(Clone, PartialEq)]
pub struct SpecimenTheme {
    pub bg: Hsla, // GPUI 的颜色类型（色相 / 饱和度 / 亮度 / 透明度）
    pub fg: Hsla,
}

impl Default for SpecimenTheme {
    fn default() -> Self {
        Self {
            bg: gpui::white(), // background-color: #ffffff
            fg: gpui::black(), // color: #000000
        }
    }
}

impl SpecimenTheme {
    /// 把前景色和背景色对调（做反色版本用）。
    pub fn invert(&self) -> Self {
        Self {
            bg: self.fg,
            fg: self.bg,
        }
    }
}

// ===========================================================================
// 三、Specimen：一条“文字样张”，展示一句话在某个字号下的效果
//    用 #[derive(IntoElement)] 代替手写 impl IntoElement
// ===========================================================================

#[derive(Debug, Clone, PartialEq, IntoElement)]
struct Specimen {
    /// 元素的唯一标识，GPUI 用它来跟踪元素状态（类似 DOM 里的 id）
    id: ElementId,
    /// 额外的缩放系数，作用在字号上
    scale: f32,
    /// 可选的样式覆盖：如果给了，就用它的字号 / 行高，而不是全局设置
    text_style: Option<TextStyle>,
    /// 要展示的文本内容
    string: SharedString,
    /// 是否使用反色主题
    invert: bool,
}

impl Specimen {
    /// 造一个样张：内容固定是一句经典的英文测试句。
    /// `id` 用来区分页面上的多个样张。
    pub fn new(id: usize) -> Self {
        // "The quick brown fox jumps over the lazy dog" 是包含了 26 个字母的句子，
        // 排版测试里最常用的一句话。
        let string = SharedString::new_static("The quick brown fox jumps over the lazy dog");
        let id_string = format!("specimen-{}", id);
        let id = ElementId::Name(id_string.into());
        Self {
            id,
            scale: 1.0,
            text_style: None,
            string,
            invert: false,
        }
    }

    /// 切换反色（builder 风格：返回 self，方便链式调用）
    pub fn invert(mut self) -> Self {
        self.invert = !self.invert;
        self
    }

    /// 设置缩放系数
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

/// `RenderOnce` 和之前 hello_world 里的 `Render` 不同：
///   - `Render`  用在“有自己状态、可以反复重绘”的组件上（`&mut self`）
///   - `RenderOnce` 用在“一次性消费掉自己”的元素上（`self`），更轻量，
///     适合这种纯粹把数据画出来的小组件。
impl RenderOnce for Specimen {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // rem 是排版里的相对单位，1rem = 当前根字号。
        // 这里取出来是为了把 rem 单位换算成像素。
        let rem_size = window.rem_size();
        let scale = self.scale;
        let global_style = cx.text_context();

        let style_override = self.text_style;

        // 默认用全局的字号 / 行高
        let mut font_size = global_style.font_size;
        let mut line_height = global_style.line_height;

        // 如果调用方给了样式覆盖，就换算成像素后用它的值
        if let Some(style_override) = style_override {
            font_size = style_override.font_size.to_pixels(rem_size).into();
            line_height = match style_override.line_height {
                // 绝对长度：可能是 rem 或 px，都换算成像素
                DefiniteLength::Absolute(absolute_len) => match absolute_len {
                    AbsoluteLength::Rems(absolute_len) => absolute_len.to_pixels(rem_size).into(),
                    AbsoluteLength::Pixels(absolute_len) => absolute_len.into(),
                },
                // 分数：就是行高的倍数，直接用
                DefiniteLength::Fraction(value) => value,
            };
        }

        // 选主题，需要反色就对调前景 / 背景
        let mut theme = SpecimenTheme::default();

        if self.invert {
            theme = theme.invert();
        }

        div()
            .id(self.id) // GPUI 的元素标识，不是 CSS
            .bg(theme.bg) // 背景色：background-color: ...
            .text_color(theme.fg) // 文字颜色：color: ...
            .text_size(px(font_size * scale)) // 字号（乘上缩放系数）：font-size: ...px
            .line_height(relative(line_height)) // 行高倍数：line-height: 1.3（无单位）
            .p(px(10.0)) // 内边距 10 像素：padding: 10px
            .child(self.string) // 塞进文字内容
    }
}

// ===========================================================================
// 四、CharacterGrid：把一堆字符排成网格，检查字体的字形覆盖情况
// ===========================================================================

#[derive(Debug, Clone, PartialEq, IntoElement)]
struct CharacterGrid {
    scale: f32,
    invert: bool,
    text_style: Option<TextStyle>,
}

impl CharacterGrid {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            invert: false,
            text_style: None,
        }
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

impl RenderOnce for CharacterGrid {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut theme = SpecimenTheme::default();

        if self.invert {
            theme = theme.invert();
        }

        // 要展示的字符集：数字、拉丁字母、希腊字母、西里尔字母、
        // 一些特殊符号（长 s、尖 s、冰岛字母等），最后几个是 emoji，
        // 用来验证字体能不能正确画出这些字形。
        let characters = vec![
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "A", "B", "C", "D", "E", "F", "G",
            "H", "I", "J", "K", "L", "M", "N", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y",
            "Z", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z", "ẞ", "ſ", "ß", "ð", "Þ", "þ", "α", "β",
            "Γ", "γ", "Δ", "δ", "η", "θ", "ι", "κ", "Λ", "λ", "μ", "ν", "ξ", "π", "τ", "υ", "φ",
            "χ", "ψ", "∂", "а", "в", "Ж", "ж", "З", "з", "К", "к", "л", "м", "Н", "н", "Р", "р",
            "У", "у", "ф", "ч", "ь", "ы", "Э", "э", "Я", "я", "ij", "öẋ", ".,", "⣝⣑", "~", "*",
            "_", "^", "`", "'", "(", "{", "«", "#", "&", "@", "$", "¢", "%", "|", "?", "¶", "µ",
            "❮", "<=", "!=", "==", "--", "++", "=>", "->", "🏀", "🎊", "😍", "❤️", "👍", "👎",
        ];

        // 每行 20 个格子；行数用向上取整除法算出来
        let columns = 20;
        let rows = characters.len().div_ceil(columns);

        // 把字符切成若干行
        let grid_rows = (0..rows).map(|row_idx| {
            let start_idx = row_idx * columns;
            let end_idx = (start_idx + columns).min(characters.len());

            div()
                .w_full() // 占满整行：width: 100%
                .flex() // 弹性布局：display: flex
                .flex_row() // 横向排列：flex-direction: row
                // 把这一行区间内的字符逐个渲染成小格子
                .children((start_idx..end_idx).map(|i| {
                    div()
                        .text_center() // 文字居中：text-align: center
                        .size(px(62.)) // 62x62 的方格：width: 62px; height: 62px
                        .bg(theme.bg) // 背景色：background-color: ...
                        .text_color(theme.fg) // 文字颜色：color: ...
                        .text_size(px(24.0)) // 字号：font-size: 24px
                        .line_height(relative(1.0)) // 行高等于字号：line-height: 1
                        .child(characters[i])
                }))
                // 最后一行往往填不满，补上空白格把布局撑齐（视觉上对齐网格）
                .when(end_idx - start_idx < columns, |d| {
                    d.children(
                        iter::repeat_with(|| div().flex_1()).take(columns - (end_idx - start_idx)),
                    )
                })
        });

        // 整个网格容器：内边距 + 行间距 + 纵向排列
        div().p_4().gap_2().flex().flex_col().children(grid_rows)
        // p_4():    padding: 1rem (16px)
        // gap_2():  gap: 0.5rem (8px)
        // flex():   display: flex
        // flex_col(): flex-direction: column
    }
}

// ===========================================================================
// 五、TextExample：整个示例的根组件（有状态，所以用 Render 而不是 RenderOnce）
// ===========================================================================

struct TextExample {
    /// 已经用掉的样张编号，每次创建新样张就自增，保证 id 唯一
    next_id: usize,
    /// 当前选中的字体族
    font_family: SharedString,
}

impl TextExample {
    /// 取下一个样张编号（自增）
    fn next_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    /// 造一个按钮：黑色底、白色字，按下时半透明。
    /// `on_click` 是点击时要执行的逻辑。
    fn button(
        text: &str,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .id(text.to_string()) // 元素标识，on_click 需要它来定位元素
            .flex_none() // 不参与弹性伸缩：flex: none
            .child(text.to_string()) // 按钮上的文字
            .bg(gpui::black()) // 背景色：background-color: #000000
            .text_color(gpui::white()) // 文字颜色：color: #ffffff
            .active(|this| this.opacity(0.8)) // 鼠标按下时半透明：:active { opacity: 0.8 }
            .px_3() // 左右内边距：padding-left/right: 0.75rem
            .py_1() // 上下内边距：padding-top/bottom: 0.25rem
            .on_click(cx.listener(move |this, _, _, cx| on_click(this, cx))) // 点击回调
    }
}

/// 可切换的字体族列表。以 `.` 开头的是 GPUI 内部字体名，
/// 其余是系统字体名（找不到就会回退到默认字体）。
const FONT_FAMILIES: [&str; 5] = [
    ".ZedMono",
    ".SystemUIFont",
    "Menlo",
    "Monaco",
    "Courier New",
];

impl Render for TextExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tcx = cx.text_context();
        // 应用默认配色（容器色等），下面右侧色带会用到
        let colors = cx.default_colors().clone();

        let font_size = tcx.font_size;
        let line_height = tcx.line_height;
        let type_scale = tcx.type_scale;

        // 用 type_scale 算出字号阶梯：往上每级乘 1.33，往下每级除 1.33。
        // 命名参考排版里的 "step up / step down"（升阶 / 降阶）。
        let step_down_2 = 1.0 / (type_scale * type_scale);
        let step_down_1 = 1.0 / type_scale;
        let base = 1.0;
        let step_up_1 = base * type_scale;
        let step_up_2 = step_up_1 * type_scale;
        let step_up_3 = step_up_2 * type_scale;
        let step_up_4 = step_up_3 * type_scale;
        let step_up_5 = step_up_4 * type_scale;
        let step_up_6 = step_up_5 * type_scale;

        div()
            .font_family(self.font_family.clone()) // 全局字体族：font-family: ".ZedMono"
            .size_full() // 铺满整个窗口：width: 100%; height: 100%
            // ---------- 顶部工具条 ----------
            .child(
                div()
                    .bg(gpui::white()) // 背景色：background-color: #ffffff
                    .border_b_1() // 底部 1 像素边框：border-bottom-width: 1px
                    .border_color(gpui::black()) // 边框颜色：border-color: #000000
                    .p_3() // 内边距：padding: 0.75rem (12px)
                    .flex() // 弹性布局：display: flex
                    // 一个按钮：显示当前字体名，点击切换到列表里的下一个字体
                    .child(Self::button(&self.font_family, cx, |this, cx| {
                        // 找到当前字体在列表中的位置，取下一个（取模绕回开头）
                        let new_family = FONT_FAMILIES
                            .iter()
                            .position(|f| *f == this.font_family.as_str())
                            .map(|idx| FONT_FAMILIES[(idx + 1) % FONT_FAMILIES.len()])
                            .unwrap_or(FONT_FAMILIES[0]);

                        this.font_family = SharedString::new(new_family);
                        // 通知 GPUI：状态变了，需要重新渲染
                        cx.notify();
                    })),
            )
            // ---------- 中间可滚动的主内容区 ----------
            .child(
                div()
                    .id("text-example")
                    .overflow_y_scroll() // 纵向可滚动：overflow-y: scroll
                    .overflow_x_hidden() // 横向裁掉：overflow-x: hidden
                    .bg(rgb(0xffffff)) // 背景色：background-color: #ffffff
                    .size_full() // 铺满剩余空间：width: 100%; height: 100%
                    .child(
                        div()
                            .flex() // 弹性布局：display: flex
                            .flex_col() // 纵向排列：flex-direction: column
                            .p_3() // 内边距：padding: 0.75rem
                            .text_size(px(font_size)) // 基础字号：font-size: 16px
                            .line_height(relative(line_height)) // 行高倍数：line-height: 1.3
                            // 三行多语言测试文本
                            .children([
                                // 中日韩文字
                                div().child("CJK: 中文「日本語」한글"),
                                // 假名 + 组合浊点、谚文组合字母
                                div().child(
                                    "Kana: が / か\u{3099}; Hangul: 각 / \u{1100}\u{1161}\u{11a8}",
                                ),
                                // Emoji：含肤色修饰符、国旗、组合 emoji（👩‍💻）
                                div().child("Emoji: 😀 ❤️ ❤\u{fe0e} 👍🏽 🇯🇵 1\u{fe0f}\u{20e3} 👩‍💻 👨‍👩‍👧‍👦"),
                            ]),
                    )
                    // 字符覆盖网格
                    .child(div().child(CharacterGrid::new().scale(base)))
                    // 字号阶梯样张：每一档字号都来一份正常版 + 一份反色版
                    .child(
                        div()
                            .child(Specimen::new(self.next_id()).scale(step_down_2))
                            .child(Specimen::new(self.next_id()).scale(step_down_2).invert())
                            .child(Specimen::new(self.next_id()).scale(step_down_1))
                            .child(Specimen::new(self.next_id()).scale(step_down_1).invert())
                            .child(Specimen::new(self.next_id()).scale(base))
                            .child(Specimen::new(self.next_id()).scale(base).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_1))
                            .child(Specimen::new(self.next_id()).scale(step_up_1).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_2))
                            .child(Specimen::new(self.next_id()).scale(step_up_2).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_3))
                            .child(Specimen::new(self.next_id()).scale(step_up_3).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_4))
                            .child(Specimen::new(self.next_id()).scale(step_up_4).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_5))
                            .child(Specimen::new(self.next_id()).scale(step_up_5).invert())
                            .child(Specimen::new(self.next_id()).scale(step_up_6))
                            .child(Specimen::new(self.next_id()).scale(step_up_6).invert()),
                    ),
            )
            // ---------- 右侧 240px 的色带 ----------
            .child(div().w(px(240.)).h_full().bg(colors.container))
        // w(px(240.)): 宽 240px —— width: 240px
        // h_full():    铺满高度 —— height: 100%
        // bg(...):     背景色  —— background-color: ...
    }
}

// ===========================================================================
// 六、启动入口
// ===========================================================================

fn run_example() {
    application().run(|cx: &mut App| {
        // 加载示例所需的字体；失败就直接退出
        if !example_support::load_fonts(cx) {
            return;
        }

        // 设置应用菜单栏（这里只放一个空菜单，标题叫 "GPUI Typography"）
        cx.set_menus(vec![Menu {
            name: "GPUI Typography".into(),
            disabled: false,
            items: vec![],
        }]);

        // 额外加载一个 Lilex 字体（等宽字体），编译期直接把 ttf 文件嵌进二进制
        let fonts = [include_bytes!("../assets/fonts/lilex/Lilex-Regular.ttf")]
            .iter()
            .map(|b| Cow::Borrowed(&b[..]))
            .collect();

        _ = cx.text_system().add_fonts(fonts);

        // 初始化默认配色（后面 cx.default_colors() 才有值）
        cx.init_colors();
        // 把排版设置注册成全局状态，供所有组件读取
        cx.set_global(GlobalTextContext(Arc::new(TextContext::default())));

        // 打开窗口：标题栏 + 920x720 的固定尺寸
        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("GPUI Typography".into()),
                        ..Default::default()
                    }),
                    // 窗口左上角在 (0, 0)，尺寸 920x720
                    window_bounds: Some(WindowBounds::Windowed(bounds(
                        point(px(0.0), px(0.0)),
                        size(px(920.), px(720.)),
                    ))),
                    ..Default::default()
                },
                // 窗口根组件
                |_window, cx| {
                    cx.new(|_cx| TextExample {
                        next_id: 0,
                        font_family: ".ZedMono".into(), // 初始字体
                    })
                },
            )
            .unwrap();

        // 把窗口提到前台，抢到输入焦点
        window
            .update(cx, |_view, _window, cx| {
                cx.activate(true);
            })
            .unwrap();
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main
#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，页面加载完成后自动调用
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
