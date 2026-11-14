//! 系统通知示例：向操作系统的通知中心发送、替换、关闭系统通知，
//! 并接收用户点通知正文或操作按钮后的响应（详见第 20 章）。
//!
//! 运行方式：`cargo run --example system_notifications`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 标题和一段说明文字；
//!   2. 两个按钮：「Show or replace」反复发送（同一个 tag 会替换旧通知）、
//!      「Dismiss」关闭通知；
//!   3. 一个状态卡片，显示最近一次操作或收到的通知响应；
//!   4. macOS 下多一行小字提醒：只有从 app bundle 里跑才会真的弹通知。
//!
//! 关键概念（详见第 20 章）：
//!   1. `cx.set_app_identity()`：设置应用标识和显示名，通知中心靠它归组；
//!   2. `cx.show_system_notification()` / `cx.dismiss_system_notification()`：
//!      发送和关闭通知，`tag` 相同的两次发送是「替换」而不是「多发一条」；
//!   3. `cx.on_system_notification_response()`：注册响应回调，
//!      `action_id` 为 `Some` 说明用户点了操作按钮，为 `None` 说明点了通知正文。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, Div, SharedString, Stateful, SystemNotification,
    SystemNotificationAction, SystemNotificationResponse, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 通知的 tag：标识「这是同一条通知」，重复发送会替换，响应也带它回来。
const NOTIFICATION_TAG: &str = "gpui-system-notification-example";

/// 示例组件：记录发送次数和最近一次操作结果，都显示在界面上。
struct SystemNotificationExample {
    /// 发送次数：拼进标题里，方便肉眼确认「替换」真的发生了。
    revision: usize,
    /// 状态卡片里显示的最近一条信息。
    status: SharedString,
}

impl Render for SystemNotificationExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex() // 弹性布局：display: flex
            .flex_col() // 改成纵向排列：flex-direction: column
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .gap_4() // 子元素之间的间距：gap: 1rem (16px)
            .p_8() // 四周内边距：padding: 2rem (32px)
            .bg(rgb(0x18181b)) // 背景色：background-color: #18181b
            .text_color(rgb(0xf4f4f5)) // 文字颜色：color: #f4f4f5
            .child(div().text_2xl().child("GPUI system notifications"))
            .child(div().text_sm().text_color(rgb(0xa1a1aa)).child(
                "Post repeatedly to replace the notification with the same tag. Click the \
                         notification body or an action button to send a response back to GPUI.",
            ))
            .child(
                div()
                    .flex() // 弹性布局：display: flex
                    .gap_3() // 按钮之间的间距：gap: 0.75rem (12px)
                    // 「Show or replace」按钮：发送一条带两个操作按钮的通知
                    .child(button("show", "Show or replace").on_click(cx.listener(
                        |this, _, _, cx| {
                            // listener 闭包收四个参数：自己的状态、事件、window、cx（见第 12 章）
                            this.revision += 1;
                            let revision = this.revision;
                            cx.show_system_notification(SystemNotification {
                                // tag 不变：第 2 次起就是替换同一条通知，而不是新发一条
                                tag: NOTIFICATION_TAG.into(),
                                title: format!("Example notification {revision}").into(),
                                body: "This notification was posted by the GPUI example.".into(),
                                // 通知上的操作按钮：用户点了会把 id 回传给响应回调
                                actions: vec![
                                    SystemNotificationAction {
                                        id: "open".into(),
                                        label: "Open".into(),
                                    },
                                    SystemNotificationAction {
                                        id: "snooze".into(),
                                        label: "Snooze".into(),
                                    },
                                ],
                            });
                            this.status = format!("Posted notification revision {revision}").into();
                            cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                        },
                    )))
                    // 「Dismiss」按钮：按 tag 关掉通知（尽力而为，有的平台撤不回已展示的通知）
                    .child(
                        button("dismiss", "Dismiss").on_click(cx.listener(|this, _, _, cx| {
                            cx.dismiss_system_notification(NOTIFICATION_TAG);
                            this.status = "Dismissed the notification".into();
                            cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                        })),
                    ),
            )
            .child(
                div()
                    .mt_2() // 上外边距：margin-top: 0.5rem (8px)
                    .p_4() // 四周内边距：padding: 1rem (16px)
                    .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                    .bg(rgb(0x27272a)) // 背景色：background-color: #27272a
                    .child(self.status.clone()),
            )
            // when(条件, ...): 只在 macOS 上追加这行小字提醒
            .when(cfg!(target_os = "macos"), |this| {
                this.child(div().mt_2().text_xs().text_color(rgb(0x71717a)).child(
                    "macOS only delivers notifications when this example runs from an app bundle.",
                ))
            })
    }
}

/// 造一个深灰小按钮。`id` 是元素唯一标识（`on_click` 要求元素带 id），`label` 是显示文字。
fn button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id) // 元素唯一 id：on_click 等交互方法要求元素带 id（见第 7 章）
        .px_4() // 左右内边距：padding: 0 1rem (16px)
        .py_2() // 上下内边距：padding: 0.5rem 0
        .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
        .bg(rgb(0x3f3f46)) // 背景色：background-color: #3f3f46
        .hover(|style| style.bg(rgb(0x52525b))) // 悬停变亮（状态样式，见第 7 章）：#52525b
        .active(|style| style.bg(rgb(0x71717a))) // 按住再亮一点（状态样式，见第 7 章）：#71717a
        .cursor_pointer() // 悬停显示手型：cursor: pointer
        .child(label)
}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    application().run(|cx: &mut App| {
        // 先加载字体；失败就直接退出（比如字体文件找不到）。
        if !example_support::load_fonts(cx) {
            return;
        }
        // 设置应用标识和显示名：通知中心靠 identifier 归组、用 name 展示，
        // 要在开窗口、发通知之前设置（见第 20 章）。
        cx.set_app_identity("dev.zed.gpui.system-notifications", "GPUI Notifications");

        let view = cx.new(|_| SystemNotificationExample {
            revision: 0,
            status: "No notification posted yet".into(),
        });
        // 注册通知响应回调（App 级，一个应用一个）：
        // 用户点通知正文或操作按钮都会走到这里（见第 20 章）。
        cx.on_system_notification_response({
            let view = view.clone(); // 克隆一份实体句柄，回调里才能更新界面
            move |response, cx| {
                let SystemNotificationResponse { tag, action_id } = response;
                view.update(cx, |this, cx| {
                    this.status = match action_id {
                        // Some(action_id)：用户点的是某个操作按钮
                        Some(action_id) => {
                            format!("Received action '{action_id}' for tag '{tag}'").into()
                        }
                        // None：用户点的是通知正文
                        None => format!("Notification body clicked for tag '{tag}'").into(),
                    };
                    cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                });
            }
        });

        // 主窗口的初始位置尺寸：屏幕中央 560x360。
        let bounds = Bounds::centered(None, size(px(560.), px(360.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口化 + 初始位置尺寸
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("System Notifications Example".into()), // 系统标题栏文字
                    ..Default::default() // 其余选项用默认值
                }),
                ..Default::default()
            },
            move |_, _| view, // 根实体直接用上面建好的，闭包只是把它交出去
        )
        .expect("failed to open system notifications example window");

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
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
