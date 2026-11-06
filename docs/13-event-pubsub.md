# 13. 事件发布订阅

第 6 章讲的是 UI 事件(点击、鼠标、悬停):  由用户操作触发,GPUI按元素层级分发. 本章讲的是**实体(Entity)之间的自定义事件**:  一个实体定义并发射事件, 其他实体订阅后做出反应, 用来解耦业务逻辑层的通信.

## 13.1 EventEmitter trait: 事件发射器

事件本身只是普通的 `'static` 类型(枚举或结构体都行), 什么都不用实现. 要发射事件的**实体结构体**实现 `EventEmitter<事件类型>` 这个标记 trait, 声明"我能发这种事件":

```rust
use gpui::*;

/// 事件载荷: 结构体或枚举都可以
struct Change {
    increment: usize,
}

struct Counter {
    count: usize,
}

/// EventEmitter 是空的标记 trait:
/// 只声明 Counter 可以发射 Change 事件, 没有方法要实现
impl EventEmitter<Change> for Counter {}
```

一个实体可以发多种事件:  给每个事件类型写一个 `impl` 就行, 订阅方按事件类型分别订阅.

**NOTE**: 本仓库使用的GPUI里 EventEmitter 是手写的空实现, **没有** `#[derive(EventEmitter)]` 这个派生宏, 也不是"在事件枚举上派生" —— 实现写在**发射方实体**上, 事件类型自己不碰任何 trait.

## 13.2 cx.emit(): 发射事件

`cx.emit()` 只存在于实体自己的 `Context<T>` 上(要求 `T: EventEmitter<Evt>`), 通常写在实体的更新方法里, 和 `cx.notify()` 搭配:

```rust
impl Counter {
    fn increment(&mut self, delta: usize, cx: &mut Context<Self>) {
        self.count += delta;
        cx.notify(); // 通知 GPUI: 状态变了, 需要重新渲染
        cx.emit(Change { increment: delta }); // 发射事件, 通知所有订阅者
    }
}
```

也可以在 `entity.update(...)` 的闭包里直接发(完整示例 [`ownership_post.rs`](../examples/ownership_post.rs) 的写法):

```rust
counter.update(cx, |counter, cx| {
    counter.count += 2;
    cx.notify();
    cx.emit(Change { increment: 2 });
});
```

`emit()` 并不是"调用那一行立刻执行所有回调", 而是把事件排进 effect 队列, **本轮 update 结束时统一派发**(见 12.5). 所以在同一个闭包里 `emit()` 之后马上读订阅者状态, 可能还没更新.

## 13.3 cx.subscribe(): 订阅事件

`cx.subscribe()` 在订阅者的 `Context<T>` 上调用, 收一个发射者句柄和回调, 返回 `Subscription`. 回调是**四个参数** `(订阅者, 发射者句柄, &事件, cx)`:

| 方法 | 定义处 | 回调形状 |
| :--- | :--- | :--- |
| `cx.subscribe(&entity, h)` | `Context<T>` | `(\|this, emitter, event, cx\| ...)` |
| `cx.subscribe_self(h)` | `Context<T>` | `(\|this, event, cx\| ...)` 订阅自己 |
| `cx.subscribe_in(&entity, window, h)` | `Context<T>` | `(\|this, emitter, event, window, cx\| ...)` 回调里多拿一个 `Window` |
| `cx.subscribe(&entity, h)` | `App` | `(\|emitter, event, cx\| ...)` 不绑定订阅者实体 |

```rust
// 创建另一个实体, 在构造中订阅 counter 的事件(取自 ownership_post.rs)
let subscriber = cx.new(|cx: &mut Context<Counter>| {
    cx.subscribe(&counter, |subscriber, _emitter, event, _cx| {
        subscriber.count += event.increment * 2; // 收到事件后改自己的状态
    })
    .detach(); // 不持有订阅句柄: 订阅一直有效, 直到 counter 被释放

    Counter {
        count: counter.read(cx).count * 2,
    }
});
```

**Subscription 要保存住或 `.detach()`, 否则订阅立刻失效**:  它标了 `#[must_use]`, Drop 时自动退订. 两种存法:

- 存进结构体字段(常用 `_subscription: Subscription`):  组件释放时自动退订, 生命周期跟订阅者走.
- `.detach()`:  脱离句柄, 订阅持续到**被订阅的实体**释放为止, 适合"监听一次就不管了".

`Subscription::join(a, b)` 把两个订阅合成一个, 一起保存或一起 detach.

订阅自己的事件用 `subscribe_self`, 回调少一个发射者参数(发射者就是自己):

```rust
cx.subscribe_self(|this, event: &Change, _cx| {
    this.count += event.increment; // 自己 emit 的事件自己也能收到
});
```

💡 提示: 回调收的事件是 `&Evt` 借用(枚举要用 `*id` 这类方式取值拷出来), 第二个参数是发射者的 `Entity<T2>` 句柄, 用它能反查发射者状态.

## 13.4 多级事件传播(父子实体之间的事件流)

**NOTE**:GPUI的实体事件**没有** DOM 那样的父子冒泡 —— 事件只发给"订阅了该实体"的监听器, 不会自动沿实体层级向上传播. 所谓"多级传播", 是指**逐级订阅、逐级转发**:

```rust
// 中间层: 订阅底层实体, 加工后转发自己的事件
struct Tally {
    total: usize,
}
struct Tallied {
    total: usize,
}
impl EventEmitter<Tallied> for Tally {}

impl Tally {
    fn bind(&mut self, counter: &Entity<Counter>, cx: &mut Context<Self>) {
        // 第一跳: Tally 订阅 Counter
        cx.subscribe(counter, |tally, _emitter, event, cx| {
            tally.total += event.increment;
            // 第二跳: 再用自己的事件发给订阅了 Tally 的上层
            cx.emit(Tallied { total: tally.total });
        })
        .detach();
    }
}
```

顶层只要订阅 `Tally` 就能拿到加工后的结果, 不必知道 `Counter` 的存在 —— 这正是事件解耦的意义. 如果中间层不需要加工/过滤, 让顶层直接订阅底层实体即可, 省掉转发.

## 13.5 事件生命周期

一次完整的事件流走这几步:

1. **定义**:  事件是普通类型;  在发射方实体上写 `impl EventEmitter<事件>`.
2. **发射**:  `cx.emit()` 把事件排进 effect 队列, 不立刻回调.
3. **派发**:  本轮 update 结束时GPUI统一 flush effect, 按入队顺序把每个事件发给该实体的所有订阅者.
4. **处理**:  订阅回调里改自己的状态, 通常再调 `cx.notify()` 触发重新渲染.
5. **清理**:  `Subscription` 被 Drop(或组件释放)即退订;  发射实体被释放时, 挂在它身上的监听器一并移除.

**NOTE**: 事件是**延迟派发**的 —— `emit()` 那一刻订阅者收不到, 要等本轮 `update` 结束. 在 `counter.update(cx, ...)` 返回后再读订阅者, 才能看到事件处理完的最新状态.

```rust
counter.update(cx, |counter, cx| {
    counter.count += 2;
    cx.emit(Change { increment: 2 });
});
// update 返回时 effect 已经 flush 完, 订阅者状态是新的(ownership_post.rs 的断言)
assert_eq!(subscriber.read(cx).count, 4);
```

💡 提示: EventEmitter 事件与 UI 事件的区别 —— UI 事件(`on_click` 等)由用户操作触发、GPUI 分发;  EventEmitter 事件由业务代码自己定义和发射. 典型组合:  在 `on_click` 回调里调 `cx.emit()`, 把 UI 事件转成业务事件再广播给其他实体.

## 13.6 事件速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **事件声明** | `impl EventEmitter<E> for T {}` | 空实现, 标记实体 T 可发射事件 E |
| **发射** | `cx.emit(event)` | 仅 `Context<T>`(`T: EventEmitter<E>`), 排队后本轮 update 结束派发 |
| **订阅** | `cx.subscribe(&entity, h)` | 订阅另一实体, 回调 4 参:  `(this, emitter, &event, cx)` |
| **订阅(变体)** | `cx.subscribe_self(h)` `cx.subscribe_in(&entity, window, h)` | 订阅自己/回调里多拿 `Window` |
| **订阅(App 级)** | `cx.subscribe(&entity, h)` | `App` 上的版本, 回调 3 参:  `(emitter, &event, cx)` |
| **订阅生命周期** | `Subscription` `.detach()` `Subscription::join(a, b)` | Drop 退订;  detach 到发射实体释放为止 |
| **实体释放钩子** | `cx.on_release(h)` `cx.observe_release(&entity, h)` | 自己/别的实体被释放时收尾 |

完整示例:  [`cargo run --example ownership_post`](../examples/ownership_post.rs)(不开窗口的纯逻辑示例, 用 assert 演示"发射 → 订阅 → 状态同步"的事件流).

## 13.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`ownership_post`](../examples/ownership_post.rs) | `EventEmitter` + `emit()` + `subscribe()`, assert 演示"发射 → 订阅 → 状态同步"的事件流(见 13.1–13.3) |
