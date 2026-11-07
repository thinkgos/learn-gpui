# 7. 事件管理

事件处理是构建交互式界面的核心. GPUI提供了完整的事件系统, 涵盖鼠标、键盘等交互, 并支持事件传播控制. 本章将介绍GPUI中各种事件的监听与处理方式.

## 7.1 事件监听基础

所有事件处理器的回调都是统一形状:  闭包收**三个参数** `(event, window, cx)`:

```rust
// 普通闭包: |event: &E, window: &mut Window, cx: &mut App|
div().on_mouse_move(|event, _window, cx| {
    // event 是事件对象, window 是窗口, cx 是全局应用状态
})

// 配合实体状态用 cx.listener: 闭包收四个参数, 多了第一个 &mut Self
div().id("button").on_click(cx.listener(|this, _event, _window, cx| {
    this.open = !this.open; // 直接改组件状态
    cx.notify();            // 通知GPUI重新渲染
}))
```

**NOTE**: `cx.listener` 的闭包是**四个参数** `(this, event, window, cx)`, 其中 `window` 在第三位.

另一个重要区分: 某些处理器需要**有状态元素**(先调 `.id(...)` 才有):

| 需要 `.id(...)` | 任意元素可用 |
| :--- | :--- |
| `on_click` `on_aux_click` | `on_mouse_down` `on_mouse_up` `on_mouse_move` |
| `on_drag` `on_hover` `active()` | `on_mouse_down_out` `on_scroll` `on_drop` |
| `tooltip()` `focusable()` `overflow_scroll()` | `on_key_down` `on_action` `hover()` ... |

## 7.2 点击事件

`.on_click` 监听点击, 需要有状态元素; 通常搭配 `cx.listener()` 修改组件状态:

```rust
div()
    .id("toggle") // on_click 需要有状态元素
    .on_click(cx.listener(|this, event, _window, cx| {
        if event.is_right_click() {
            return; // 右键不切换
        }
        this.open = !this.open;
        cx.notify();
    }))
    .child("点我切换")
```

`ClickEvent` 是一个枚举, 三种来源都会合成点击:

| 变体 | 触发方式 |
| :--- | :--- |
| `Mouse` | 在元素上按下 + 释放同一鼠标键 |
| `Keyboard` | 元素获得焦点后按 Enter/Space |
| `Touch` | 触屏轻点 |

它不是公开字段的结构体, 要用方法取值:  `event.position()` 位置、`event.is_right_click()` 右键、`event.is_secondary()` 次要操作(右键或长按)、`event.click_count()` 连击次数. 非左键的点击走 `.on_aux_click()`(同样需要 `.id`).

## 7.3 鼠标事件

GPUI 提供了完整的鼠标事件 API(回调形状都是 `(event, window, cx)`):

| 事件 | 说明 | 备注 |
| :--- | :--- | :--- |
| `.on_mouse_down(MouseButton::Left, l)` | 指定键按下 | 第一参数指定按键 |
| `.on_mouse_up(MouseButton::Left, l)` | 指定键释放 | |
| `.on_any_mouse_down(l)` | 任意键按下 | |
| `.on_mouse_move(l)` | 鼠标在元素上移动 | |
| `.on_mouse_down_out(l)` | 鼠标在元素**外**按下 | 见 6.5 |
| `.on_mouse_up_out(MouseButton::Left, l)` | 鼠标在元素外释放 | |
| `.on_scroll(l)` | 滚轮/触控板滚动 | |
| `.on_mouse_exit(l)` | 鼠标移出**窗口**(不是移出元素) | |

**NOTE**: 进出元素的通知用有状态的 `.on_hover(|entered: &bool, window, cx| ...)`(见 6.4), 样式变化用 `.hover(|style| ...)`(见 6.4). 旧的 `.on_scroll_wheel()` 已改名为 `.on_scroll()`.

常用事件对象的字段:

```rust
// MouseDownEvent: button 按键/position 位置/modifiers 修饰键/click_count 连击数
div().on_mouse_down(MouseButton::Left, |event: &MouseDownEvent, _window, _cx| {
    let pos = event.position;   // 鼠标位置(Point<Pixels>)
    let mods = event.modifiers; // ctrl/shift/alt/cmd 状态
})

// ScrollEvent: delta 滚动增量(像素或行数)
div().on_scroll(|event: &ScrollEvent, _window, _cx| {
    match event.delta {
        ScrollDelta::Pixels(p) => { /* 精确滚动(触控板) */ }
        ScrollDelta::Lines(l) => { /* 行滚动(滚轮) */ }
    }
})
```

`MouseButton` 的取值: `Left`/`Right`/`Middle`/`Navigate(Back | Forward)`.

## 7.4 悬停与按压效果

悬停/按压是**样式回调**(改样式, 不是事件处理):

```rust
div()
    .id("button")                       // active() 需要有状态元素
    .hover(|s| s.bg(rgb(0x3a3a3a)))     // 悬停时的样式(任意元素可用)
    .active(|s| s.bg(rgb(0x1a1a1a)))    // 鼠标按住时的样式(需要 .id)
    .on_click(cx.listener(|_this, _event, _window, cx| {
        cx.notify();
    }))
    .child("按钮")
```

- `.hover(|style| ...)`/`.active(|style| ...)` 收一个"样式变换"闭包, 参数是 `StyleRefinement`, 作用类似 Tailwind 的 `hover:`/`active:` 变体.
- `.group_hover("组名", ...)`/`.group_active("组名", ...)` 让同组(`.group("组名")`)的兄弟元素一起响应.
- 要在悬停开始/结束时**做事情**(而不只是换样式), 用有状态的 `.on_hover(|entered: &bool, window, cx| ...)`, `true` 进入、`false` 离开.

## 7.5 鼠标按下外部检测

`.on_mouse_down_out()` 用于检测鼠标在元素外部按下, 常用于关闭弹出层、下拉菜单等场景. 它工作在捕获阶段, 即使内部元素拦截了事件也能触发:

```rust
use gpui::*;

pub struct Popover {
    open: bool,
}

impl Render for Popover {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
        .relative()
        .on_mouse_down_out(cx.listener(|this, _event, _window, cx| {
            // 点击弹出层外部时关闭
            this.open = false;
            cx.notify();
        }))
        .child(
            div()
            .id("toggle") // on_click 需要有状态元素
            .on_click(cx.listener(|this, _event, _window, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child(if self.open { "关闭" } else { "打开" })
        )
    }
}
```

💡 提示: 弹层嵌套时(弹层里还有弹层), 里层打开时要暂时摘掉外层的 `on_mouse_down_out`, 否则点里层会把两层一起关掉. 完整实现(嵌套弹层 + 浮层定位)见 [`cargo run --example popover`](../examples/popover.rs).

## 7.6 事件传播

GPUI 的事件分两个阶段, 相当于 DOM 的捕获/冒泡:

| 阶段 | 鼠标事件 | 键盘事件 |
| :--- | :--- | :--- |
| **捕获**(Capture) | 按绘制顺序从前到后(先绘制的底层元素 → 后绘制的顶层元素); 所有注册了监听器的元素都会收到, 需自行用 hitbox 判断是否命中 | 沿 `dispatch tree` 路径从根 → 焦点元素 |
| **冒泡**(Bubble, 默认) | 按绘制顺序从后到前(顶层元素 → 底层元素); 通常只处理命中的最顶层元素, 可 `cx.stop_propagation()` 阻止继续传递 | 沿 `dispatch tree` 路径从焦点元素 → 根 |
| "层级"的依据 | paint order(视觉层叠) | focus path(焦点祖先链) |

```sh
    [ 窗口根节点 (Root / Window) ]
             /               \
            v  (1. 捕获阶段)   ^  (2. 冒泡阶段)
      [ 中间父容器 Div ]       | 
          /                  |
         v                   |
   [ 目标/焦点子 Div ] --------+
```

键盘事件按 `dispatch tree` 的路径, 它没有"屏幕位置", 目标是当前焦点元素. GPUI 维护一棵 `dispatch tree`, 取从根到焦点节点的路径:

```Root -> Workspace -> Pane -> Editor(focused)`

- `Capture`: 沿路径从根走到焦点元素
- `Bubble`: 从焦点元素走回根

默认情况下事件一路冒泡, 调用 `cx.stop_propagation()` 可以阻止事件继续传播;  `cx.propagate()` 则相反, 用于"处理了但还想让别人也看到"(动作处理有默认消费行为, 见 8.3):

```rust
div().on_mouse_down(MouseButton::Left, |event, _window, cx| {
    if event.modifiers.control {
        cx.stop_propagation(); // ctrl+点击不再传给父元素
    }
})
```

```rust
// window.prevent_default(): 阻止默认行为(目前主要用于"按下时不让父元素抢焦点")
```

## 7.7 拖放

拖放由三组 API 构成: 拖拽源 `.on_drag()`、放置目标 `.on_drop()`、拖拽中的跟踪 `.on_drag_move()`:

```rust
// 拖拽源(需要 .id): 第一个参数是拖拽携带的数据,
// 第二个参数是构造"拖拽影子"视图的闭包(跟鼠标走的那个小窗)
div()
    .id(("item", index))
    .cursor_move() // 拖拽光标: cursor: move
    .on_drag(item_data, |data: &ItemData, offset, _window, cx| {
        cx.new(|_| DragGhost { data: data.clone(), offset })
    })

// 放置目标(普通元素即可): 松手时收到拖拽数据
div()
    .on_drop(cx.listener(|this, data: &ItemData, _window, _cx| {
        this.dropped = Some(data.clone());
    }))
```

- 拖拽数据是任意 `'static` 类型, `on_drop` 只在"拖拽数据类型匹配"时触发.
- `.can_drop(|data: &dyn Any, window, cx| -> bool)` 给放置目标加过滤条件.
- `.drag_over::<T>(|style, data, window, cx| ...)` 在有匹配拖拽悬停时改放置目标的样式(高亮提示).
- 拖拽开始后移动超过 2 像素才算拖拽;  `.on_drag_move(|event: &DragMoveEvent<T>, ...)` 收到拖拽期间的所有移动(包括移出元素后).

完整示例: [`cargo run --example drag_drop`](../examples/drag_drop.rs).

## 7.8 鼠标压力

压感设备(如力度触控板)的压力是**独立事件**, 不是鼠标事件的字段:

```rust
div().on_mouse_pressure(cx.listener(|this, event: &MousePressureEvent, _window, cx| {
    this.pressure = event.pressure; // 压力值 0.0~1.0
    this.stage = event.stage;       // Zero/Normal/Force 三档
    cx.notify();
}))
```

**NOTE**: 压力走 `MousePressureEvent`(目前仅 **macOS** 力度触控板实现). 触屏的压力在 `TouchEvent.force` 里. 完整示例: [`cargo run --example mouse_pressure`](../examples/mouse_pressure.rs).

## 7.9 事件方法速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **回调形状** | `\|event, window, cx\|`/`cx.listener(\|this, event, window, cx\|)` | 普通闭包 3 参, listener 闭包 4 参 |
| **点击** | `.on_click()` `.on_aux_click()` | 左键点击/其他键点击(需 `.id`) |
| **鼠标** | `.on_mouse_down(btn, l)` `.on_mouse_up(btn, l)` `.on_mouse_move(l)` `.on_mouse_exit(l)` | 按下/释放/移动/移出窗口 |
| **外部检测** | `.on_mouse_down_out(l)` `.on_mouse_up_out(btn, l)` | 元素外按下/释放(捕获阶段) |
| **滚动** | `.on_scroll(l)` | 滚轮/触控板(`on_scroll_wheel` 已弃用) |
| **悬停/按压** | `.hover()` `.active()` `.on_hover(l)` `.group_hover()` `.group_active()` | 样式回调与进出通知 |
| **拖放** | `.on_drag(data, ctor)` `.on_drop(l)` `.on_drag_move(l)` `.can_drop()` `.drag_over()` | 拖拽源/放置目标/过程跟踪 |
| **压力** | `.on_mouse_pressure(l)` | 压感事件 `MousePressureEvent` |
| **键盘** | `.on_key_down(l)` `.on_key_up(l)` | 键盘事件沿焦点路径传播(见第 8 章) |
| **传播控制** | `cx.stop_propagation()` `cx.propagate()` `window.prevent_default()` | 停止/恢复传播、阻止默认行为 |

## 7.10 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`popover`](../examples/popover.rs) | 两级嵌套浮层 + 点击外部关闭(见 7.5; 浮层定位另见第 16 章) |
| [`drag_drop`](../examples/drag_drop.rs) | 把 Item 拖进 "Drop items here", 顺带演示拖拽预览(见 7.7; 拖拽全流程另见第 11 章) |
| [`mouse_pressure`](../examples/mouse_pressure.rs) | 实时显示压力阶段和压力值(见 7.8) |

> 💡 提示: 鼠标压力目前仅 macOS 力度触控板实现, 触屏的压力在 `TouchEvent.force` 里.
