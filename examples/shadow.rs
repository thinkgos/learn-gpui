//! 阴影示例：`BoxShadow` 自定义阴影 + 7 级预设（`shadow_2xs()` ~ `shadow_2xl()`）
//! 的效果对照，按行分别看形状、预设档位、模糊半径、扩散半径、方向、多层叠加和
//! 内阴影（见 4.3）。
//!
//! 运行方式：`cargo run --example shadow`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块（从上到下每行一类对照，每格上面是白卡片、下面是标签）：
//!   1. 同一个阴影打在五种形状上：直角 / 圆角 4px / 8px / 12px / 圆形；
//!   2. 阴影预设 8 档：无 → 2XS → XS → S → M → L → XL → 2XL；
//!   3. 模糊半径递增：Blur 0 → 2 → 4 → 8 → 16；
//!   4. 扩散半径递增：Spread 0 → 2 → 4 → 8 → 16；
//!   5. 直角 / 大圆角元素上的扩散对照；
//!   6. 方向阴影：偏移把阴影推向左 / 右 / 上 / 下（圆形、直角、大圆角各一行）；
//!   7. 多层阴影：一个元素同时叠 4 个彩色阴影（上红、右黄、下绿、左蓝）；
//!   8. 内阴影：`.inset()` 的 5 种参数组合，以及「外阴影 + 内阴影」同框叠加。
//!
//! 关键概念：
//!   1. `BoxShadow::new(x偏移, y偏移, 颜色)` 固定三参，其余用链式补：
//!      `.blur_radius()`（模糊）、`.spread_radius()`（扩散）、`.inset()`（内阴影）（见 4.3）；
//!   2. `.shadow(vec![...])` 收一个列表，可以一次叠多层阴影（列表里后面的画在更上层）；
//!   3. 7 个预设 `shadow_2xs()` ~ `shadow_2xl()` 的数值与 Tailwind 的 shadow 刻度一致；
//!   4. 阴影跟随圆角裁剪：同一个阴影打在不同形状上，阴影轮廓始终贴着元素形状。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, BoxShadow, Context, Div, SharedString, Window, WindowBounds, WindowOptions, div,
    hsla, prelude::*, px, relative, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、阴影「底板」：五种形状的白色卡片
// ===========================================================================

/// 阴影对照用的底板工厂：五种形状共用同一套白卡片样式，
/// 只差圆角，方便看「阴影跟随圆角裁剪」这一点。
struct Shadow {}

impl Shadow {
    /// 白色圆形卡片（64×64）。
    fn base() -> Div {
        div()
            .size_16() // 16 号尺寸的正方形：width: 4rem; height: 4rem (64px)
            .bg(rgb(0xffffff)) // 背景色：background-color: #ffffff
            .rounded_full() // 圆形：border-radius: 9999px
            .border_1() // 边框：border-width: 1px
            .border_color(hsla(0.0, 0.0, 0.0, 0.1)) // 边框颜色：10% 黑（描边只是让白卡片可见）
    }

    /// 白色直角卡片（不倒圆角）。
    fn square() -> Div {
        div()
            .size_16()
            .bg(rgb(0xffffff))
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.1))
    }

    /// 小圆角卡片（4px）。
    fn rounded_small() -> Div {
        div()
            .size_16()
            .bg(rgb(0xffffff))
            .rounded(px(4.)) // 四角 4px：border-radius: 4px
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.1))
    }

    /// 中圆角卡片（8px）。
    fn rounded_medium() -> Div {
        div()
            .size_16()
            .bg(rgb(0xffffff))
            .rounded(px(8.)) // 四角 8px：border-radius: 8px
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.1))
    }

    /// 大圆角卡片（12px）。**NOTE**: 界面上的标签写的是 "Rounded 16"，
    /// 实际圆角以代码为准（12px）。
    fn rounded_large() -> Div {
        div()
            .size_16()
            .bg(rgb(0xffffff))
            .rounded(px(12.)) // 四角 12px：border-radius: 12px
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.1))
    }
}

// ===========================================================================
// 二、展示单元：卡片 + 底部标签
// ===========================================================================

/// 把「示例卡片 + 底部标签」拼成一列展示单元，宽度是父容器的 1/6，
/// 右边框把相邻格子隔开。
fn example(label: impl Into<SharedString>, example: impl IntoElement) -> impl IntoElement {
    let label = label.into();

    div()
        .flex() // 弹性布局：display: flex
        .flex_col() // 纵向排列：flex-direction: column
        .justify_center() // 主轴（纵向）居中：justify-content: center
        .items_center() // 交叉轴（横向）居中：align-items: center
        .w(relative(1. / 6.)) // 宽度 = 父容器的 1/6：width: 16.667%
        .border_r_1() // 右边框：border-right-width: 1px
        .border_color(hsla(0.0, 0.0, 0.0, 1.0)) // 边框颜色：#000000
        .child(
            // 上半：卡片展示区，垂直方向占满剩余高度
            div()
                .flex()
                .items_center()
                .justify_center()
                .flex_1() // 占满剩余高度：flex: 1 1 0%
                .py_12() // 上下内边距：padding-top: 3rem; padding-bottom: 3rem (48px)
                .child(example),
        )
        .child(
            // 下半：标签条，用顶边框和展示区分开
            div()
                .w_full() // 铺满宽度：width: 100%
                .border_t_1() // 顶边框：border-top-width: 1px
                .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                .p_1() // 内边距：padding: 0.25rem (4px)
                .flex()
                .items_center()
                .child(label),
        )
}

// ===========================================================================
// 三、根组件：一行行对照渲染
// ===========================================================================

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for Shadow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("shadow-example") // 有 id 才是「有状态元素」，下面的滚动方法才用得了（见 4.7）
            .overflow_y_scroll() // 内容超高时出纵向滚动条：overflow-y: scroll
            .bg(rgb(0xffffff)) // 背景色：background-color: #ffffff
            .size_full() // 铺满窗口：width: 100%; height: 100%
            .text_xs() // 标签字号：font-size: 0.75rem (12px)
            .child(div().flex().flex_col().w_full().children(vec![
                    // ---------- 第一行：同一个阴影打在五种形状上 ----------
                    // 阴影跟随圆角裁剪：轮廓始终贴着元素形状（见 4.3）
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .flex_row() // 横向排列：flex-direction: row
                        .children(vec![
                            example(
                                "Square",
                                Shadow::square().shadow(vec![
                                    // 整条对应 CSS：box-shadow: 0 8px 8px 0 rgb(255 0 0 / 0.3)
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3)) // x/y 偏移 0/8px，30% 透明的红
                                        .blur_radius(px(8.)), // 模糊半径：box-shadow 的 blur 部分
                                ]),
                            ),
                            example(
                                "Rounded 4",
                                Shadow::rounded_small().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded 8",
                                Shadow::rounded_medium().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded 16",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Circle",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第二行：7 级阴影预设 + 无阴影对照 ----------
                    // 预设数值与 Tailwind 的 shadow 刻度一致（见 4.3 的对照表）
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .w_full()
                        .children(vec![
                            example("None", Shadow::base()), // 无阴影，当基准
                            // 极轻微阴影（细分隔线级）
                            example("2X Small", Shadow::base().shadow_2xs()), // box-shadow: 0 1px 0 0 rgb(0 0 0/0.05)
                            // 超小阴影（轻微浮起）
                            example("Extra Small", Shadow::base().shadow_xs()), // box-shadow: 0 1px 2px 0 rgb(0 0 0/0.05)
                            // 小阴影（按钮悬停、小卡片）
                            example("Small", Shadow::base().shadow_sm()), // box-shadow: 0 1px 3px 0 rgb(0 0 0/0.1), ...
                            // 中阴影（卡片、下拉菜单）
                            example("Medium", Shadow::base().shadow_md()), // box-shadow: 0 4px 6px -1px rgb(0 0 0/0.1), ...
                            // 大阴影（弹出面板、对话框）
                            example("Large", Shadow::base().shadow_lg()), // box-shadow: 0 10px 15px -3px rgb(0 0 0/0.1), ...
                            // 超大阴影（模态框、重要提示）
                            example("Extra Large", Shadow::base().shadow_xl()), // box-shadow: 0 20px 25px -5px rgb(0 0 0/0.1), ...
                            // 最大阴影（全局拖拽预览、最高层级）
                            example("2X Large", Shadow::base().shadow_2xl()), // box-shadow: 0 25px 50px -12px rgb(0 0 0/0.25)
                        ]),
                    // ---------- 第三行：模糊半径递增的一行（Blur 0 → 16） ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Blur 0",
                                Shadow::base().shadow(vec![BoxShadow::new(
                                    px(0.),
                                    px(8.),
                                    hsla(0.0, 0.0, 0.0, 0.3),
                                )]), // 不调 .blur_radius() 就是 0：边缘是硬的
                            ),
                            example(
                                "Blur 2",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(2.)), // 模糊 2px
                                ]),
                            ),
                            example(
                                "Blur 4",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(4.)),
                                ]),
                            ),
                            example(
                                "Blur 8",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Blur 16",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(16.)), // 模糊越大越「散」
                                ]),
                            ),
                        ]),
                    // ---------- 第四行：扩散半径递增的一行（Spread 0 → 16） ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Spread 0",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.)),
                                ]), // 不调 .spread_radius() 就是 0：阴影和元素一样大
                            ),
                            example(
                                "Spread 2",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(2.)), // 扩散半径：box-shadow 的 spread 部分，向外撑大 2px
                                ]),
                            ),
                            example(
                                "Spread 4",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(4.)),
                                ]),
                            ),
                            example(
                                "Spread 8",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Spread 16",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(16.)), // 扩散越大，阴影圈越往外涨
                                ]),
                            ),
                        ]),
                    // ---------- 第五行：直角卡片上的扩散对照 ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Square Spread 0",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Square Spread 8",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Square Spread 16",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(16.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第六行：大圆角卡片上的扩散对照 ----------
                    // 扩散不「削」掉圆角：阴影轮廓照样贴着圆角形状走
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Rounded Large Spread 0",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Spread 8",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Spread 16",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.3))
                                        .blur_radius(px(8.))
                                        .spread_radius(px(16.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第七行：方向阴影（圆形底板） ----------
                    // 改 x/y 偏移就能把阴影推向任意方向
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Left",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(-8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3)) // x = -8px：阴影打在左边
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Right",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3)) // x = 8px：阴影打在右边
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Top",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(-8.), hsla(0.0, 0.5, 0.5, 0.3)) // y = -8px：阴影打在上边
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Bottom",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3)) // y = 8px：阴影打在下边
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第八行：方向阴影（直角底板） ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Square Left",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(-8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Square Right",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Square Top",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(-8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Square Bottom",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第九行：方向阴影（大圆角底板） ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Rounded Large Left",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(-8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Right",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(8.), px(0.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Top",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(-8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Bottom",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.5, 0.5, 0.3))
                                        .blur_radius(px(8.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第十行：多层阴影 ----------
                    // .shadow(vec![...]) 一次叠 4 层彩色阴影，列表里后面的画在更上层：
                    // 上红（0°）、右黄（60/360）、下绿（120/360）、左蓝（240/360）
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .children(vec![
                            example(
                                "Circle Multiple",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(
                                        px(0.),
                                        px(-12.),
                                        hsla(0.0 / 360., 1.0, 0.5, 0.3), // 色相 0°/360° = 红
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(12.),
                                        px(0.),
                                        hsla(60.0 / 360., 1.0, 0.5, 0.3), // 色相 60°/360° = 黄
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(0.),
                                        px(12.),
                                        hsla(120.0 / 360., 1.0, 0.5, 0.3), // 色相 120°/360° = 绿
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(-12.),
                                        px(0.),
                                        hsla(240.0 / 360., 1.0, 0.5, 0.3), // 色相 240°/360° = 蓝
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                ]),
                            ),
                            example(
                                "Square Multiple",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(
                                        px(0.),
                                        px(-12.),
                                        hsla(0.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(12.),
                                        px(0.),
                                        hsla(60.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(0.),
                                        px(12.),
                                        hsla(120.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(-12.),
                                        px(0.),
                                        hsla(240.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                ]),
                            ),
                            example(
                                "Rounded Large Multiple",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(
                                        px(0.),
                                        px(-12.),
                                        hsla(0.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(12.),
                                        px(0.),
                                        hsla(60.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(0.),
                                        px(12.),
                                        hsla(120.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                    BoxShadow::new(
                                        px(-12.),
                                        px(0.),
                                        hsla(240.0 / 360., 1.0, 0.5, 0.3),
                                    )
                                    .blur_radius(px(8.))
                                    .spread_radius(px(2.)),
                                ]),
                            ),
                        ]),
                    // ---------- 第十一行：内阴影（CSS 里就是 box-shadow: inset ...） ----------
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .w_full()
                        .children(vec![
                            example(
                                "Inset basic",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(0.), hsla(0.0, 0.0, 0.0, 0.5))
                                        .blur_radius(px(12.))
                                        .inset(), // 内阴影：box-shadow: inset 0 0 12px rgb(0 0 0/0.5)
                                ]),
                            ),
                            example(
                                "Inset offset",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(6.), px(6.), hsla(0.0, 0.0, 0.0, 0.5)) // 带偏移的内阴影，光源感在右下
                                        .blur_radius(px(8.))
                                        .inset(),
                                ]),
                            ),
                            example(
                                "Inset spread",
                                Shadow::base().shadow(vec![
                                    BoxShadow::new(px(0.), px(0.), hsla(0.0, 0.0, 0.0, 0.5))
                                        .blur_radius(px(4.))
                                        .spread_radius(px(8.)) // 内阴影同样能扩散：往「内」涨
                                        .inset(),
                                ]),
                            ),
                            example(
                                "Inset rounded",
                                Shadow::rounded_large().shadow(vec![
                                    BoxShadow::new(px(0.), px(4.), hsla(0.0, 0.0, 0.0, 0.5))
                                        .blur_radius(px(10.))
                                        .spread_radius(px(2.))
                                        .inset(),
                                ]),
                            ),
                            example(
                                "Inset sharp",
                                Shadow::square().shadow(vec![
                                    BoxShadow::new(px(0.), px(0.), hsla(0.0, 0.0, 0.0, 0.6))
                                        .spread_radius(px(6.)) // 不模糊、只扩散：得到一圈硬边内框
                                        .inset(),
                                ]),
                            ),
                        ]),
                    // ---------- 第十二行：外阴影 + 内阴影叠加在同一个元素上 ----------
                    // vec 里两层一起给：第一层外阴影，第二层内阴影
                    div()
                        .border_b_1()
                        .border_color(hsla(0.0, 0.0, 0.0, 1.0))
                        .flex()
                        .w_full()
                        .children(vec![example(
                            "Drop + Inset",
                            Shadow::rounded_medium().shadow(vec![
                                BoxShadow::new(px(0.), px(8.), hsla(0.0, 0.0, 0.0, 0.25)) // 外阴影：托底
                                    .blur_radius(px(12.)),
                                BoxShadow::new(px(0.), px(2.), hsla(0.0, 0.0, 0.0, 0.4)) // 内阴影：压出内凹感
                                    .blur_radius(px(4.))
                                    .inset(),
                            ]),
                        )]),
                ]))
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
        // 开一个 1000×800 的窗口，位置在屏幕中央
        let bounds = Bounds::centered(None, size(px(1000.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Shadow {}),
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
