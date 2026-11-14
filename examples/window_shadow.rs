//! 自绘窗口装饰示例：`WindowDecorations::Client` 下的整套「自己画」方案 ——
//! 自绘阴影边框、8 向缩放手柄、自绘标题栏拖动、右键系统窗口菜单。
//!
//! 运行方式：`cargo run --example window_shadow`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 窗口外圈 10 像素的「装饰带」：一圈自绘阴影，同时是 8 个方向的缩放手柄；
//!   2. 内容区：淡紫背景，中间一个白底方块，方块里嵌着绿色的「自绘标题栏」，
//!      按住它能拖动窗口，右键能弹出系统窗口菜单。
//!
//! 关键概念（详见第 22 章）：
//!   1. `WindowDecorations::Client` 装饰由应用自己画；运行时用
//!      `window.window_decorations()` 查实际形态，`Decorations::Client { tiling }`
//!      的 `tiling` 说明哪些边正被平铺贴靠，贴靠的边要关掉圆角/阴影/边框（见 22.1）；
//!   2. `window.set_client_inset(shadow_size)` 声明内容区外还有一圈 10px 的
//!      装饰带，外圈不被内容挡住，缩放手柄也有地方放（见 22.4）；
//!   3. `resize_edge()` 判断鼠标落在哪条边/角，`window.start_window_resize(edge)`
//!      把缩放交给系统；不在边上就 `window.start_window_move()` 交给系统移动（见 22.3）；
//!   4. 自绘标题栏就是一个普通 div，鼠标按下时调 `start_window_move()`，
//!      右键调 `show_window_menu()` 弹系统窗口菜单（见 22.2）；
//!   5. 窗口阴影用 `shadow(vec![BoxShadow::new(...).blur_radius(...)])` 自绘（见 22.5）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, CursorStyle, Decorations, HitboxBehavior, Hsla, MouseButton, Pixels,
    Point, ResizeEdge, Size, Window, WindowBackgroundAppearance, WindowBounds, WindowDecorations,
    WindowOptions, black, canvas, div, green, point, prelude::*, px, rgb, size, transparent_black,
    white,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 示例根实体：没有自己的状态，界面全靠渲染时读窗口的装饰状态现画。
struct WindowShadow {}

// 待办事项（zed 原始示例留下的笔记）：
// 1. 需要一个「计算鼠标在哪条边/角上」的方案，然后据此分派动作；
// 2. 阴影渲染需要大幅改进；
// 3. 把这里的技巧实装到 zed 里。

impl Render for WindowShadow {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 查询当前实际生效的装饰形态：Server（系统画）或 Client { tiling }（自己画）
        let decorations = window.window_decorations();
        let rounding = px(10.0); // 自绘圆角半径：border-radius: 10px
        let shadow_size = px(10.0); // 外圈装饰带宽度：阴影宽度 = 缩放手柄的命中宽度
        let border_size = px(1.0); // 自绘边框宽度：border-width: 1px
        let grey = rgb(0x808080); // 自绘边框颜色：border-color: #808080
        // 声明客户区内边距：内容区四周留 10px 当装饰带（见 22.4）。
        // 要在布局前调用，这段宽度之后既是阴影的落脚处，也是缩放手柄的热区。
        window.set_client_inset(shadow_size);

        div()
            .id("window-backdrop") // 外圈背景：阴影带画在这一层
            .bg(transparent_black()) // 背景色：background-color: transparent
            // map(闭包)：把当前元素交给闭包再变换一次，这里用来按装饰形态分叉
            .map(|div| match decorations {
                // Server：装饰是系统的，外圈什么都不用画
                Decorations::Server => div,
                // Client：外圈自绘 —— 热区 + 光标 + 圆角 + 阴影带 + 拖拽缩放
                Decorations::Client { tiling, .. } => div
                    .bg(gpui::transparent_black()) // 背景色：background-color: transparent
                    .child(
                        canvas(
                            // prepaint：铺满整个窗口插一个热区，鼠标坐标都用它来判断（见第 18 章）
                            |_bounds, window, _cx| {
                                window.insert_hitbox(
                                    Bounds::new(
                                        point(px(0.0), px(0.0)),
                                        window.window_bounds().get_bounds().size,
                                    ),
                                    HitboxBehavior::Normal,
                                )
                            },
                            // paint：鼠标在热区上时，按位置换成对应的 8 种缩放光标（见 22.3）
                            move |_bounds, hitbox, window, _cx| {
                                let mouse = window.mouse_position();
                                let size = window.window_bounds().get_bounds().size;
                                let Some(edge) = resize_edge(mouse, shadow_size, size) else {
                                    return; // 不在外圈：保持默认光标
                                };
                                window.set_cursor_style(
                                    match edge {
                                        ResizeEdge::Top | ResizeEdge::Bottom => {
                                            CursorStyle::ResizeUpDown
                                        }
                                        ResizeEdge::Left | ResizeEdge::Right => {
                                            CursorStyle::ResizeLeftRight
                                        }
                                        ResizeEdge::TopLeft | ResizeEdge::BottomRight => {
                                            CursorStyle::ResizeUpLeftDownRight
                                        }
                                        ResizeEdge::TopRight | ResizeEdge::BottomLeft => {
                                            CursorStyle::ResizeUpRightDownLeft
                                        }
                                    },
                                    &hitbox,
                                );
                            },
                        )
                        .size_full() // 撑满整个窗口：width: 100%; height: 100%
                        .absolute(), // 绝对定位不占布局：position: absolute
                    )
                    // 下面这组 when 是「被平铺贴靠就不画」：
                    // 圆角（贴靠的角）、内边距（贴靠的边没地方放阴影）逐边关掉
                    .when(!(tiling.top || tiling.right), |div| {
                        div.rounded_tr(rounding) // 右上圆角：border-top-right-radius: 10px
                    })
                    .when(!(tiling.top || tiling.left), |div| div.rounded_tl(rounding)) // 左上圆角
                    .when(!tiling.top, |div| div.pt(shadow_size)) // 上内边距：padding-top: 10px
                    .when(!tiling.bottom, |div| div.pb(shadow_size)) // 下内边距：padding-bottom: 10px
                    .when(!tiling.left, |div| div.pl(shadow_size)) // 左内边距：padding-left: 10px
                    .when(!tiling.right, |div| div.pr(shadow_size)) // 右内边距：padding-right: 10px
                    // 鼠标在外圈移动就标脏重绘，canvas 的 paint 重跑一次才能换光标
                    .on_mouse_move(|_e, window, _cx| window.refresh())
                    // 左键按下：落在外圈就交给系统缩放，落在内部就交给系统移动（见 22.3）
                    .on_mouse_down(MouseButton::Left, move |e, window, _cx| {
                        let size = window.window_bounds().get_bounds().size;
                        let pos = e.position;

                        match resize_edge(pos, shadow_size, size) {
                            Some(edge) => window.start_window_resize(edge),
                            None => window.start_window_move(),
                        };
                    }),
            })
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .child(
                div()
                    .cursor(CursorStyle::Arrow) // 内容区恢复默认箭头：cursor: default
                    .map(|div| match decorations {
                        // Server：边框也是系统的，不用画
                        Decorations::Server => div,
                        // Client：内容区四角圆角 + 四边细边框 + 外阴影
                        Decorations::Client { tiling } => div
                            .border_color(grey) // 边框颜色：border-color: #808080
                            .when(!(tiling.top || tiling.right), |div| {
                                div.rounded_tr(rounding) // 右上圆角：border-top-right-radius: 10px
                            })
                            .when(!(tiling.top || tiling.left), |div| div.rounded_tl(rounding)) // 左上圆角
                            .when(!tiling.top, |div| div.border_t(border_size)) // 上边框：border-top-width: 1px
                            .when(!tiling.bottom, |div| div.border_b(border_size)) // 下边框：border-bottom-width: 1px
                            .when(!tiling.left, |div| div.border_l(border_size)) // 左边框：border-left-width: 1px
                            .when(!tiling.right, |div| div.border_r(border_size)) // 右边框：border-right-width: 1px
                            // 没被平铺才画外阴影：贴靠邻窗时阴影会压到人家身上
                            .when(!tiling.is_tiled(), |div| {
                                div.shadow(vec![
                                    // x/y 偏移 0、半透明黑：box-shadow: 0 0 rgba(0, 0, 0, 0.4)
                                    gpui::BoxShadow::new(
                                        px(0.),
                                        px(0.),
                                        Hsla {
                                            h: 0.,
                                            s: 0.,
                                            l: 0.,
                                            a: 0.4,
                                        },
                                    )
                                    .blur_radius(shadow_size / 2.), // 模糊半径：blur 5px
                                ])
                            }),
                    })
                    // 内容区里移动鼠标不冒泡到外圈，外圈就不会重绘、不会换缩放光标
                    .on_mouse_move(|_e, _, cx| {
                        cx.stop_propagation();
                    })
                    .bg(gpui::rgb(0xCCCCFF)) // 背景色：background-color: #ccccff
                    .size_full() // 撑满内容区：width: 100%; height: 100%
                    .flex() // 弹性布局：display: flex
                    .flex_col() // 改成纵向排列：flex-direction: column
                    .justify_around() // 主轴均匀留白（这里是垂直方向）：justify-content: space-around
                    .child(
                        div().w_full().flex().flex_row().justify_around().child(
                            div()
                                .flex() // 弹性布局：display: flex
                                .bg(white()) // 背景色：background-color: #ffffff
                                .size(px(300.0)) // 宽高都是 300 像素：width: 300px; height: 300px
                                .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
                                .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
                                .shadow_lg() // 大阴影：box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), ...
                                .border_1() // 1 像素边框：border-width: 1px
                                .border_color(rgb(0x0000ff)) // 边框颜色：border-color: #0000ff
                                .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
                                .text_color(rgb(0xffffff)) // 文字颜色：color: #ffffff
                                .child(
                                    div()
                                        .id("hello") // 元素唯一 id：下面的交互方法要求元素带 id
                                        .w(px(200.0)) // 宽 200 像素：width: 200px
                                        .h(px(100.0)) // 高 100 像素：height: 100px
                                        .bg(green()) // 背景色：background-color: #00ff00
                                        .shadow(vec![
                                            // 纯黑全不透明阴影：box-shadow: 0 0 #000000
                                            gpui::BoxShadow::new(
                                                px(0.),
                                                px(0.),
                                                Hsla {
                                                    h: 0.,
                                                    s: 0.,
                                                    l: 0.,
                                                    a: 1.0,
                                                },
                                            )
                                            .blur_radius(px(20.0)), // 模糊半径：blur 20px
                                        ])
                                        // 只有 Client 装饰才需要这块「自绘标题栏」：
                                        // 系统装饰时标题栏本来就有，这段交互不用挂
                                        .map(|div| match decorations {
                                            Decorations::Server => div,
                                            Decorations::Client { .. } => div
                                                // 按住标题栏拖动窗口：移动交给系统（见 22.2）
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    |_e, window, _| {
                                                        window.start_window_move();
                                                    },
                                                )
                                                // 右键弹出系统窗口菜单（移动、最小化等，见 22.2）
                                                .on_click(|e, window, _| {
                                                    if e.is_right_click() {
                                                        window.show_window_menu(e.position());
                                                    }
                                                })
                                                .text_color(black()) // 文字颜色：color: #000000
                                                .child("this is the custom titlebar"),
                                        }),
                                ),
                        ),
                    ),
            )
    }
}

/// 按鼠标坐标判断落在哪个缩放边/角上：拿位置和四边的 10px 装饰带比大小，
/// 两轴都贴边是角、只有一轴贴边是边、都不贴返回 `None`（见 22.3）。
/// 角的判断必须排在边之前，否则贴角的位置会先被当成边截走。
fn resize_edge(pos: Point<Pixels>, shadow_size: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let edge = if pos.y < shadow_size && pos.x < shadow_size {
        ResizeEdge::TopLeft
    } else if pos.y < shadow_size && pos.x > size.width - shadow_size {
        ResizeEdge::TopRight
    } else if pos.y < shadow_size {
        ResizeEdge::Top
    } else if pos.y > size.height - shadow_size && pos.x < shadow_size {
        ResizeEdge::BottomLeft
    } else if pos.y > size.height - shadow_size && pos.x > size.width - shadow_size {
        ResizeEdge::BottomRight
    } else if pos.y > size.height - shadow_size {
        ResizeEdge::Bottom
    } else if pos.x < shadow_size {
        ResizeEdge::Left
    } else if pos.x > size.width - shadow_size {
        ResizeEdge::Right
    } else {
        return None;
    };
    Some(edge)
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 窗口初始位置尺寸：屏幕中央 600x600（见 21.4）
        let bounds = Bounds::centered(None, size(px(600.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口化 + 初始位置尺寸
                window_background: WindowBackgroundAppearance::Opaque, // 窗口底色：不透明
                window_decorations: Some(WindowDecorations::Client), // 装饰由应用自己画（见 22.1）
                ..Default::default() // 其余选项用默认值
            },
            |window, cx| {
                cx.new(|cx| {
                    // 系统外观（亮/暗）变化时标脏重绘，自绘装饰跟着重画（见 22.5）
                    cx.observe_window_appearance(window, |_, window, _| {
                        window.refresh(); // 标脏：下一帧重新渲染
                    })
                    .detach(); // 订阅句柄丢出去，订阅一直有效
                    WindowShadow {}
                })
            },
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic
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
