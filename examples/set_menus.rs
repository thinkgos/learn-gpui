//! 菜单栏示例：用 `cx.set_menus()` 给应用挂菜单栏，菜单项点一下就是分发一个动作（见 9.6）。
//!
//! 运行方式：`cargo run --example set_menus`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 窗口里只有一行居中文字 "Set Menus Example"；
//!   2. 真正的看点在**菜单栏**（macOS 上是屏幕顶部系统那一行）："set_menus" 菜单里依次是
//!      系统 Services 子菜单、分隔线、置灰项、可勾选项、"Mode" 子菜单、分隔线和 Quit。
//!
//! **NOTE**: 菜单栏**只在 macOS 显示**。Linux（含 Wayland）的平台层只是把菜单树存进内存，
//! `cx.get_menus()` 取得回来，但**没有**任何东西画它 —— 所以在 Ubuntu + niri/GNOME/KDE
//! 下看不到菜单栏是正常现象。想在 Linux 上触发这些动作，用快捷键（`cmd-` 在 Linux 映射到
//! Super 键，会和 niri 这类合成器的快捷键撞车，建议写 `ctrl-` 系列）或自己在窗口里画菜单栏
//! （点菜单项 `cx.dispatch_action(&Quit)`，见 9.6）。
//!
//! 关键概念：
//!   1. 菜单项就是「另一个动作触发器」：`MenuItem::action(标题, 动作)`，
//!      点菜单等价于分发动作，处理逻辑还是 `.on_action()` 那一份（见 9.6）；
//!   2. `gpui::NoAction` 是「什么都不做」的内置动作，专门用来做置灰占位菜单项；
//!   3. 勾选状态来自全局状态 `AppState`，切换后要**重新 set_menus** 菜单才会刷新。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Context, Global, Menu, MenuItem, SharedString, SystemMenuType, Window, WindowOptions,
    actions, div, prelude::*,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 窗口内容：只有一行字，用来证明「界面很闲，菜单栏很忙」。
struct SetMenus;

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for SetMenus {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div() // 创建一个 div 元素，类似 HTML 里的 <div>
            .flex() // 弹性布局：display: flex
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .size_full() // 宽高撑满窗口：width: 100%; height: 100%
            .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
            .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
            .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
            .text_color(gpui::black()) // 文字颜色：color: #000000
            .child("Set Menus Example")
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
        // 放一份全局状态：菜单的勾选状态存在这里（见下面的 AppState）
        cx.set_global(AppState::new());

        // 把应用弄到前台来（macOS 上菜单栏跟着应用走，不激活可能看不到）
        cx.activate(true);
        // 注册动作处理函数，好让菜单栏里的 `MenuItem::action` 能引用到它们（见 9.4）
        cx.on_action(quit);
        cx.on_action(toggle_check);
        // 添加菜单项
        set_app_menus(cx);
        // 打开窗口：内容就是那个只有一行字的 SetMenus
        cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| SetMenus {}))
            .unwrap();
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

// ===========================================================================
// 一、菜单背后的状态
// ===========================================================================

/// 视图模式：菜单勾选状态的取值。
#[derive(PartialEq)]
enum ViewMode {
    List,
    Grid,
}

impl ViewMode {
    /// 两种模式来回切。
    fn toggle(&mut self) {
        *self = match self {
            ViewMode::List => ViewMode::Grid,
            ViewMode::Grid => ViewMode::List,
        }
    }
}

/// 菜单标题直接用模式名，所以给它一个转 `SharedString` 的实现。
impl Into<SharedString> for ViewMode {
    fn into(self) -> SharedString {
        match self {
            ViewMode::List => "List",
            ViewMode::Grid => "Grid",
        }
        .into()
    }
}

/// 全局状态：菜单栏要读它来决定勾选，动作要改它来切换模式。
struct AppState {
    /// 当前视图模式。
    view_mode: ViewMode,
}

impl AppState {
    fn new() -> Self {
        Self {
            view_mode: ViewMode::List,
        }
    }
}

/// 标记成全局状态：`cx.global::<AppState>()` 就能读到它。
impl Global for AppState {}

// ===========================================================================
// 二、菜单栏与动作
// ===========================================================================

/// 把菜单栏装到应用上（`cx.set_menus`）。
///
/// 菜单项的动作是 `ToggleCheck` / `Quit`，勾选状态读全局 `AppState`，
/// 所以状态切换后要再调一次本函数刷新菜单。
fn set_app_menus(cx: &mut App) {
    let app_state = cx.global::<AppState>();
    cx.set_menus([Menu::new("set_menus").items([
        // 系统自带的 Services 子菜单（macOS 菜单栏的惯例项）
        MenuItem::os_submenu("Services", SystemMenuType::Services),
        MenuItem::separator(), // 分隔线
        // 置灰项：挂 `gpui::NoAction` 占位，再 .disabled(true) 置灰
        MenuItem::action("Disabled Item", gpui::NoAction).disabled(true),
        MenuItem::submenu(Menu::new("Disabled Submenu").disabled(true)), // 整个子菜单置灰
        MenuItem::separator(),
        // 可勾选项：勾选状态跟着 view_mode 走
        MenuItem::action("List Mode", ToggleCheck).checked(app_state.view_mode == ViewMode::List),
        MenuItem::submenu(
            Menu::new("Mode").items([
                // 标题用 ViewMode 转出来的 "List" / "Grid"，两个菜单项共用 ToggleCheck
                MenuItem::action(ViewMode::List, ToggleCheck)
                    .checked(app_state.view_mode == ViewMode::List),
                MenuItem::action(ViewMode::Grid, ToggleCheck)
                    .checked(app_state.view_mode == ViewMode::Grid),
            ]),
        ),
        MenuItem::separator(),
        MenuItem::action("Quit", Quit), // 菜单项就是动作触发器（见 9.6）
    ])]);
}

// 用 `actions!` 宏声明动作（见 9.1），也可以用 `#[derive(Action)]` 在结构体上派生。
actions!(set_menus, [Quit, ToggleCheck]);

/// 处理 `Quit` 动作：这是在 App 上注册的全局处理器（见 9.4）。
fn quit(_: &Quit, cx: &mut App) {
    println!("Gracefully quitting the application...");
    cx.quit();
}

/// 处理 `ToggleCheck` 动作：切换视图模式，然后重新装一遍菜单让勾选跟着变。
fn toggle_check(_: &ToggleCheck, cx: &mut App) {
    let app_state = cx.global_mut::<AppState>();
    app_state.view_mode.toggle();
    set_app_menus(cx);
}
