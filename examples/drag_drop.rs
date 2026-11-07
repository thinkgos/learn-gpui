//! 拖放示例：把顶部三个彩色方块（Item）拖到底部 "Drop items here" 区域里放下，
//! 演示 `.on_drag()`（拖拽源）、`.on_drop()`（放置目标）和跟随光标的拖拽预览。
//!
//! 运行方式：`cargo run --example drag_drop`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 标题 "Drop & Drop"；
//!   2. 一排三个 128px 的彩色方块（Item 0/1/2），每个都是可拖拽源；
//!   3. 一个 512x128 的放置框，放下后边框和底色变成放进来那项的颜色。
//!
//! 三个关键点（详见第 11 章）：
//!   1. `DragInfo` 一物两用：既是 `.on_drag()` 携带的拖拽数据，
//!      又实现 `Render` 当跟随光标的拖拽预览（GPUI 没有内置的拖拽数据类型）；
//!   2. `.on_drag(value, 构造器)` 两部分都要给：拖拽数据 + 预览实体的构造闭包；
//!   3. `.on_drop(listener)` 的回调按类型自动匹配：只有拖的是 `DragInfo` 才触发。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, Half, Hsla, Pixels, Point, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 拖拽数据 + 拖拽预览，一物两用（示例自己定义的类型，见第 11 章）：
///   - 当数据：记录拖的是第几项、什么颜色；
///   - 当预览：实现 `Render`，拖拽期间跟随光标绘制。
#[derive(Clone, Copy)]
struct DragInfo {
    /// 第几项（0/1/2），只用来显示 "Item N"。
    ix: usize,
    /// 项目颜色：预览色块和放下后的反馈都用它。
    color: Hsla,
    /// 按下点相对原元素左上角的偏移，拖拽开始时由构造器存进来。
    position: Point<Pixels>,
}

impl DragInfo {
    /// 构造拖拽数据，position 先置零，拖拽开始时再由构造器填真值。
    fn new(ix: usize, color: Hsla) -> Self {
        Self {
            ix,
            color,
            position: Point::default(),
        }
    }

    /// 链式设置按下点偏移（拿走所有权的 builder 写法），在预览构造器里用。
    fn position(mut self, pos: Point<Pixels>) -> Self {
        self.position = pos;
        self
    }
}

/// 拖拽预览的样子：120x50 的半透明色块，跟着光标走。
impl Render for DragInfo {
    fn render(&mut self, _: &mut Window, _: &mut Context<'_, Self>) -> impl IntoElement {
        let size = gpui::size(px(120.), px(50.)); // 预览色块的尺寸：120x50 像素

        // 外层 div 不画东西，只用 padding 把预览挪位置：
        // GPUI 画预览时会先减掉按下点偏移（让预览停在当初抓取的位置），
        // 这里再加上「按下点偏移 - 预览半宽/半高」的 padding，两者抵消后，
        // 预览框正好以光标为中心。
        div()
            .pl(self.position.x - size.width.half()) // padding-left: <偏移 x - 60px>
            .pt(self.position.y - size.height.half()) // padding-top: <偏移 y - 25px>
            .child(
                div() // 真正可见的预览色块
                    .flex() // 弹性布局：display: flex
                    .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                    .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                    .w(size.width) // 宽度：width: 120px
                    .h(size.height) // 高度：height: 50px
                    .bg(self.color.opacity(0.5)) // 50% 不透明度的项目色：background-color: <颜色> / 0.5
                    .text_color(gpui::white()) // 文字颜色：color: #ffffff
                    .text_xs() // 字号缩小：font-size: 0.75rem; line-height: 1rem
                    .shadow_md() // 中等阴影：box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)
                    .child(format!("Item {}", self.ix)),
            )
    }
}

/// 示例组件：记录最近一次放下的是哪一项。
struct DragDrop {
    /// 放置框里最后收到的拖拽数据，None 表示还没放过东西。
    drop_on: Option<DragInfo>,
}

impl DragDrop {
    /// 初始状态：放置框还是空的。
    fn new() -> Self {
        Self { drop_on: None }
    }
}

/// 实现 `Render`：顶部一排拖拽源，底部一个放置目标。
impl Render for DragDrop {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = [gpui::blue(), gpui::red(), gpui::green()]; // 三个方块的颜色

        div()
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .gap_5() // 子元素之间的间距：gap: 1.25rem (20px)
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .text_color(rgb(0x333333)) // 文字颜色：color: #333333
            .child(div().text_xl().text_center().child("Drop & Drop")) // 标题（字号加大、居中）
            .child(
                div() // 一排三个可拖拽的方块
                    .w_full() // 宽度撑满：width: 100%
                    .mb_10() // 下外边距：margin-bottom: 2.5rem (40px)
                    .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                    .flex() // 弹性布局：display: flex
                    .flex_row() // 横向排列：flex-direction: row
                    .gap_4() // 方块之间的间距：gap: 1rem (16px)
                    .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                    .children(items.into_iter().enumerate().map(|(ix, color)| {
                        // 每个方块一份拖拽数据：拖的是第几项、什么颜色
                        let drag_info = DragInfo::new(ix, color);

                        div()
                            .id(("item", ix)) // 唯一 id：on_drag 只在带 id 的元素上可用
                            .size_32() // 32 号尺寸的正方形：width: 8rem; height: 8rem (128px)
                            .flex() // 弹性布局：display: flex
                            .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                            .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                            .border_2() // 2 像素边框：border-width: 2px
                            .border_color(color) // 边框颜色：border-color: <项目色>
                            .text_color(color) // 文字颜色：color: <项目色>
                            .cursor_move() // 拖动光标：cursor: move
                            .hover(|this| this.bg(color.opacity(0.2))) // 悬停时铺一层 20% 项目色底：类似 :hover 的样式回调
                            .child(format!("Item ({})", ix))
                            // .on_drag(拖拽数据, 预览构造器)：按住左键移动超过 2px 就自动开始拖拽
                            .on_drag(drag_info, |info: &DragInfo, position, _, cx| {
                                // 构造器收 4 个参数：拖拽数据、按下点相对元素原点的偏移、window、cx
                                cx.new(|_| info.position(position)) // 造预览实体，跟随光标绘制
                            })
                    })),
            )
            .child(
                div() // 放置目标
                    .id("drop-target")
                    .w_128() // 宽度：width: 32rem (512px)
                    .h_32() // 高度：height: 8rem (128px)
                    .flex() // 弹性布局：display: flex
                    .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                    .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                    .border_3() // 3 像素边框：border-width: 3px
                    .border_color(self.drop_on.map(|info| info.color).unwrap_or(gpui::black())) // 放下过用项目色，否则黑色：border-color: ...
                    .when_some(self.drop_on, |this, info| this.bg(info.color.opacity(0.5))) // 条件渲染：放下过就铺一层 50% 项目色底
                    .on_drop(cx.listener(|this, info: &DragInfo, _, _| {
                        // 回调按类型自动匹配：只有拖的是 DragInfo 才触发，直接拿到 &DragInfo
                        this.drop_on = Some(*info); // 记下放进来的数据，驱动边框 / 底色变化
                    }))
                    .child("Drop items here"),
            )
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 计算窗口位置：让 800x600 的窗口出现在屏幕中央。
        // None 表示不指定显示在哪个显示器上，用默认的。
        let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default()                                 // 其余选项都用默认值
            },
            |_, cx| cx.new(|_| DragDrop::new()),
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        // 让这个窗口成为活动窗口（抢到输入焦点）。
        cx.activate(true);
    });
}

// ---------- 入口函数 ----------
// 桌面平台：普通的 main。
#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

// wasm 平台：由 `#[wasm_bindgen(start)]` 标记，在页面加载完成后自动调用。
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
