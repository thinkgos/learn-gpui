//! Tab 导航示例：演示 `.tab_index()` / `.tab_stop()` / `.tab_group()` 怎么排 Tab 序（见 8.3）。
//!
//! 运行方式：`cargo run --example tab_stop`
//! 同样同时支持桌面和 wasm 两套入口。
//!
//! 界面分块：
//!   1. 顶部一行提示：显示刚按的是 Tab 还是 Shift-Tab；
//!   2. 五行焦点条目：tab_index 1/2/3 的三个 tab 站点、一个默认句柄
//!      （tab_stop: false，Tab 到不了，半透明显示）、一个和第二行同为
//!      tab_index 2 的站点（同号按元素出现顺序排）；
//!   3. 两个按钮：用元素版 `.tab_index(4)` / `.tab_index(5)` 排进 Tab 序；
//!   4. 两个 tab group：`.tab_group()` 把子树划成一组，按钮上标的
//!      [组序号, 组内序号] 就是 Tab 序里的路径。
//!
//! 关键点（见 8.3）：Tab 序按 (tab_index, 出现顺序) 排；`.tab_stop(false)`
//! 的元素保住序位但 Tab 停不上去（只能程序聚焦）。
//!
//! 注释格式沿用 hello_world.rs：「中文含义 + 等价 CSS」。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{
    App, Bounds, Context, Div, ElementId, FocusHandle, KeyBinding, SharedString, Stateful, Window,
    WindowBounds, WindowOptions, actions, div, prelude::*, px, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

// 定义两个动作（不带数据的空动作）：Tab 前进、Shift-Tab 后退。
// 按键绑定在 run_example 里做（见第 9 章）。
actions!(example, [Tab, TabPrev]);

/// 示例组件：五行条目的焦点句柄 + 顶部提示文字。
struct Example {
    /// 根元素的焦点句柄：启动时聚焦它，Tab / Shift-Tab 动作才会派发到下面的 on_action。
    focus_handle: FocusHandle,
    /// 五行条目的焦点句柄，Tab 顺序设置写在构造里。
    items: Vec<FocusHandle>,
    /// 顶部提示文字，跟着按键 / 点击更新。
    message: SharedString,
}

impl Example {
    /// 构造五个条目的焦点句柄，Tab 顺序设置都在这里做（句柄版，见 8.3）。
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let items = vec![
            cx.focus_handle().tab_index(1).tab_stop(true), // 第 1 站：tab_index 1
            cx.focus_handle().tab_index(2).tab_stop(true), // 第 2 站：tab_index 2
            cx.focus_handle().tab_index(3).tab_stop(true), // 第 3 站：tab_index 3
            cx.focus_handle(), // 默认句柄：tab_index 0、tab_stop false —— Tab 到不了
            cx.focus_handle().tab_index(2).tab_stop(true), // 又一个 tab_index 2：同号按元素出现顺序排
        ];

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx); // 启动时把焦点给根元素（见 8.1）

        Self {
            focus_handle,
            items,
            message: SharedString::from("Press `Tab`, `Shift-Tab` to switch focus."),
        }
    }

    /// Tab：在 Tab 序里让焦点前进一位（见 8.3）。
    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
        self.message = SharedString::from("You have pressed `Tab`.");
    }

    /// Shift-Tab：在 Tab 序里让焦点后退一位。
    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
        self.message = SharedString::from("You have pressed `Shift-Tab`.");
    }
}

/// 实现 `Render`：五行焦点条目 + 两个普通按钮 + 两个 tab group。
impl Render for Example {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 「当前聚焦的 tab 站点」套的蓝框样式，函数直接当样式回调传给 .focus() / .when()
        fn tab_stop_style<T: Styled>(this: T) -> T {
            this.border_3().border_color(gpui::blue()) // 3px 蓝框：border-width: 3px; border-color: #0000ff
        }

        // 底部按钮的底子：黑底白字的圆角条，聚焦时套蓝框样式
        fn button(id: impl Into<ElementId>) -> Stateful<Div> {
            div()
                .id(id) // 元素唯一 id：交互方法要求元素带 id
                .h_10() // 高度：height: 2.5rem (40px)
                .flex_1() // 平分剩余空间：flex: 1 1 0%
                .flex() // 弹性布局：display: flex
                .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                .border_1() // 1 像素边框：border-width: 1px
                .border_color(gpui::black()) // 边框颜色：border-color: #000000
                .bg(gpui::black()) // 背景色：background-color: #000000
                .text_color(gpui::white()) // 文字颜色：color: #ffffff
                .focus(tab_stop_style) // 聚焦样式：持有焦点就套蓝框（函数当样式回调用）
                .shadow_sm() // 小阴影：box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05)
        }

        div()
            .id("app")
            .track_focus(&self.focus_handle) // 绑定焦点句柄：根元素可聚焦，下面的 on_action 才收得到动作
            .on_action(cx.listener(Self::on_tab)) // 动作沿焦点路径派发：Tab 前进
            .on_action(cx.listener(Self::on_tab_prev)) // Shift-Tab 后退
            .size_full() // 撑满窗口：width: 100%; height: 100%
            .flex() // 弹性布局：display: flex
            .flex_col() // 纵向排列：flex-direction: column
            .p_4() // 四周内边距：padding: 1rem (16px)
            .gap_3() // 子元素之间的间距：gap: 0.75rem (12px)
            .bg(gpui::white()) // 背景色：background-color: #ffffff
            .text_color(gpui::black()) // 文字颜色：color: #000000
            .child(self.message.clone()) // 顶部提示文字
            .children(
                self.items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(ix, item_handle)| {
                        div()
                            .id(("item", ix))
                            .track_focus(&item_handle) // 绑定这一行的焦点句柄
                            .h_10() // 高度：height: 2.5rem (40px)
                            .w_full() // 宽度撑满：width: 100%
                            .flex() // 弹性布局：display: flex
                            .justify_center() // 主轴居中（这里是水平居中）：justify-content: center
                            .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                            .border_1() // 1 像素边框：border-width: 1px
                            .border_color(gpui::black()) // 边框颜色：border-color: #000000
                            .when(
                                // 既是 tab 站点、又正持有焦点，才套蓝框：
                                // tab_stop 是句柄上的布尔开关（GPUI 没有独立的 `TabStop` 类型）
                                item_handle.tab_stop && item_handle.is_focused(window),
                                tab_stop_style,
                            )
                            .map(|this| match item_handle.tab_stop {
                                // 按 tab_stop 分两种长相：能停 Tab 的显示序号，不能的半透明标注
                                true => this
                                    .hover(|this| this.bg(gpui::black().opacity(0.1))) // 悬停时铺一层 10% 黑底：类似 :hover
                                    .child(format!("tab_index: {}", item_handle.tab_index)),
                                false => this.opacity(0.4).child("tab_stop: false"), // 整体 40% 不透明度：opacity: 0.4
                            })
                    }),
            )
            .child(
                div() // 两个普通按钮：演示元素版的 tab 设置
                    .flex() // 弹性布局：display: flex
                    .flex_row() // 横向排列：flex-direction: row
                    .gap_3() // 按钮之间的间距：gap: 0.75rem (12px)
                    .items_center() // 交叉轴居中（这里是垂直居中）：align-items: center
                    .child(
                        button("el1")
                            .tab_index(4) // 元素版 tab 设置（上面 items 用的是句柄版，效果一致）
                            .child("Button 1")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.message = "You have clicked Button 1.".into();
                                cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                            })),
                    )
                    .child(
                        button("el2")
                            .tab_index(5) // 元素版 tab 设置
                            .child("Button 2")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.message = "You have clicked Button 2.".into();
                                cx.notify(); // 通知 GPUI：状态变了，需要重新渲染
                            })),
                    ),
            )
            .child(
                div() // tab group 1（见 8.3）
                    .id("group-1")
                    .tab_index(6) // 组序号：子元素的 Tab 序路径是 [6, 组内序号]
                    .tab_group() // 把子树划成一组：子元素的 tab_index 只在组内比较（只有元素版，没有句柄版）
                    .tab_stop(false) // 组本身不是 Tab 站点：占住序位，但 Tab 停不到组上
                    .child(
                        button("group-1-button-1")
                            .tab_index(1) // 组内序号：Tab 序路径 [6, 1]
                            .child("Tab index [6, 1]"),
                    )
                    .child(
                        button("group-1-button-2")
                            .tab_index(2) // 组内序号：Tab 序路径 [6, 2]
                            .child("Tab index [6, 2]"),
                    )
                    .child(
                        button("group-1-button-3")
                            .tab_index(3) // 组内序号：Tab 序路径 [6, 3]
                            .child("Tab index [6, 3]"),
                    ),
            )
            .child(
                div() // tab group 2：组内序号和 group 1 相同，靠组序号 7 排在它后面
                    .id("group-2")
                    .tab_index(7) // 组序号：子元素的 Tab 序路径是 [7, 组内序号]
                    .tab_group() // 把子树划成一组
                    .tab_stop(false) // 组本身不是 Tab 站点
                    .child(
                        button("group-2-button-1")
                            .tab_index(1) // 组内序号：Tab 序路径 [7, 1]
                            .child("Tab index [7, 1]"),
                    )
                    .child(
                        button("group-2-button-2")
                            .tab_index(2) // 组内序号：Tab 序路径 [7, 2]
                            .child("Tab index [7, 2]"),
                    )
                    .child(
                        button("group-2-button-3")
                            .tab_index(3) // 组内序号：Tab 序路径 [7, 3]
                            .child("Tab index [7, 3]"),
                    ),
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
        // 按键绑定（见第 9 章）：tab 前进、shift-tab 后退。
        // 第三个参数是上下文约束，None 表示全局生效。
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);

        // 计算窗口位置：让 800x600 的窗口出现在屏幕中央。
        // None 表示不指定显示在哪个显示器上，用默认的。
        let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default()                                 // 其余选项都用默认值
            },
            |window, cx| cx.new(|cx| Example::new(window, cx)),
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
