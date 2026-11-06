# 2. 核心概念

| 概念 | 说明 |
| -- | -- |
| `App` | GPUI的全局单例和核心管理器, 代表了整个桌面应用程序的运行内核, 并且只在主线程使用 |
| `Window` | 操作系统的原生窗口实体 |
| `Context` | GPUI的灵魂, 可以直观地理解为 **(&mut App, 当前实体/窗口引用)** 的安全包装, GPUI通过传递`cx`(`Context`) 来让你安全的修改状态, 触发重绘或派发任务 |
| `Entity` | GPUI内存中被框架托管的强类型状态容器(类似于其他框架里的 `Store Model`, 或者ECS架构中的实体概念) |
| `Element` | 界面上可见/可交互的单元(`div()` 的产物), 由 `render` 每帧构建的一次性值, 用完即弃 |
| `Global` | 跨窗口的应用级单例状态(主题, 设置, 连接池等) |
| `Render/RenderOnce Trait` | GPUI中最核心的`Trait`. 当一个 `Entity` 实现了 `Render` trait 时, 它就从一个普通的数据实体升级成了"视图(`View`)"; 实现 `RenderOnce` 的只是无状态组件, 不构成视图 |

核心数据流:

1. `App` 启动整个应用并维持生命周期.
2. `App` 打开一个 `Window`.
3. 你通过 `Context` 创建了一个结构体数据, 它成为了一个 `Entity`.
4. 这个结构体实现了 `Render Trait`, 因此它成为了一个 `View`,GPUI能够调用它的 `render` 函数画出界面.
5. 当用户点击按钮时, 事件触发 `Context` 修改了 `Entity` 的内部数据, 并调用 `cx.notify()`.
6. `Context` 联动 `App` 和 `Window` 触发重新渲染, 调用 `Render::render`, 屏幕画面随之更新.

💡 提示: 记住一句话 -- **GPUI不监视你的字段**. 改了数据不调用 `cx.notify()`, 界面永远不变; 反过来 `render` 里不要写副作用(它可能一次都不跑, 也可能一帧跑多次).

## 2.1 `App`(全局应用)

- **生命周期与事件循环**: 管理应用的启动, 退出以及操作系统的事件循环.
- **全局状态与注册表**: 持有一个全局的 `EntityMap`, 管理着应用中所有的 `Entity`.
- **全局单例**(`Globals`): 允许注册和获取跨窗口共享的全局服务或配置(通过 `cx.set_global` 和 `cx.global`).
- **异步调度**: 提供后台和前台执行器(`BackgroundExecutor`/`ForegroundExecutor`), 用于派发异步任务.

**NOTE**: `App`严格运行在主线程(UI线程)上, 它不是`Send`的. 在开发中, 你通常不会直接裸用一个全局的`App`实例, 而是通过它的各种派生上下文(`Context`)来间接操作它.

几乎所有API都定义在 `AppContext`/`VisualContext` 两个trait上(而非直接挂在 `App` 上), 所以 `App`, `Context<T>`, `AsyncApp` 等派生上下文共用同一套方法(`cx.new`, `cx.update_entity`, ...). 写可复用的辅助函数时应面向trait编程:

```rust
use gpui::{AppContext, Entity};

fn reset<C: AppContext>(count: &Entity<u64>, cx: &mut C) {
    count.update(cx, |count, cx| {
        *count = 0;
        cx.notify();
    });
}
```

**NOTE**: 这两个 trait 收拢的是"各上下文通用"的那部分 API, 不是全部. 全局读写的 `set_global`/`update_global` 在 `BorrowAppContext` 上, `global::<T>()` 是 `App` 的固有方法(`Context<T>` 靠 `Deref` 共享), `AsyncApp` 提供的是 `read_global`/`update_global` -- 写 `C: AppContext` 的泛型函数时不要假设这些全局方法也在.

## 2.2 `Window`(窗口)

- **渲染与布局宿主**: 每个窗口都是一个独立的渲染表面, 负责承载该窗口内部的UI元素树和布局计算.
- **窗口级事件处理**: 管理窗口的缩放, 移动, 关闭, 最小化, 以及处理窗口级别的键盘焦点(Focus), 鼠标悬停和快捷键分发(Actions).
- **与操作系统的桥梁**: 处理系统原生的窗口装饰, 全屏切换等.

**NOTE**: `Window` 本身**不是**Context, 只是通常与上下文一起传递. 看 `render` 的签名 -- `window` 和`cx`是并列的两个参数:

```rust
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement
//                      ^^^^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^
//                      窗口的事(焦点/尺寸/输入)   实体的事(状态/notify/订阅)
```

窗口相关操作走 `window`(焦点, 尺寸, 缩放, 鼠标位置, 绘制), 实体操作走 `cx`. 需要同时"更新实体 + 拿到窗口"时用 `entity.update_in(cx, |state, window, cx| ...)`.

## 2.3 `Context`(上下文)

GPUI的`Context`其实是一个家族, 不同回调拿到的`cx`类型不同:

| 上下文 | 你在哪拿到它 | 额外能力 |
| -- | -- | -- |
| `&mut App` | `application().run` 回调, 元素事件回调 | 全部应用级能力: 实体, 窗口, 全局, 按键绑定, 平台服务 |
| `&mut Context<T>` | 创建/更新/渲染实体 `T` 时 | `notify` `emit` `observe` `subscribe` `listener` `spawn` -- 都以该实体为中心 |
| `&mut Window` | 与上述上下文并列传递 | 焦点, 尺寸, 缩放, 鼠标位置, 绘制 |
| `&mut AsyncApp` | `cx.spawn` 出来的future里 | 能跨 `.await` 持有应用访问 |
| `&mut AsyncWindowContext` | `cx.spawn_in` 出来的future里 | 同上, 再加一个窗口 |

源码上 `Context<'a, T>` 就是 `{ &mut App, WeakEntity<T> }` 并 `Deref<Target = App>`. 所以在 `Context<T>` 上能直接调 `App` 的所有方法, 而它的实体身份让 `notify()` 不用再传句柄.

常用能力:

- **状态变更与通知**: 修改数据后调用 `cx.notify()`, 告诉 GPUI: "这个组件的数据变了, 请在下一帧重新渲染它".
- **订阅与响应**: 允许一个 `Entity` 监听另一个 `Entity` 的事件或状态变化(`cx.observe`/`cx.subscribe`, 见1.7).
- **派发异步任务**: 通过 `cx.spawn(...)` 启动一个异步 `Future`(例如网络请求, 文件读写), 并在完成后安全地回到UI线程更新状态.
- **创建子实体**: 通过 `cx.new(|cx| ...)` 在当前上下文中注册并创建一个新的 `Entity`.
- **事件桥接**: `cx.listener(...)` 把元素事件回调桥接成能拿到 `&mut Self` 的形式(见1.7).

## 2.4 `Entity` 实体

- **持有业务数据**: 任何需要被GPUI管理, 响应或渲染的结构体, 都可以被注册为 `Entity`.
- **句柄管理**: 当你通过 `cx.new(...)` 创建一个结构体时, GPUI会返回一个强类型的句柄 `Entity<T>`(它类似于一个带类型的安全指针或ID).GPUI另外还提供弱引用 `WeakEntity<T>` 防止循环引用.
- **被动响应**: 由于被`App`托管, 其他组件可以通过 `Entity` 句柄去读取, 修改它, 或者监听它的变动.

值本体归 `App` 所有, 读写必须借助上下文:

```rust
let counter = cx.new(|_cx| Counter { count: 0 }); // 所有权交给 App
counter.update(cx, |counter, cx| {
    counter.count += 1;
    cx.notify();
});
assert_eq!(counter.read(cx).count, 1);
```

四条必知规则:

1. **借用会panic**: `update` 期间值被借出, 闭包里再 `update` **同一个**实体会panic(`cannot update Counter while it is already being updated`). 更新**别的**实体没有问题, 而且是常用模式.
2. **`read` 的借用规则**: 不能拿着 `read` 返回的 `&T` 再使用 `cx`(编译期拦截). 先把数据拷出来, 或把整个操作放进一个 `update` 里.
3. **强弱句柄决定生死**: 最后一个 `Entity<T>` 被drop时, 值在当前update结束时释放(先执行 `cx.on_release` 回调). 父子互相持有强句柄 = 循环引用泄漏, 回指一律用 `WeakEntity<T>`(`handle.downgrade()`, 访问返回 `Result`). 框架内部也在帮你: `cx.listener` 和 `cx.spawn` 的回调持有的是弱句柄, 按钮不会把视图拖着不放.
4. **文本用 `SharedString`**: 不可变且clone极便宜 -- `render` 每帧重建元素树会大量复制字符串, 用 `String` 会很疼.

## 2.5 `Render`/`RenderOnce`(渲染特征)

- **连接数据与UI的桥梁**: 它定义了数据应当如何呈现为视觉元素.
- **声明式描述**: 在`render`方法中, 你不需要手动去"修改"DOM 或像素, 而是每次都通过类似 Tailwind CSS 风格的 DSL(如 `div().flex().child()`)声明式地描述当前状态下的UI结构.

> **NOTE**:GPUI**不做**DOM式的差异对比(Diffing). 元素树每帧全新构建, 用完即弃; 真正的缓存在**视图层** -- 上一帧渲染期间读过的数据没有被 `notify` 时, 该视图的 `render` 直接跳过, 复用上次的布局和绘制结果(见1.9). 所以准确的说法是"视图级复用", 而不是"元素级diff".

签名对比才是两个Trait的本质区别:

| Trait | 签名 | 本质 |
| -- | -- | -- |
| `Render` | `render(&mut self, window, cx: &mut Context<Self>)` | 有状态视图, 本质是 `Entity`, 跨帧存活, 靠 `cx.notify()` 触发更新 |
| `RenderOnce` | `render(self, window, cx: &mut App)` | 无状态组件, 本质是一次性值, 每帧构造-渲染-丢弃 |

`RenderOnce::render` 按值拿走 `self`, 且拿到的是 `&mut App` 而不是 `Context<Self>` -- 它**没有实体身份, 不能 `notify`**. 想上报事件只能接收回调, 由使用它的视图决定怎么办(组件库"外观与状态分离"的由来). 配合 `#[derive(IntoElement)]` 后可以像元素一样使用:

```rust
use gpui::{App, SharedString, Window, div, prelude::*};

#[derive(IntoElement)]
struct Badge {
    label: SharedString,
    count: usize,
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().gap_1().child(self.label).child(self.count.to_string())
    }
}

// 使用: div().child(Badge { label: "Inbox".into(), count: unread })
```

> 💡 提示: 如何选择`Render`还是`RenderOnce`?
> 如果你的组件需要在多次渲染之间保持状态(例如: 计数器, 输入框, 展开/折叠面板), 或者需要被其他代码持有句柄, 使用 `Render`.
如果组件只是展示传入的数据, 不需要维护自己的状态, 使用`RenderOnce`更轻量.

## 2.6 `Element` 与组件体系

官方把GPUI分成三层寄存器: **entity(状态)/ view(声明式UI)/ element(命令式UI)**. 产出UI的三种方式:

| 你想要 | 写法 | 本质 |
| -- | -- | -- |
| 有自己状态的界面块 | **view**: 实现 `Render` 的类型 | 一个Entity, 跨帧存活 |
| 无状态的可复用界面块 | **component**: 实现 `RenderOnce` 的类型 | 一个值, 渲染时被消费 |
| 自定义布局/绘制 | **element**: 实现 `Element` 的类型 | 一个值, 驱动帧的三个阶段 |

日常90%场景用 `div()` + fluent链式API(实现了 `Styled`/`ParentElement`/`InteractiveElement`). 常用技巧:

- **条件样式**: `.when(cond, |el| el.bg(...))`, `.when_some(opt, |el, v| ...)`(`FluentBuilder`), 不打断链式调用.
- **分支返回不同类型**: 用 `.into_any_element()` 擦成 `AnyElement`.
- **自定义 `Element`**: 仅在需要完全控制布局/绘制时使用(文本编辑器, 图表, 海量图元画布), 它的 `request_layout`/`prepaint`/`paint` 三方法对应帧的三个阶段(见1.9); 一次性手绘用 `canvas()` 更省事.

**ElementId -- 交互元素必须有id**:

```rust
div().id(("message", index)).child(...).on_click(cx.listener(...))
```

点击要跨帧记忆("按下"发生在上一帧), 而元素每帧销毁, 这种交互状态由框架按id跨帧保存. 所以 `id()` 把 `div()` 变成 `Stateful<Div>`, 解锁 `on_click`/滚动/拖拽. 两条规则: id在兄弟间唯一; **id必须稳定** -- 列表会增删或排序时用内容id(如数据库id), 别用index.

## 2.7 `notify`, 事件与订阅

`cx.notify()` 不是"立刻重绘", 它做两件事:

1. **标脏窗口**: 上一帧渲染时**读过**这个实体的窗口被标记为需要新帧("读即依赖" -- render里读了哪个entity, 就自动依赖它).
2. **触发observers**: `cx.observe` 注册的回调被调用.

这些效果以Effect形式排队, **直到最外层update结束才统一flush**. 两个实用推论: 同一事件里 `notify` 三次和一次成本相同; observer一定在值不再被借出后才跑. `Window::refresh()` 则是"不通知任何实体, 整窗标脏", 适合系统外观变化等外部原因.

两种事件粒度:

| 机制 | 语义 | 回调签名 |
| -- | -- | -- |
| `cx.observe(&other, cb)` | "你变了"(粗粒度) | `cb(&mut Self, Entity<W>, &mut Context<T>)` |
| `cx.emit(event)` + `cx.subscribe(&other, cb)` | "发生了什么"(类型化) | `cb(&mut Self, Entity<W>, &Evt, &mut Context<T>)` |

`emit` 需要先 `impl EventEmitter<Evt> for T`. 一个实体可以emit多种事件, 订阅者只听到订阅的那种; 与notify一样, 事件在当前update结束后才送达.

两者都返回 `Subscription`, 生命周期是坑点:

- **存进字段** -> 随实体一起死亡(推荐, 一般就是想要的语义).
- **`.detach()`** -> 与实体解绑, 长期有效.
- **什么都不做直接丢弃** -> **立刻退订**(类型标了 `#[must_use]`).

`cx.listener` 也属于这个体系: 元素的事件回调只拿到 `(&ClickEvent, &mut Window, &mut App)`, 不知道是哪个视图创建了它; `cx.listener(|this, event, window, cx| ...)` 把闭包桥接成能拿到 `&mut Self` 的形式, 内部持弱句柄.

## 2.8 `Global` 与异步任务

**Global** -- 跨窗口单例状态, 本质是个marker trait:

```rust
struct Theme {
    dark: bool,
}
impl Global for Theme {}

cx.set_global(Theme { dark: false });
cx.update_global::<Theme, _>(|theme, _cx| theme.dark = true);
assert!(cx.global::<Theme>().dark);
```

选择标准: **世界上只有一个, 且与应用同寿**(主题, 设置, 连接池)用 `Global`; **可能有多份, 或有独立生命周期**的一律用 `Entity`. `cx.observe_global::<G>()` 可监听全局变更.

**异步** -- 重活不进UI线程:

```rust
cx.spawn(async move |this, cx| {
    // this: WeakEntity<T>, cx: &mut AsyncApp
    let data = fetch(url).await;
    this.update(cx, |this, cx| {
        this.data = data;
        cx.notify();
    })
    .ok(); // 视图可能已被释放, update 返回 Result
})
.detach(); // Task 丢弃即取消, 要后台跑完必须 detach(或持有/返回 Task)
```

**NOTE**: `cx.spawn` 返回的 `Task<T>` 标了 `#[must_use]`, **丢弃会让任务立刻被取消**. 后台跑完就链 `.detach()`(如上), 要拿返回值就把 `Task` 存进字段或返回给调用方.

要点: 任务拿到**弱**句柄(不阻止视图释放), 因此 `update` 返回 `Result`; `render` 和事件回调里**绝不能阻塞**, 网络/文件/重计算一律丢给 `spawn`. 需要跨await操作窗口用 `cx.spawn_in`(`AsyncWindowContext`).

## 2.9 帧的四阶段与视图级缓存

一次绘制走四个阶段:

| 阶段 | 干什么 | 此刻知道什么 |
| -- | -- | -- |
| Render | 视图返回元素树 | 视图状态 |
| Request layout | 每个元素提交样式规则, Taffy算出边界 | 有样式, 没尺寸 |
| Prepaint | 元素得知自己的Bounds, 注册hitbox | 有最终边界 |
| Paint | 往场景画矩形/字形/图片, 注册事件监听 | 边界 + hitbox |

**视图级缓存(retained view rendering)**: 视图保留上一帧的绘制结果. 窗口重绘时, GPUI只重新 `render` 发生了变化的视图 -- "依赖"是渲染期间读过的entity/global/输入状态; 只要读过的数据都没被 `notify`, 该视图直接复用上次的布局和绘制输出. 改了数据但忘了 `notify`(或藏在 `Rc<RefCell<_>>`/原子量后面), 复用的视图会一直显示旧画面.

> 💡 提示:GPUI**不是固定帧率游戏循环** -- 有脏窗口或有动画才请求下一帧, 空闲即静止. `render` 保持无副作用, 可能跑0次也可能跑多次.

## 2.10 常见坑速查

| 坑 | 后果 | 解法 |
| -- | -- | -- |
| 改数据忘 `cx.notify()` | 界面不变 | 改完就notify |
| `render` 里写副作用 | 行为不确定(可能跑0次) | 副作用放事件回调或任务 |
| 列表用index当id且会增删 | 交互状态错乱 | 用内容id |
| `Subscription` 不存字段也不 `detach` | 订阅立刻失效 | 存字段或detach |
| `update` 闭包里再update同一实体 | panic | 用闭包里的 `&mut T` |
| 父子互持强句柄 | 内存泄漏 | 回指用 `WeakEntity` |
| 事件回调里做网络/重计算 | UI卡死 | `cx.spawn` |
