# 12. Entity 状态管理

GPUI 用实体(Entity)承载所有可变状态: 视图的字段、列表的数据、拖拽的载荷(见第 10 章)都住在实体里. 本章讲实体的创建、读写、通知、观察和所有权规则 —— 这是理解GPUI数据流的基础.

## 12.1 Entity:GPUI的核心状态容器

`Entity<T>` 是GPUI托管状态的类型安全句柄: 数据本体由GPUI统一持有, 你手里的句柄只负责引用它. 三个最基本的操作:

| 操作 | 方法 | 说明 |
| :--- | :--- | :--- |
| 创建 | `cx.new(\|cx\| T { ... })` | 构造 `T` 并返回 `Entity<T>` |
| 读取 | `entity.read(cx)` | 不可变借用, 拿到 `&T` |
| 更新 | `entity.update(cx, \|data, cx\| ...)` | 可变借用, 拿到 `&mut T` 和这个实体的 `Context<T>` |

纯数据结构体、自定义视图(实现 `Render`, 见第 1 章)都可以做成实体, 区别只是视图会被窗口渲染出来.

## 12.2 创建和更新 Entity

```rust
pub struct Counter {
    pub count: i32,
}

// 创建: 闭包返回初始状态, cx.new 返回句柄
let counter: Entity<Counter> = cx.new(|_cx| Counter { count: 0 });

// 读取: read 收 &App, 返回 &Counter
let current = counter.read(cx).count;

// 更新: 闭包拿到 &mut Counter 和该实体的 Context<T>
counter.update(cx, |counter, cx| {
    counter.count += 1;
    cx.notify(); // 状态变了要通知, 否则界面不动(见 11.4)
});
```

`Entity` 上与读写相关的方法:

| 方法 | 说明 |
| :--- | :--- |
| `entity.read(cx)` | 不可变借用 `&T`,`cx`收 `&App` |
| `entity.read_with(cx, f)` | 不可变借用并套一层闭包: `f(&T, &App) -> R` |
| `entity.update(cx, f)` | 可变借用并套一层闭包: `f(&mut T, &mut Context<T>) -> R` |
| `entity.update_in(cx, f)` | 同上, 闭包多收一个 `&mut Window`(视图上下文) |
| `entity.write(cx, value)` | 整体替换值, 自动 `notify` |
| `cx.reserve_entity()` + `cx.insert_entity()` | 先占位拿句柄/实体 id, 稍后再填内容(构建互相引用时用) |

**NOTE**: 创建实体用 `cx.new(...)`, 生产代码里**没有** `cx.new_entity()` 这个方法(只在测试专用的 `TestAppContext` 上有). `new` 是 `AppContext` trait 的方法, `App`、`Context<T>`、`AsyncApp` 上都能调 —— 也就是说可以在一个实体的构造闭包里直接创建另一个实体.

## 12.3 cx.observe(): 观察状态变化

`cx.observe()` 订阅另一个实体的「状态变了」通知: 只要被观察的实体调了 `cx.notify()`, 回调就会被触发. 回调收三个参数: 观察者自己的状态、被观察实体的句柄、观察者的 `Context`:

```rust
pub struct Parent {
    child: Entity<Child>,
    child_data: String,
}

impl Parent {
    fn new(child: Entity<Child>, cx: &mut Context<Self>) -> Self {
        cx.observe(&child, |parent, child_entity, cx| {
            // 被观察的 child 一调 notify 就会进到这里
            parent.child_data = child_entity.read(cx).value.clone(); // 同步它的数据
            cx.notify(); // 自己也变了, 通知重绘
        })
        .detach(); // 或存进字段, 见下文

        Parent { child, child_data: String::new() }
    }
}
```

`observe` 返回 `Subscription`: 它被丢弃时订阅就取消, 所以要么 `.detach()`(一直有效, 直到被观察实体释放), 要么存进实体字段里续命:

```rust
// 存字段: 订阅活多久由实体决定
_todos_subscription: cx.observe(todos, |_, _, cx| cx.notify()),
```

同族方法还有 `cx.observe_self()`(观察自己)和 `cx.observe_release()`(实体被释放时收尸).

💡 提示: 想发「带数据的事件」而不只是「状态变了」, 用 `EventEmitter` + `cx.subscribe()`/`cx.emit()`: 被观察方 `cx.emit(事件)`, 订阅方收到 `(&mut 自己, 发送者句柄, &事件, cx)`. [`examples/ownership_post.rs`](../examples/ownership_post.rs) 里两个都有.

## 12.4 cx.notify(): 通知重新渲染

`cx.notify()` 告诉 GPUI「这个实体的状态变了」, 引发一串连锁反应:

- 注册在它身上的观察者(`cx.observe` 等)被触发, 由观察者决定要不要同步/再 `notify`;
- 引用它的视图被标记为待重绘, 窗口在下一帧重新构建受影响的元素树.

**NOTE**: `entity.update(cx, ...)` 改字段**不会**自动重绘 —— 忘了调 `cx.notify()` 界面就不会变, 这是最常见的新手坑. 个别鼠标回调(如 `on_drop`)看起来"没 notify 也刷新了", 那是事件派发顺带触发的刷新, 别依赖这种巧合.

## 12.5 cx.listener(): 类型安全的回调

GPUI 事件回调的标准形状是 `Fn(&E, &mut Window, &mut App)`, 里面摸不到自己的状态. `cx.listener()` 把它补成「先借出自己实体的状态再执行」的闭包, 闭包收四个参数: 自己的状态、事件、window、自己的 `Context`:

```rust
div()
    .px_4() // 内边距: padding: 0 1rem
    .py_2() // 内边距: padding: 0.5rem 0
    .bg(rgb(0x4040ff)) // 背景色: background-color: #4040ff
    .rounded_md() // 圆角: border-radius: 6px
    .child("Move me to a new window")
    .on_mouse_down(
        MouseButton::Left,
        cx.listener(|_this, _, _window, cx| {
            // 第一个参数是 &mut HelloWorld, 可以随便读写自己的字段
            cx.emit(MoveToNewWindow); // 在自己的上下文里发事件
        }),
    )
```

返回值正好是事件回调的形状, 所以能直接递给 `on_click`、`on_drop`、`on_mouse_down` 等一切元素方法.

**NOTE**: listener 闭包收**四个**参数(this、事件、window、cx), **没有** `|view, event, cx|` 三参数写法 —— 那是旧版GPUI的签名, 现在编译不过. 回调里也要改状态别忘了 `cx.notify()`.

## 12.6 所有权模型

GPUI 基于 Rust 所有权设计了自己的托管方式:

- **数据本体唯一**: `T` 的所有权在GPUI的实体表里, 谁都不能直接搬走;
- **`entity.clone()` 克隆的是句柄**, 只加引用计数, 不复制数据(类似 `Rc`);
- **读写经由句柄**: `read` 拿 `&T`(可多个并存), `update` 拿 `&mut T` + `Context<T>`(独占);
- **冲突是运行时检查的**: 同一实体的 `update` 未结束就再 `read`/`update` 会 panic(双租约), 并打出调试位置.

```rust
let entity1 = cx.new(|_| MyData { value: 42 });
let entity2 = entity1.clone(); // 克隆句柄, 数据只有一份

entity2.update(cx, |data, cx| {
    data.value = 100; // entity1 和 entity2 指向同一份数据
    cx.notify();
});

assert_eq!(entity1.read(cx).value, 100); // 两个句柄看到同一份数据
```

**NOTE**: "借用冲突编译期就能抓到、不会运行时 panic" 这个说法**不成立**. 闭包内的 `&T`/`&mut T` 当然受编译器保护, 但跨句柄的读写冲突是GPUI**运行时**租约检查的, 嵌套借用同一实体照样 panic. 真正的线程安全另有一套: 读写都收敛到主线程, 异步任务经 `AsyncApp` 回来更新(见 11.7).

## 12.7 弱引用 WeakEntity

`WeakEntity<T>` 是不阻止实体释放的弱句柄, 用来打破「订阅/异步任务 ↔ 实体」之间的引用循环. `entity.downgrade()` 得到弱引用, `cx.weak_entity()` 拿自己实体的弱引用(如 `Context::spawn` 的异步闭包里收到的就是它):

```rust
// 降级成弱引用: 不增加强引用计数, 不阻止实体被释放
let weak: WeakEntity<Counter> = entity.downgrade();

// 使用前升级: 实体可能已经没了, 所以返回 Option
if let Some(entity) = weak.upgrade() {
    entity.update(cx, |data, cx| {
        data.count += 1;
        cx.notify();
    });
}

// 一步到位的弱引用更新: 实体已释放时返回 Err 而不是 panic
weak.update(cx, |data, cx| {
    data.count += 1;
    cx.notify();
})?;
```

| 方法 | 说明 |
| :--- | :--- |
| `entity.downgrade()` | 强句柄降级为 `WeakEntity<T>` |
| `weak.upgrade()` | 升回 `Entity<T>`, 实体已释放则 `None` |
| `weak.update(cx, f)`/`weak.read_with(cx, f)` | 弱引用版读写, 返回 `Result` |
| `weak.is_upgradable()` | 实体是否还活着 |
| `WeakEntity::new_invalid()` | 永远升不了级的占位弱引用 |

**NOTE**: `upgrade()` **没有** cx 参数, 直接返回 `Option<Entity<T>>`; 检查存活用 `is_upgradable()`, **没有** `weak.is_valid(cx)` 这个方法.

什么时候用弱引用: 订阅回调、长期存活的异步任务、互相引用的父子实体. 反过来, 短命闭包里直接 `clone()` 强句柄就好.

## 12.8 Entity vs Rc<RefCell<T>> 的优势

自己拿 `Rc<RefCell<T>>` 也能攒状态, 但GPUI的实体和 UI 系统是一体的:

| 维度 | `Rc<RefCell<T>>` | `Entity<T>` |
| :--- | :--- | :--- |
| UI 联动 | 手动通知、手动重绘 | `cx.notify()` 直接连上观察者和渲染 |
| 借用冲突 | `borrow_mut` 运行时 panic | 运行时租约检查, panic 带类型名和位置 |
| 生命周期 | 引用计数归零就丢, 无处挂清理逻辑 | 释放时可挂 `on_release`/`observe_release` 回调 |
| 弱引用 | `Weak<T>` + `upgrade()` | 内建 `WeakEntity`, 读写直接返回 `Result` |
| 异步/跨窗口 | 自己套 `Arc<Mutex<>>` 传数据 | 句柄可克隆传递, 更新经 `AsyncApp` 回主线程, 无需加锁 |
| 类型安全 | 裸数据 | `EntityId` + 类型参数, 可按类型派发(拖拽的 TypeId 匹配就是这么做的) |

一句话: `Entity` 不只是容器, 它是把「状态、通知、渲染、生命周期」缝在一起的句柄, 写GPUI程序几乎所有状态都该住在实体里.

## 12.9 Entity 方法速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **创建** | `cx.new()` `cx.reserve_entity()` `cx.insert_entity()` | 创建/先占位后插入 |
| **读取** | `entity.read(cx)` `entity.read_with(cx, f)` | 不可变借用 |
| **更新** | `entity.update(cx, f)` `entity.update_in(cx, f)` `entity.write(cx, v)` | 可变借用/带 window/整体替换 |
| **通知** | `cx.notify()` | 通知观察者并触发重绘 |
| **观察** | `cx.observe()` `cx.observe_self()` `cx.observe_release()` | 监听 notify/自己/释放 |
| **事件** | `cx.subscribe()` `cx.emit()` | 类型化事件(`EventEmitter`) |
| **回调** | `cx.listener()` | 事件回调里读写自己的状态 |
| **句柄** | `entity.clone()` `entity.entity_id()` `cx.entity()` | 克隆句柄/取 id/取自己的强句柄 |
| **弱引用** | `entity.downgrade()` `weak.upgrade()` `weak.update()` `weak.is_upgradable()` | 降级/升级/弱更新/查存活 |
| **订阅** | `Subscription::detach()` 或存进字段 | 丢弃即取消, 不丢就一直有效 |

完整示例: [`cargo run --example ownership_post`](../examples/ownership_post.rs)(创建、读写、订阅、通知), [`cargo run --example move_entity_between_windows`](../examples/move_entity_between_windows.rs)(listener、订阅、实体跨窗口迁移).

## 12.10 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`ownership_post`](../examples/ownership_post.rs) | 不开窗口的纯逻辑示例: 创建、读写、订阅、通知, 全程用 assert 验证(见 12.2–12.4) |
| [`move_entity_between_windows`](../examples/move_entity_between_windows.rs) | listener、订阅、实体跨窗口迁移: 点击把实体挪到新窗口, `_in` 回调仍派发到它当前所在的窗口(见 12.5) |
