//! 实体跨窗口迁移示例：实体用 `_in` 系列 API（`spawn_in`/`subscribe_in`）注册回调，
//! 然后被一次点击整个搬进新窗口。重点演示：搬家之后派发的回调拿到的窗口，
//! 是实体「当前」所在的窗口，而不是注册回调时那个旧窗口。
//!
//! 运行方式：`cargo run --example move_entity_between_windows`
//! 界面分块：一个 500x500 的窗口，从上到下依次是问候语、当前窗口 id、
//! tick 计数、迁移计数，最后是「Move me to a new window」按钮。
//! 点按钮后同一个实体出现在新开的窗口里，旧窗口关闭；
//! 实体里每秒一次的 tick 任务照常跑，日志和界面上的窗口 id 都换成新窗口的。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use std::time::Duration; // tick 任务的定时器：每 1 秒醒一次

use gpui::{
    App, AppContext as _, Bounds, Context, EventEmitter, MouseButton, Render, SharedString,
    Subscription, Task, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 「把我挪到新窗口」事件：空结构体只当事件类型用，不携带数据。
struct MoveToNewWindow;

/// 示例组件：界面显示两个计数，用来观察回调有没有跟着实体走。
struct HelloWorld {
    /// 要显示的文本内容。
    text: SharedString,
    /// tick 任务每秒 +1；打印时带上「当时」的窗口 id。
    tick_count: u32,
    /// 实体每被搬进一个新窗口就 +1。
    move_count: u32,
    /// 异步任务句柄：`Task` 一被丢弃任务就取消，所以要存住。
    _tasks: Vec<Task<()>>,
    /// 订阅句柄：`Subscription` 一被丢弃就退订，所以同样要存住。
    _subscriptions: Vec<Subscription>,
}

/// `EventEmitter` 是空的标记 trait：
/// 只声明 HelloWorld 可以发射 MoveToNewWindow 事件（见第 13 章）。
impl EventEmitter<MoveToNewWindow> for HelloWorld {}

impl HelloWorld {
    /// 构造组件，顺手注册两个「跟实体走」的回调：每秒一次的 tick 任务、
    /// 自订阅的 MoveToNewWindow 处理器。两者都走 `_in` 系列 API，
    /// 回调里拿到的 `Window` 是派发时刻实体所在的窗口。
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // 取自己实体的强句柄：构造还没结束，下面自订阅要用它。
        let self_entity = cx.entity();

        // spawn_in: 在「这个窗口」的上下文里起异步任务。
        // 闭包收 (自己实体的弱句柄, AsyncWindowContext)：弱句柄不阻止实体释放，
        // 所以更新可能失败，任务里要处理 Err。
        let task = cx.spawn_in(window, async move |this, cx| {
            loop {
                // 在后台执行器上睡 1 秒，不堵主线程。
                cx.background_executor().timer(Duration::from_secs(1)).await;
                // update_in: 更新实体状态，闭包比 update 多收一个 &mut Window。
                // 这个 window 按「实体当前所在的窗口」解析 —— 实体搬家后就换成新窗口，
                // 这正是本示例要验证的行为（弱引用更新见第 12 章）。
                let result = this.update_in(cx, |this, window, _cx| {
                    this.tick_count += 1;
                    println!(
                        "tick #{} fired in entity's current window {}",
                        this.tick_count,
                        window.window_handle().window_id().as_u64(),
                    );
                    _cx.notify();
                });
                // 返回 Err 说明实体已经释放，任务收工。
                if let Err(err) = result {
                    println!("tick task giving up: {err}");
                    return;
                }
            }
        });

        // subscribe_in: 订阅自己发的 MoveToNewWindow 事件（见第 13 章）。
        // 比 subscribe 的回调多收一个 &mut Window，同样指向实体「当前」的窗口。
        let subscription = cx.subscribe_in::<_, MoveToNewWindow>(
            &self_entity,
            window,
            move |this, _emitter, _event, window, cx| {
                // 打印事件派发时的窗口 id：点击发生在旧窗口，所以迁移前这行是旧窗口的 id。
                let entered_window_id = window.window_handle().window_id().as_u64();
                println!(
                    "MoveToNewWindow handler fired in entity's current window {entered_window_id}",
                );

                this.move_count += 1;
                cx.notify(); // 通知 GPUI：状态变了，需要重新渲染

                let entity = cx.entity(); // 再取一份实体句柄，交给新窗口
                let old_window = window.window_handle(); // 记下旧窗口句柄，稍后关掉它
                // 开新窗口 + 关旧窗口推迟到本轮事件派发结束之后（defer）：
                // 现在正处于旧窗口的派发过程里，不能当场把它自己关掉。
                cx.defer(move |cx| {
                    // 新窗口的位置尺寸：屏幕中央的 500x500。
                    let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
                    // open_window 的第二个参数是「造根视图」的闭包，返回值必须是 Entity。
                    // 这里把现有的 entity 原样交出去：实体连同它的状态、任务、订阅
                    // 一起搬进新窗口，成为新窗口的根视图。
                    cx.open_window(
                        WindowOptions {
                            window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                            ..Default::default() // 其余选项都用默认值
                        },
                        move |_, _| entity,
                    )
                    .expect("failed to open new window");
                    // 关掉旧窗口：remove_window 只是把它标记成「要关」。
                    old_window
                        .update(cx, |_, window, _| window.remove_window())
                        .ok();
                });
            },
        );

        Self {
            text: "World".into(),
            tick_count: 0,
            move_count: 0,
            _tasks: vec![task],                 // 存住任务句柄
            _subscriptions: vec![subscription], // 存住订阅句柄
        }
    }
}

/// 实现 `Render`：告诉 GPUI 这个组件的界面由哪些元素组成。
impl Render for HelloWorld {
    /// window 参数就是实体「当前」所在的窗口，把它显示出来方便观察搬家效果。
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let window_id = window.window_handle().window_id().as_u64();

        div() // 创建一个 div 元素，类似 HTML 里的 <div>
            .flex() // 弹性布局：display: flex
            .flex_col() // 改成纵向排列：flex-direction: column
            .gap_3() // 子元素之间的间距：gap: 0.75rem (12px)
            .bg(rgb(0x505050)) // 背景色：background-color: #505050
            .size(px(500.0)) // 宽高都是 500 像素：width: 500px; height: 500px
            .justify_center() // 主轴居中（这里是垂直居中）：justify-content: center
            .items_center() // 交叉轴居中（这里是水平居中）：align-items: center
            .text_xl() // 字号加大：font-size: 1.25rem; line-height: 1.75rem
            .text_color(rgb(0xffffff)) // 文字颜色：color: #ffffff
            .child(format!("Hello, {}!", self.text))
            // 整个示例最有用的一行：实体此刻在哪个窗口里渲染
            .child(format!("Rendering in window: {window_id}"))
            .child(format!("Ticks observed by entity: {}", self.tick_count))
            .child(format!("Moves observed by entity: {}", self.move_count))
            // 迁移按钮
            .child(
                div()
                    .px_4() // 左右内边距：padding: 0 1rem
                    .py_2() // 上下内边距：padding: 0.5rem 0
                    .bg(rgb(0x4040ff)) // 背景色：background-color: #4040ff
                    .rounded_md() // 中等圆角：border-radius: 0.375rem (6px)
                    .child("Move me to a new window")
                    // 鼠标左键按下就发事件，搬家动作由 subscribe_in 注册的处理器接手
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_this, _, _window, cx| {
                            // listener 闭包收四个参数：自己的状态、事件、window、cx（见第 12 章）
                            cx.emit(MoveToNewWindow); // 发射事件，让订阅者去搬家
                        }),
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
        // 计算窗口位置：让 500x500 的窗口出现在屏幕中央。
        // None 表示不指定显示在哪个显示器上，用默认的。
        let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
        // 打开第一个窗口：根视图是 HelloWorld，
        // tick 任务和自订阅都在它的构造函数里注册。
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)), // 窗口用上面算出的位置尺寸
                ..Default::default()                                 // 其余选项都用默认值
            },
            |window, cx| cx.new(|cx| HelloWorld::new(window, cx)),
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
    gpui_platform::web_init(); // Web 环境需要先做一次初始化
    run_example();
}
