//! 文字换行 / 截断示例：演示 GPUI 里文字放不下时的各种处理——
//! 单行 / 多行的省略号截断、硬截断（不加省略号）、禁止换行直接裁剪、正常换行。
//!
//! 运行方式：`cargo run --example text_wrapper`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面从上到下分七块：
//!   1. 省略号 × flex：一行里放几个不同宽度的盒子，对比截断表现
//!   2. truncate()：单行截断 + 省略号（Tailwind 风格的三合一快捷方式）
//!   3. line_clamp(2)：最多 2 行，超出截断 + 省略号
//!   4. text_overflow(Truncate(""))：单行硬截断（截断标记是空字符串，不加省略号）
//!   5. 同上 + line_clamp(3)：最多 3 行，超出硬截断
//!   6. whitespace_nowrap + overflow_hidden：不换行，超出直接裁掉
//!   7. 正常换行：长文本自动折行
//!
//! 演示文本故意混了超长英文单词、中日文混排和一个带参数的长 URL，
//! 方便观察断行 / 截断在各种边界上的表现。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, TextOverflow, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例组件（结构体名字沿用上游的 HelloWorld，其实和 Hello World 无关）：
/// 界面就是一列「文字放不下怎么办」的对比演示。
struct HelloWorld {}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 所有演示块共用的长文本，内容故意混了几种难处理的东西：
        //   - 45 个字母的超长英文单词（没法在词中间断）
        //   - 中文 / 日文混排（CJK 逐字可断行）
        //   - 一个带查询参数的长 URL（没有空格，全靠 URL 内部断点）
        // 字符串行尾的 `\` 是 Rust 的续行符：换行和下一行的缩进空白都会被吃掉。
        let text = "The longest word 你好世界这段是中文，こんにちはこの段落は日本語です in any of the major \
            English language dictionaries is pneumonoultramicroscopicsilicovolcanoconiosis, a word that \
            refers to a lung disease contracted from the inhalation of very fine silica particles, \
            a url https://github.com/zed-industries/zed/pull/35724?query=foo&bar=2, \
            specifically from a volcano; medically, it is the same as silicosis.";
        // 最外层容器，一列演示块从上往下排：
        // id("page"):  给元素一个 ID，返回可挂交互的 Stateful 元素（GPUI 概念：
        //              类似 HTML 的 id 属性，但用途是元素身份 / 交互追踪，
        //              这里只演示写法，没有挂任何交互）
        // size_full(): 撑满窗口：width: 100%; height: 100%
        // flex():      弹性布局：display: flex
        // flex_col():  纵向排列：flex-direction: column
        // p_2():       内边距：padding: 0.5rem (8px)
        // gap_2():     演示块之间的间距：gap: 0.5rem (8px)
        // bg(...):     背景色：background-color: #ffffff
        div()
            .id("page")
            .size_full()
            .flex()
            .flex_col()
            .p_2()
            .gap_2()
            .bg(gpui::white())
            // ---------- 一、省略号在 flex 布局里的表现 ----------
            // text_ellipsis(): 放不下时在末尾截断并补省略号：text-overflow: ellipsis
            // （GPUI 里它本身就带「截断成单行」的效果，不依赖 white-space: nowrap）
            .child(
                div()
                    .flex() // 弹性布局：display: flex
                    .flex_row() // 横向排列：flex-direction: row（默认就是 row，这里显式写出来）
                    .flex_shrink_0() // 不许被压矮（纵向 flex 里主轴是高度）：flex-shrink: 0
                    .gap_2() // 盒子之间的间距：gap: 0.5rem (8px)
                    .child(
                        div()
                            .flex() // 内容宽度的 flex 子项
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::red()) // 边框颜色：border-color: #ff0000
                            .text_ellipsis() // 省略号截断：text-overflow: ellipsis
                            .child("longer text in flex 1"),
                    )
                    .child(
                        div()
                            .flex() // 同上，只是文字更短
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::red()) // 边框颜色：border-color: #ff0000
                            .text_ellipsis() // 省略号截断：同上
                            .child("short flex"),
                    )
                    .child(
                        div()
                            .overflow_hidden() // 超出部分裁掉不显示：overflow: hidden
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::red()) // 边框颜色：border-color: #ff0000
                            .text_ellipsis() // 省略号截断：同上
                            .w_full() // 宽度撑满父容器：width: 100%
                            .child("A short text in normal div"),
                        // 和前两个对比：前两个是内容宽度的 flex 子项，
                        // 这个是普通 div、显式指定 width: 100%
                    ),
            )
            // ---------- 二、truncate()：单行截断 + 省略号 ----------
            // truncate() 是三合一快捷方式，等价于依次调用：
            //   overflow_hidden() + whitespace_nowrap() + text_ellipsis()
            .child(
                div()
                    .flex_shrink_0() // 不许被压矮：flex-shrink: 0
                    .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                    .truncate() // 单行截断 + 省略号（见上面的说明）
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_color(gpui::blue()) // 边框颜色：border-color: #0000ff
                    .child("ELLIPSIS: ".to_owned() + text),
            )
            // ---------- 三、多行省略号：line_clamp(2) ----------
            // line_clamp(n): 最多显示 n 行，超出部分截断（自动带上 overflow: hidden）
            // 配合 text_ellipsis()，截断处会补省略号
            .child(
                div()
                    .flex_shrink_0() // 不许被压矮：flex-shrink: 0
                    .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                    .overflow_hidden() // 超出部分裁掉：overflow: hidden（line_clamp 也会自动加，写了不冲突）
                    .text_ellipsis() // 截断处补省略号：text-overflow: ellipsis
                    .line_clamp(2) // 最多 2 行，超出截断：-webkit-line-clamp: 2
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_color(gpui::blue()) // 边框颜色：border-color: #0000ff
                    .child("ELLIPSIS 2 lines: ".to_owned() + text),
            )
            // ---------- 四、硬截断：单行、不加省略号 ----------
            // text_overflow(TextOverflow::Truncate(s))：放不下时在末尾截断，
            // 并把字符串 s 显示在截断处作为标记。这里 s 是空字符串 ""，
            // 表示直接裁掉、不显示任何标记（对比二的省略号）。
            .child(
                div()
                    .flex_shrink_0() // 不许被压矮：flex-shrink: 0
                    .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                    .overflow_hidden() // 超出部分裁掉：overflow: hidden
                    .text_overflow(TextOverflow::Truncate("".into())) // 末尾截断，标记为空字符串
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_color(gpui::green()) // 边框颜色：border-color: #008000
                    .child("TRUNCATE: ".to_owned() + text),
            )
            // ---------- 五、硬截断 + 限行：最多 3 行 ----------
            .child(
                div()
                    .flex_shrink_0() // 不许被压矮：flex-shrink: 0
                    .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                    .overflow_hidden() // 超出部分裁掉：overflow: hidden
                    .text_overflow(TextOverflow::Truncate("".into())) // 末尾截断，标记为空字符串
                    .line_clamp(3) // 最多 3 行，超出截断
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_color(gpui::green()) // 边框颜色：border-color: #008000
                    .child("TRUNCATE 3 lines: ".to_owned() + text),
            )
            // ---------- 六、禁止换行直接裁掉 ----------
            // 不设置 text_overflow（没有截断标记），文字也不换行：超出部分硬裁
            .child(
                div()
                    .flex_shrink_0() // 不许被压矮：flex-shrink: 0
                    .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                    .whitespace_nowrap() // 禁止换行：white-space: nowrap
                    .overflow_hidden() // 超出部分裁掉不显示：overflow: hidden
                    .border_1() // 1 像素边框：border-width: 1px
                    .border_color(gpui::black()) // 边框颜色：border-color: #000000
                    .child("NOWRAP: ".to_owned() + text),
            )
            // ---------- 七、正常换行 ----------
            // 什么都不加：文字按容器宽度自动折行。
            // 可以观察超长单词、中日文、URL 都在哪里断开。
            .child(div().text_xl().w_full().child(text))
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）
        if !example_support::load_fonts(cx) {
            return;
        }

        // 计算窗口位置：让 800x600 的窗口出现在屏幕中央
        // None 表示不指定显示在哪个显示器上，用默认的
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        // 打开一个新窗口
        cx.open_window(
            WindowOptions {
                // 窗口使用我们上面算出来的位置和尺寸（而不是最大化 / 全屏）
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // 其余选项都用默认值
                ..Default::default()
            },
            // 窗口的内容：创建一个 HelloWorld 实例作为根组件
            |_, cx| cx.new(|_| HelloWorld {}),
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        // 让这个窗口成为活动窗口（抢到输入焦点）
        cx.activate(true);
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main
#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
