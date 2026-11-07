//! 不开窗口的纯逻辑示例：演示 GPUI 实体的创建、读写、事件订阅和通知，
//! 全程用 assert 验证数据流，没有界面。
//!
//! 运行方式：`cargo run --example ownership_post`
//! 界面分块：无窗口，程序跑完 assert 就退出。数据流分四步：
//!   1. 创建 counter 实体（count = 0）；
//!   2. 创建 subscriber 实体：构造时订阅 counter 的 Change 事件，初始值取 counter 的两倍；
//!   3. counter 加 2 并发出 Change { increment: 2 } 事件；
//!   4. assert 验证 subscriber 的值 = 初始 0×2 + 事件 2×2 = 4。

// 只有在编译目标是 wasm 时，才禁用标准的 `main` 入口（wasm 用下面的 `start` 函数启动）。
#![cfg_attr(target_family = "wasm", no_main)]

// 引入示例用的辅助模块（主要是加载字体）。
// `#[path = ...]` 告诉编译器这个模块的源文件在别的目录下。
#[path = "example_support/fonts.rs"]
mod example_support;

use gpui::{App, Context, Entity, EventEmitter, prelude::*};
// gpui_platform 提供跨平台的应用启动入口（桌面 / Web 各有实现）
use gpui_platform::application;

/// 状态容器：纯数据结构体也能做成实体，不实现 `Render` 就不会被画上屏。
struct Counter {
    /// 计数值。注意两个实体（subscriber 也是 Counter）共用这个结构体。
    count: usize,
}

/// 事件载荷：结构体或枚举都行，本身不用实现任何 trait（见第 13 章）。
struct Change {
    /// 增量
    increment: usize,
}

/// `EventEmitter` 是空的标记 trait：
/// 只声明 Counter 可以发射 Change 事件，没有方法要实现。
impl EventEmitter<Change> for Counter {}

/// 真正启动应用的函数（桌面和 wasm 共用）。
fn run_example() {
    // application().run(...) 启动应用主循环，并在闭包里做初始化。
    // 本示例不开窗口：注册完数据流、assert 验证完就结束。
    application().run(|cx: &mut App| {
        // 先加载字体（本示例用不到字体，保持和其他示例一致的骨架）；失败就退出。
        if !example_support::load_fonts(cx) {
            return;
        }

        // 创建第一个实体：cx.new 的闭包返回初始状态，整个表达式拿到 Entity<Counter> 句柄。
        // 数据本体由 GPUI 托管，句柄可以随意克隆，克隆的只是引用（见第 12 章）。
        let counter: Entity<Counter> = cx.new(|_cx| Counter { count: 0 });

        // 创建第二个实体。构造闭包里可以读别的实体、注册订阅（见第 12 章）。
        let subscriber = cx.new(|cx: &mut Context<Counter>| {
            // 订阅 counter 的 Change 事件（见第 13 章）。
            // 回调收四个参数：自己、发射者句柄、事件、cx；
            // 这里把事件里的增量翻倍记到自己身上。
            cx.subscribe(&counter, |this, _emitter, event, _cx| {
                this.count += event.increment * 2;
            })
            .detach(); // 不持有订阅句柄：detach 后订阅一直有效，直到 counter 被释放

            // 初始值 = counter 当前值的两倍（此刻 counter 是 0）。
            Counter {
                count: counter.read(cx).count * 2,
            }
        });

        // 更新 counter：可变借用，闭包同时拿到 &mut Counter 和它的 Context。
        counter.update(cx, |counter, cx| {
            counter.count += 2; // 0 → 2
            cx.notify(); // 通知 GPUI：状态变了（没有视图和观察者，这里只是演示标准写法）
            cx.emit(Change { increment: 2 }); // 发射事件，subscriber 的回调会把 2×2 记上去
        });

        // emit 不是当场执行回调：事件先进 effect 队列，到最外层 update 收尾时统一派发
        // （本例 counter.update 就是最外层 update，它返回时回调已经跑完）。
        // 所以这里读到的 subscriber.count = 初值 0×2 + 事件 2×2 = 4。
        assert_eq!(subscriber.read(cx).count, 4);
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
