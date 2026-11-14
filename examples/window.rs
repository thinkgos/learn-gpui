//! 多窗口示例：用 `cx.open_window()` + `WindowOptions` 开出各种类型的窗口，
//! 一次看全窗口种类（WindowKind）、自绘标题栏、行为开关、隐藏应用、
//! 调整尺寸和系统对话框（prompt）等窗口管理 API。
//!
//! 运行方式：`cargo run --example window`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 主窗口（WindowDemo）：一排按钮，每个按钮用不同的 `WindowOptions` 开一个子窗口，
//!      或者触发一次窗口/应用级操作（Hide Application、Resize、Prompt）；
//!   2. 子窗口（SubWindow）：250x200 的小窗口，内容是一行标题和「Close」按钮；
//!      Dialog 类型的子窗口里还多一个「Open Nested Dialog」按钮，可以层层嵌套。
//!
//! 关键概念（详见第 20 章）：
//!   1. `cx.open_window(WindowOptions {...}, |window, cx| ...)` 开新窗口，
//!      第二个闭包负责造出根实体（这里是 SubWindow）；
//!   2. `WindowKind` 决定窗口层级和交互方式：Normal/PopUp/Floating/Dialog；
//!   3. `titlebar: None` 表示不要系统标题栏，标题栏由界面自己画（Custom Titlebar）；
//!   4. `window.prompt()` 是系统级对话框，返回 `oneshot::Receiver<usize>`，异步等用户点按钮；
//!   5. `cx.hide()`/`cx.activate()` 隐藏和恢复的是整个应用，不是单个窗口。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, KeyBinding, PromptButton, PromptLevel, Window, WindowBounds, WindowKind,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// ===========================================================================
// 一、子窗口的根实体 SubWindow
// ===========================================================================

/// 所有子窗口共用的根实体：两个开关分别控制「自绘标题栏」和「嵌套对话框按钮」。
struct SubWindow {
    /// 是否画自绘标题栏（配合开窗时的 `titlebar: None` 一起用）。
    custom_titlebar: bool,
    /// 是否是对话框：是的话多一个「Open Nested Dialog」按钮。
    is_dialog: bool,
}

// ===========================================================================
// 二、通用按钮
// ===========================================================================

/// 造一个浅灰小按钮。`text` 既是显示文字，也当元素 id 用；
/// 点击回调直接收 (window, cx)，方便里面做开窗、关窗这类窗口级操作。
fn button(text: &str, on_click: impl Fn(&mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(text.to_string()) // 元素唯一 id：on_click 等交互方法要求元素带 id（见第 7 章）
        .flex_none() // 不参与弹性伸缩：flex: none
        .px_2() // 左右内边距：padding: 0 0.5rem (8px)
        .bg(rgb(0xf7f7f7)) // 背景色：background-color: #f7f7f7
        .active(|this| this.opacity(0.85)) // 按住时略变透明（状态样式，见第 7 章）：opacity: 0.85
        .border_1() // 1 像素边框：border-width: 1px
        .border_color(rgb(0xe0e0e0)) // 边框颜色：border-color: #e0e0e0
        .rounded_sm() // 小圆角：border-radius: 0.125rem (2px)
        .cursor_pointer() // 悬停显示手型：cursor: pointer
        .child(text.to_string())
        .on_click(move |_, window, cx| on_click(window, cx)) // 点击时把 window/cx 交给回调
}

// ===========================================================================
// 三、子窗口的界面
// ===========================================================================

impl Render for SubWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 嵌套对话框的初始位置尺寸：屏幕中央 250x200。
        // 这里只是算出 Bounds，真正开窗在「Open Nested Dialog」按钮的回调里。
        let window_bounds =
            WindowBounds::Windowed(Bounds::centered(None, size(px(250.0), px(200.0)), cx));

        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .bg(rgb(0xffffff)) // 背景色：background-color: #ffffff
            .size_full() // 撑满整个窗口：width: 100%; height: 100%
            .gap_2() // 子元素之间的间距：gap: 0.5rem (8px)
            // when(条件, ...): 条件为真才追加这一段，这里是自绘标题栏
            .when(self.custom_titlebar, |cx| {
                cx.child(
                    div()
                        .flex() // 弹性布局：display: flex
                        .h(px(32.)) // 标题栏高 32 像素：height: 32px
                        .px_4() // 左右内边距：padding: 0 1rem (16px)
                        .bg(gpui::blue()) // 背景色：background-color: #0000ff
                        .text_color(gpui::white()) // 文字颜色：color: #ffffff
                        .w_full() // 宽度撑满：width: 100%
                        .child(
                            div()
                                .flex() // 弹性布局：display: flex
                                .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                                .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                                .size_full() // 撑满标题栏：width: 100%; height: 100%
                                .child("Custom Titlebar"),
                        ),
                )
            })
            .child(
                div()
                    .p_8() // 四周内边距：padding: 2rem (32px)
                    .flex() // 弹性布局：display: flex
                    .flex_col() // 纵向排列：flex-direction: column
                    .gap_2() // 子元素之间的间距：gap: 0.5rem (8px)
                    .child("SubWindow")
                    // when(条件, ...): 对话框才多一个嵌套按钮，再开一层 Dialog（可以无限套娃）
                    .when(self.is_dialog, |div| {
                        div.child(button("Open Nested Dialog", move |_, cx| {
                            cx.open_window(
                                WindowOptions {
                                    window_bounds: Some(window_bounds), // 窗口化 + 初始位置尺寸
                                    kind: WindowKind::Dialog, // 模态对话框：挡住父窗口交互直到关闭
                                    ..Default::default() // 其余选项用默认值
                                },
                                |_, cx| {
                                    // 第二个闭包造出新窗口的根实体，这里又是一个「对话框」SubWindow
                                    cx.new(|_| SubWindow {
                                        custom_titlebar: false,
                                        is_dialog: true,
                                    })
                                },
                            )
                            .unwrap(); // 简单示例里直接 unwrap，失败就 panic
                        }))
                    })
                    .child(button("Close", |window, _| {
                        window.remove_window(); // 关掉「自己这个」窗口（见 20.3）
                    })),
            )
    }
}

// ===========================================================================
// 四、主窗口的界面 WindowDemo
// ===========================================================================

/// 主窗口的根实体：没有自己的状态，界面就是一排演示按钮。
struct WindowDemo {}

impl Render for WindowDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 多数按钮开出来的子窗口共用这个位置尺寸：屏幕中央 300x300。
        let window_bounds =
            WindowBounds::Windowed(Bounds::centered(None, size(px(300.0), px(300.0)), cx));

        div()
            .p_4() // 四周内边距：padding: 1rem (16px)
            .flex() // 弹性布局：display: flex
            .flex_wrap() // 放不下就换行：flex-wrap: wrap
            .bg(rgb(0xffffff)) // 背景色：background-color: #ffffff
            .size_full() // 撑满整个窗口：width: 100%; height: 100%
            .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
            .content_center() // 换行后整块在交叉轴上居中（这里是垂直居中）：align-content: center
            .gap_2() // 按钮之间的间距：gap: 0.5rem (8px)
            // 下面每个按钮都是「用某个 WindowOptions 开一个子窗口」，从 Normal 到
            // Unminimizable 依次只改一个字段，想知道某字段干什么，对比相邻两个按钮即可。
            .child(button("Normal", move |_, cx| {
                // 最普通的窗口：只给位置尺寸，其余全默认（kind 默认是 Normal）。
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(window_bounds), // 窗口化 + 初始位置尺寸
                        ..Default::default() // 其余选项用默认值
                    },
                    |_, cx| {
                        // 第二个闭包造出新窗口的根实体
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap(); // 简单示例里直接 unwrap，失败就 panic
            }))
            .child(button("Popup", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(window_bounds),
                        kind: WindowKind::PopUp, // 置顶的弹出窗口（见 20.2）
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Floating", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(window_bounds),
                        kind: WindowKind::Floating, // 浮在父窗口之上的工具窗口（见 20.2）
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Dialog", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(window_bounds),
                        kind: WindowKind::Dialog, // 模态对话框（见 20.2）
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: true, // 对话框里再放一个「开嵌套对话框」按钮
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Custom Titlebar", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        titlebar: None, // 不要系统标题栏，标题栏由界面自己画（见 20.1）
                        window_bounds: Some(window_bounds),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: true, // 界面里补画一条蓝色标题栏
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Invisible", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        show: false, // 创建后不显示（隐藏窗口，见 20.1）
                        window_bounds: Some(window_bounds),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Unmovable", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        is_movable: false, // 用户拖不动这个窗口（见 20.1）
                        titlebar: None,
                        window_bounds: Some(window_bounds),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Unresizable", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        is_resizable: false, // 用户不能调整大小（见 20.1）
                        window_bounds: Some(window_bounds),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Unminimizable", move |_, cx| {
                cx.open_window(
                    WindowOptions {
                        is_minimizable: false, // 用户不能最小化（见 20.1）
                        window_bounds: Some(window_bounds),
                        ..Default::default()
                    },
                    |_, cx| {
                        cx.new(|_| SubWindow {
                            custom_titlebar: false,
                            is_dialog: false,
                        })
                    },
                )
                .unwrap();
            }))
            .child(button("Hide Application", |window, cx| {
                cx.hide(); // 隐藏的是整个应用，不是这一个窗口（见 20.3）

                // 3 秒后把应用重新激活：起个异步任务等定时器，不阻塞界面
                window
                    .spawn(cx, async move |cx| {
                        cx.background_executor()
                            .timer(std::time::Duration::from_secs(3))
                            .await; // 在后台执行器上睡 3 秒
                        cx.update(|_, cx| {
                            cx.activate(false); // 恢复应用到前台
                        })
                    })
                    .detach(); // 任务句柄丢出去自己跑，不跟踪结果
            }))
            .child(button("Resize", |window, _| {
                // 把内容区宽高对调，连点几下能看到尺寸在变（见 20.3）
                let content_size = window.bounds().size;
                window.resize(size(content_size.height, content_size.width));
            }))
            .child(button("Prompt", |window, cx| {
                // 系统对话框：level 用 Info；按钮传 &str 时 "ok"/"cancel" 会映射成系统按钮（见 20.3）
                let answer = window.prompt(
                    PromptLevel::Info,
                    "Are you sure?",
                    None,
                    &["OK", "Cancel"],
                    cx,
                );

                // prompt 返回 oneshot::Receiver：结果是异步的，在任务里 await 拿按钮下标
                cx.spawn(async move |_| {
                    if answer.await.unwrap() == 0 {
                        println!("You have clicked Ok");
                    } else {
                        println!("You have clicked Cancel");
                    }
                })
                .detach(); // 任务句柄丢出去自己跑
            }))
            .child(button("Prompt (non-English)", |window, cx| {
                // 中文按钮要用 PromptButton 显式指定语义：直接传 &str 的话，
                // 非 "ok"/"cancel" 文字都只算普通按钮（见 20.3）
                let answer = window.prompt(
                    PromptLevel::Info,
                    "Are you sure?",
                    None,
                    &[PromptButton::ok("确定"), PromptButton::cancel("取消")],
                    cx,
                );

                cx.spawn(async move |_| {
                    if answer.await.unwrap() == 0 {
                        println!("You have clicked Ok");
                    } else {
                        println!("You have clicked Cancel");
                    }
                })
                .detach();
            }))
    }
}

// ===========================================================================
// 五、应用入口
// ===========================================================================

// actions! 宏声明一个「退出」动作类型（见第 9 章）。
actions!(window, [Quit]);

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 主窗口的初始位置尺寸：屏幕中央 800x600。
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口化 + 初始位置尺寸
                ..Default::default() // 其余选项用默认值
            },
            |window, cx| {
                cx.new(|cx| {
                    // 监听窗口移动/缩放，变化时打印新的 bounds（见 20.4）。
                    // 注册写在实体创建时而不是 render 里：render 每帧都会跑，订阅会越堆越多。
                    cx.observe_window_bounds(window, move |_, window, _| {
                        println!("Window bounds changed: {:?}", window.bounds());
                    })
                    .detach(); // 订阅句柄丢出去，订阅一直有效

                    WindowDemo {}
                })
            },
        )
        .unwrap(); // 简单示例里直接 unwrap，失败就 panic

        // 让这个窗口成为活动窗口（抢到输入焦点）。
        cx.activate(true);
        cx.on_action(|_: &Quit, cx| cx.quit()); // Quit 动作的处理：退出应用
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]); // 键位绑定：cmd-q 触发 Quit（见第 9 章）
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
