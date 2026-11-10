//! 锚定定位 Anchor 示例：3×3 九宫格按钮，鼠标悬停时用 `deferred()` + `anchored()`
//! 在按钮外侧弹出悬浮提示，演示 `Anchor` 8 个变体的钉住效果。
//!
//! 运行方式：`cargo run --example anchor`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 顶部一行标题 "Popover with Anchor"；
//!   2. 中间 3×3 九宫格按钮（每格 120×65 像素）：悬停某一格，
//!      就在按钮外侧弹出悬浮提示；正中间一格是空白占位，不响应悬停。
//!
//! 关键概念（详见第 16 章）：
//!   1. `Anchor` 的 8 个变体：悬浮提示用自己被钉住的那个角贴住参考点，
//!      参考点取按钮相反一侧的角点/边中点，所以提示永远挂在按钮外侧；
//!   2. `resolved_position()`：算参考点坐标（`position_mode(Local)` 相对父元素）；
//!   3. `snap_to_window()`：提示超出窗口时贴边收回，保证不出窗口。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    Anchor, AnchoredPositionMode, App, Axis, Bounds, Context, Half as _, InteractiveElement,
    ParentElement, Pixels, Point, Render, SharedString, Size, Window, WindowBounds, WindowOptions,
    anchored, deferred, div, point, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、组件状态：九宫格数据 + 悬停记录
// ===========================================================================

/// 示例组件：只记录「当前悬停的是哪一格」，用它控制悬浮提示的显隐。
struct AnchorDemo {
    /// 当前悬停格子的序号（0..8）；`None` 表示没有格子被悬停。
    hovered_button: Option<usize>,
}

/// 九宫格里的一格：按钮文字 + 这一格演示的锚点变体。
struct ButtonDemo {
    /// 按钮文字（同时用作悬浮提示的文字）。
    label: SharedString,
    /// 这一格演示的锚点变体；正中间一格是 `None`，不响应悬停。
    corner: Option<Anchor>,
}

// ===========================================================================
// 二、参考点坐标计算
// ===========================================================================

/// 算出悬浮提示的参考点坐标 —— 提示的 `corner` 角会贴在这个点上。
///
/// `AnchoredPositionMode::Local` 的坐标相对父元素（外层 deferred 包装盒）量起，
/// 而它在文档流里排在按钮后面，所以原点在按钮正下方。
/// 代码先减一个按钮高度把原点拉回按钮左上角，再查表取「按钮包围盒」上的点：
/// 参考点落在按钮「与锚点相反一侧」的角点/边中点，悬浮提示因此挂在按钮外侧。
fn resolved_position(corner: Anchor, button_size: Size<Pixels>) -> Point<Pixels> {
    // 把原点从按钮正下方拉回按钮左上角：整体上移一个按钮高度。
    let offset = Point {
        x: px(0.),
        y: -button_size.height,
    };

    offset
        // 先把锚点上下翻转（TopLeft ↔ BottomLeft），再去表里查按钮包围盒上的点，
        // 查到的点因此落在锚点的「相反一侧」：提示挂在按钮外侧，不会挡住按钮。
        // 表里坐标原点是按钮左上角（上面的 offset 已把原点拉回来）。
        + match corner.other_side_along(Axis::Vertical) {
            Anchor::TopLeft => point(px(0.0), px(0.0)), // 按钮左上角
            Anchor::TopCenter => point(button_size.width.half(), px(0.0)), // 按钮上边中点
            Anchor::TopRight => point(button_size.width, px(0.0)), // 按钮右上角
            Anchor::LeftCenter => point(button_size.width, button_size.height.half()), // 按钮右边中点
            Anchor::RightCenter => point(px(0.), button_size.height.half()), // 按钮左边中点
            Anchor::BottomLeft => point(px(0.0), button_size.height), // 按钮左下角
            Anchor::BottomCenter => point(button_size.width / 2.0, button_size.height), // 按钮下边中点
            Anchor::BottomRight => point(button_size.width, button_size.height), // 按钮右下角
        }
}

// ===========================================================================
// 三、界面渲染：3×3 九宫格 + 悬停时的悬浮提示
// ===========================================================================

impl AnchorDemo {
    /// 九宫格的 9 格数据：8 个锚点变体各占一格，正中间一格只占位。
    fn buttons() -> Vec<ButtonDemo> {
        vec![
            ButtonDemo {
                label: "TopLeft".into(),
                corner: Some(Anchor::TopLeft),
            },
            ButtonDemo {
                label: "TopCenter".into(),
                corner: Some(Anchor::TopCenter),
            },
            ButtonDemo {
                label: "TopRight".into(),
                corner: Some(Anchor::TopRight),
            },
            ButtonDemo {
                label: "LeftCenter".into(),
                corner: Some(Anchor::LeftCenter),
            },
            ButtonDemo {
                label: "Center".into(),
                corner: None, // 正中间一格：不演示锚点，不可悬停
            },
            ButtonDemo {
                label: "RightCenter".into(),
                corner: Some(Anchor::RightCenter),
            },
            ButtonDemo {
                label: "BottomLeft".into(),
                corner: Some(Anchor::BottomLeft),
            },
            ButtonDemo {
                label: "BottomCenter".into(),
                corner: Some(Anchor::BottomCenter),
            },
            ButtonDemo {
                label: "BottomRight".into(),
                corner: Some(Anchor::BottomRight),
            },
        ]
    }
}

/// 实现 `Render`：画出九宫格按钮，悬停的那一格在按钮外侧挂悬浮提示。
impl Render for AnchorDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let buttons = Self::buttons(); // 9 格数据
        let button_size = size(px(120.0), px(65.0)); // 每个按钮的尺寸：width: 120px; height: 65px

        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .gap_4() // 子元素之间的间距：gap: 1rem (16px)
            .p_10() // 四周内边距：padding: 2.5rem (40px)
            .child("Popover with Anchor") // 标题文字
            .child(
                div() // 3×3 九宫格容器
                    .size_128() // 32 号正方形：width: 32rem; height: 32rem (512px)
                    .grid() // 网格布局：display: grid
                    .grid_cols(3) // 3 列：grid-template-columns: repeat(3, minmax(0, 1fr))
                    .gap_6() // 格子之间的间距：gap: 1.5rem (24px)
                    .relative() // 定位上下文：position: relative
                    .children(buttons.iter().enumerate().map(|(index, button)| {
                        // 把 9 格数据映射成 9 个格子元素
                        let is_hovered = self.hovered_button == Some(index); // 这一格是不是正被悬停
                        let is_hoverable = button.corner.is_some(); // 有锚点的格子才响应悬停
                        div()
                            .relative() // 定位上下文：position: relative（悬浮提示以它为参照）
                            .child(
                                div() // 按钮本体
                                    .id(("button", index)) // 元素唯一 id（名称+序号）：交互方法要求元素带 id
                                    .w(button_size.width) // 宽度：width: 120px
                                    .h(button_size.height) // 高度：height: 65px
                                    .flex() // 弹性布局：display: flex
                                    .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
                                    .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
                                    .bg(gpui::white()) // 背景色：background-color: #ffffff
                                    .when(is_hoverable, |this| {
                                        // 8 个外围按钮才有边框、悬停效果和文字；
                                        // 正中间一格整个跳过：白底上只留一块看不见的空白占位
                                        this.border_1() // 1 像素边框：border-width: 1px
                                            .rounded_lg() // 大圆角：border-radius: 0.5rem (8px)
                                            .border_color(gpui::black()) // 边框颜色：border-color: #000000
                                            .hover(|style| {
                                                style.bg(gpui::black()).text_color(gpui::white())
                                                // bg():        悬停时背景变黑：background-color: #000000
                                                // text_color(): 悬停时文字变白：color: #ffffff
                                            })
                                            .on_hover(cx.listener(
                                                move |this, hovered, _window, cx| {
                                                    // 悬停进入：记下这一格；移开：把记录清掉
                                                    if *hovered {
                                                        this.hovered_button = Some(index);
                                                    } else if this.hovered_button == Some(index) {
                                                        this.hovered_button = None;
                                                    }
                                                    cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                                                },
                                            ))
                                            .child(button.label.clone()) // 按钮文字
                                    }),
                            )
                            .when_some(self.hovered_button.filter(|_| is_hovered), |this, index| {
                                // 只有正被悬停的那一格才挂悬浮提示（is_hovered 保证是自己）
                                let button = &buttons[index];
                                let Some(corner) = button.corner else {
                                    return this; // 防御性写法：没有锚点的格子正常走不到这里
                                };

                                let position = resolved_position(corner, button_size); // 参考点坐标
                                this.child(deferred(
                                    // deferred(): 延迟绘制 —— 布局留在原地，绘制推迟到正文画完之后，
                                    // 所以悬浮提示永远盖在正文上，不会被后渲染的内容盖住。
                                    anchored()
                                        .anchor(corner) // 锚点：提示自己的 corner 角贴住参考点
                                        .position(position) // 参考点坐标（见 resolved_position）
                                        .position_mode(AnchoredPositionMode::Local) // 坐标相对父元素：类似 position: absolute
                                        .snap_to_window() // 超出窗口时贴边收回，保证提示不出窗口
                                        .child(
                                            div() // 提示条：黑底白字的小圆角胶囊
                                                .py_0p5() // 上下内边距：padding-top: 0.125rem; padding-bottom: 0.125rem (2px)
                                                .px_2() // 左右内边距：padding-left: 0.5rem; padding-right: 0.5rem (8px)
                                                .bg(gpui::black().opacity(0.75)) // 75% 不透明的黑背景：background-color: rgb(0 0 0 / 0.75)
                                                .text_color(rgb(0xffffff)) // 文字颜色：color: #ffffff
                                                .rounded_sm() // 小圆角：border-radius: 0.125rem (2px)
                                                .shadow_sm() // 小阴影：box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05)
                                                .min_w(px(100.0)) // 最小宽度：min-width: 100px
                                                .text_sm() // 字号缩小：font-size: 0.875rem; line-height: 1.25rem
                                                .child(button.label.clone()), // 提示文字就用按钮的 label
                                        ),
                                ))
                            })
                    })),
            )
    }
}

// ===========================================================================
// 四、启动应用
// ===========================================================================

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        cx.open_window(
            WindowOptions {
                // 窗口 750×600 像素，位置在屏幕中央
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(750.), px(600.)),
                    cx,
                ))),
                ..Default::default() // 其余选项用默认值
            },
            |_, cx| {
                // 根视图：初始没有任何格子被悬停
                cx.new(|_| AnchorDemo {
                    hovered_button: None,
                })
            },
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic
        cx.activate(true); // 让这个窗口成为活动窗口（抢到输入焦点）
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
