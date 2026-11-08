//! 渐变示例：`linear_gradient()` 造线性渐变当背景，顶部按钮在 Srgb/Oklab 两种
//! 插值空间之间切换，对比同一批色标在不同空间下的过渡效果（见 19.2）。
//!
//! 运行方式：`cargo run --example gradient`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 顶栏：左边标题 "Gradient Examples"，右边一个黑底按钮显示当前色彩空间，点一下切换；
//!   2. 两块纯色方块（红/蓝），给渐变当参照物；
//!   3. 第一排 4 个斜向渐变：45° / 135° / 225° / 315°；
//!   4. 第二排 4 个正方向渐变：0° / 90° / 180° / 360°；
//!   5. 两根色标不铺满 0~1 的渐变条（0.05~0.95），两端各留一段纯色；
//!   6. 一个硬边分界（两个色标同在 0.5）+ 一个只渐变到一半的条；
//!   7. 最底部用 `canvas()` 自绘一个梯形路径，把渐变当「颜色」填进去（见第 18 章）。
//!
//! 关键概念：
//!   1. `linear_gradient(角度, 起始色标, 结束色标)`：角度是**度数**，0 = 向上、
//!      顺时针增大，与 CSS `linear-gradient` 的方向约定一致；只收两个色标（见 19.1）；
//!   2. `linear_color_stop(颜色, 位置)`：**颜色在前**、位置（0.0~1.0）在后；
//!   3. `.color_space(ColorSpace::Srgb/::Oklab)`：决定两个色标之间怎么插值。
//!      默认 `Srgb`（中段容易发灰发暗），`Oklab` 过渡更自然（见 19.2）；
//!   4. 渐变和纯色一样是 `Background` 值：`.bg()` 收它，`window.paint_path()`
//!      填任意路径也收它，一份渐变两处用（见 19.4）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, ColorSpace, Context, Half, Render, Window, WindowOptions, canvas, div,
    linear_color_stop, linear_gradient, point, prelude::*, px, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 渐变查看器：界面是静态的，只有「当前插值色彩空间」一个状态。
struct GradientViewer {
    /// 当前色彩空间，点顶部按钮在 Srgb / Oklab 之间翻转
    color_space: ColorSpace,
}

impl GradientViewer {
    /// 新建状态：默认 `ColorSpace::Srgb`，和 CSS 默认行为一致（见 19.2）。
    fn new() -> Self {
        Self {
            color_space: ColorSpace::default(),
        }
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for GradientViewer {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 把值拷一份出来，后面的闭包要用（闭包不能借 self）
        let color_space = self.color_space;

        div()
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .size_full() // 铺满窗口：width: 100%; height: 100%
            .p_4() // 内边距：padding: 1rem (16px)
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .gap_3() // 子元素间距：gap: 0.75rem (12px)
            // ---------- 顶栏：标题 + 色彩空间切换按钮 ----------
            .child(
                div()
                    .flex()
                    .gap_2()
                    .justify_between() // 两端对齐：justify-content: space-between
                    .items_center() // 交叉轴居中：align-items: center
                    .child("Gradient Examples")
                    .child(
                        div().flex().gap_2().items_center().child(
                            div()
                                .id("method") // 有 id 才是「有状态元素」，才能用下面的 .active() 状态样式
                                .flex()
                                .px_3() // 左右内边距：padding-left: 0.75rem; padding-right: 0.75rem (12px)
                                .py_1() // 上下内边距：padding-top: 0.25rem; padding-bottom: 0.25rem (4px)
                                .text_sm() // 字号：font-size: 0.875rem (14px)
                                .bg(gpui::black()) // 背景色：background-color: #000000
                                .text_color(gpui::white()) // 文字颜色：color: #ffffff
                                .child(format!("{}", color_space)) // 按钮文字 = 当前空间名（"Srgb" / "Oklab"）
                                .active(|this| this.opacity(0.8)) // 按住时变淡（状态样式，见第 7 章）
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    // 点击：在两种插值空间之间翻转，然后通知 GPUI 重绘
                                    this.color_space = match this.color_space {
                                        ColorSpace::Oklab => ColorSpace::Srgb,
                                        ColorSpace::Srgb => ColorSpace::Oklab,
                                    };
                                    cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                                })),
                        ),
                    ),
            )
            // ---------- 两块纯色方块（红 / 蓝），给渐变当参照物 ----------
            .child(
                div()
                    .flex()
                    .flex_1() // 占满剩余高度：flex: 1 1 0%
                    .gap_3()
                    .child(
                        div()
                            .size_full() // 铺满容器：width: 100%; height: 100%
                            .rounded_xl() // 圆角：border-radius: 12px
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(gpui::red()) // 背景色：background-color: #ff0000
                            .text_color(gpui::white()) // 文字颜色：color: #ffffff
                            .child("Solid Color"),
                    )
                    .child(
                        div()
                            .size_full()
                            .rounded_xl()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(gpui::blue()) // 背景色：background-color: #0000ff
                            .text_color(gpui::white())
                            .child("Solid Color"),
                    ),
            )
            // ---------- 第一排：斜向渐变 45° / 135° / 225° / 315° ----------
            // 45° 为例：background-image: linear-gradient(45deg, #ff0000 0%, #0000ff 100%)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .gap_3()
                    .h_24() // 高度：height: 6rem (96px)
                    .text_color(gpui::white())
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            45., // 角度 45°：往右上方向（左下 → 右上）过渡
                            linear_color_stop(gpui::red(), 0.), // 起始色标：红色，位置 0.0
                            linear_color_stop(gpui::blue(), 1.), // 结束色标：蓝色，位置 1.0
                        )
                        .color_space(color_space)), // 插值空间跟着顶部按钮走
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            135., // 135°：往右下方向（左上 → 右下）
                            linear_color_stop(gpui::red(), 0.),
                            linear_color_stop(gpui::green(), 1.),
                        )
                        .color_space(color_space)),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            225., // 225°：往左下方向（右上 → 左下）
                            linear_color_stop(gpui::green(), 0.),
                            linear_color_stop(gpui::blue(), 1.),
                        )
                        .color_space(color_space)),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            315., // 315°：往左上方向（右下 → 左上）
                            linear_color_stop(gpui::green(), 0.),
                            linear_color_stop(gpui::yellow(), 1.),
                        )
                        .color_space(color_space)),
                    ),
            )
            // ---------- 第二排：正方向渐变 0° / 90° / 180° / 360° ----------
            // 0° 从下往上、90° 从左往右、180° 从上往下、360° 等价于 0°
            .child(
                div()
                    .flex()
                    .flex_1()
                    .gap_3()
                    .h_24()
                    .text_color(gpui::white())
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            0., // 0°：从下往上
                            linear_color_stop(gpui::red(), 0.),
                            linear_color_stop(gpui::white(), 1.),
                        )
                        .color_space(color_space)),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            90., // 90°：从左往右
                            linear_color_stop(gpui::blue(), 0.),
                            linear_color_stop(gpui::white(), 1.),
                        )
                        .color_space(color_space)),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            180., // 180°：从上往下
                            linear_color_stop(gpui::green(), 0.),
                            linear_color_stop(gpui::white(), 1.),
                        )
                        .color_space(color_space)),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            360., // 360°：绕一圈回到 0°，同样是从下往上
                            linear_color_stop(gpui::yellow(), 0.),
                            linear_color_stop(gpui::white(), 1.),
                        )
                        .color_space(color_space)),
                    ),
            )
            // ---------- 色标不铺满 0~1：渐变带压到中间，两端各留一段纯色 ----------
            // 位置写 0.05~0.95：相当于 CSS 里的 linear-gradient(#008000 5%, #ffff00 95%)
            .child(
                div().flex_1().rounded_xl().bg(linear_gradient(
                    0., // 从下往上
                    linear_color_stop(gpui::green(), 0.05), // 绿色从 5% 处才开始
                    linear_color_stop(gpui::yellow(), 0.95), // 黄色到 95% 处就到头
                )
                .color_space(color_space)),
            )
            .child(
                div().flex_1().rounded_xl().bg(linear_gradient(
                    90., // 从左往右
                    linear_color_stop(gpui::blue(), 0.05),
                    linear_color_stop(gpui::red(), 0.95),
                )
                .color_space(color_space)),
            )
            // ---------- 硬边分界 & 只渐变一半 ----------
            .child(
                div()
                    .flex()
                    .flex_1()
                    .gap_3()
                    .child(
                        div().flex().flex_1().gap_3().child(
                            div().flex_1().rounded_xl().bg(linear_gradient(
                                90.,
                                // 两个色标位置相同（都是 0.5）：没有过渡，
                                // 是一条硬边分界（CSS 里的 color-stop 重合写法）
                                linear_color_stop(gpui::blue(), 0.5),
                                linear_color_stop(gpui::red(), 0.5),
                            )
                            .color_space(color_space)),
                        ),
                    )
                    .child(
                        div().flex_1().rounded_xl().bg(linear_gradient(
                            180., // 从上往下
                            linear_color_stop(gpui::green(), 0.),
                            linear_color_stop(gpui::blue(), 0.5), // 渐变只走到 50%，剩下的全是蓝
                        )
                        .color_space(color_space)),
                    ),
            )
            // ---------- canvas 自绘：梯形路径填渐变（见第 18 章） ----------
            // canvas() 收两个闭包：prepaint（布局后做测量/预处理）和 paint（真正绘制），
            // prepaint 的返回值会作为第二个参数传给 paint（见 18.1）
            .child(div().h_24().child(canvas(
                move |_, _, _| {}, // prepaint：没什么要测量的，空实现
                move |bounds, _, window, _| {
                    // paint：先按 bounds 拼出一个梯形路径，再用渐变填充
                    let size = size(bounds.size.width * 0.8, px(80.)); // 梯形外接框：宽 = 画布宽的 80%，高 80px
                    let square_bounds = Bounds {
                        origin: point(
                            bounds.size.width.half() - size.width.half(), // 水平居中
                            bounds.origin.y,
                        ),
                        size,
                    };
                    let height = square_bounds.size.height;
                    let horizontal_offset = height; // 左右各往里收一个高度，做出两条斜边
                    let vertical_offset = px(30.);
                    let mut builder = gpui::PathBuilder::fill(); // 填充模式的路径构建器
                    builder.move_to(square_bounds.bottom_left()); // 从左下角出发
                    builder
                        .line_to(square_bounds.origin + point(horizontal_offset, vertical_offset));
                    builder.line_to(
                        square_bounds.top_right() + point(-horizontal_offset, vertical_offset),
                    );

                    builder.line_to(square_bounds.bottom_right());
                    builder.line_to(square_bounds.bottom_left()); // 回到起点，闭合成梯形
                    let path = builder.build().unwrap();
                    window.paint_path(
                        path,
                        linear_gradient(
                            180., // 从上往下
                            linear_color_stop(gpui::red(), 0.),
                            linear_color_stop(gpui::blue(), 1.),
                        )
                        .color_space(color_space), // 渐变在这里当「颜色」用，填进任意路径
                    );
                },
            )))
    }
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // 启动主循环，在闭包里做初始化
    application().run(|cx: &mut App| {
        // 加载字体；失败就直接退出
        if !example_support::load_fonts(cx) {
            return;
        }
        // 开窗口：内容就是那个渐变查看器
        cx.open_window(
            WindowOptions {
                focus: true, // 打开后自动拿键盘焦点
                ..Default::default()
            },
            |_, cx| cx.new(|_| GradientViewer::new()),
        )
        .unwrap();
        // 把应用弄到前台来（忽略其他应用）
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
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
