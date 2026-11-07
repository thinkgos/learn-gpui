# 8. 焦点管理

焦点管理是构建可访问 GUI 应用的关键. GPUI提供了完善的焦点系统, 支持键盘导航、Tab 切换和程序化聚焦.

## 8.1 FocusHandle: 创建和管理焦点

`FocusHandle` 是GPUI焦点系统的核心, 每个可聚焦的元素都通过 `FocusHandle` 来管理焦点状态. 用 `cx.focus_handle()` 创建, 用 `.track_focus()` 绑到元素上:

```rust
use gpui::*;

pub struct Button {
    focus_handle: FocusHandle,
}

impl Button {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(), // 创建焦点句柄
        }
    }
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
        .track_focus(&self.focus_handle) // 绑定句柄: 元素可聚焦, 焦点样式生效
        .on_click(cx.listener(|this, _event, window, cx| {
            this.focus_handle.focus(window, cx); // 点击时把焦点移过来
        }))
        .child("按钮")
    }
}
```

`FocusHandle` 常用方法:

| 方法 | 说明 |
| :--- | :--- |
| `cx.focus_handle()` | 创建焦点句柄 |
| `handle.focus(window, cx)` | 把焦点移到这个句柄 |
| `handle.is_focused(window)` | 这个句柄是否持有焦点 |
| `handle.contains_focused(window, cx)` | 自己或子孙持有焦点(焦点是否在容器内) |
| `handle.within_focused(window, cx)` | 自己是否位于焦点元素内部 |
| `handle.tab_index(n)`/`handle.tab_stop(b)` | 链式设置 Tab 顺序(见 8.3) |
| `window.blur(cx)` | 清除焦点(谁都不聚焦) |
| `window.focused(cx)` | 当前持有焦点的句柄(`Option<FocusHandle>`) |

## 8.2 焦点样式: focus() 与 focus_visible()

`focus()` 和 `focus_visible()` **是两个样式方法**(定义在元素上, 配合 `.track_focus()` 生效), 对应 CSS 的 `:focus` 和 `:focus-visible`:

| 方法 | 什么时候生效 | 典型用途 |
| :--- | :--- | :--- |
| `.focus(\|s\| ...)` | 元素持有焦点(不管怎么来的) | 搜索框聚焦时显示边框 |
| `.focus_visible(\|s\| ...)` | 元素持有焦点**且最后一次输入来自键盘** | 键盘导航时才显示焦点框 |
| `.in_focus(\|s\| ...)` | 元素位于焦点元素**内部** | 整个卡片内的控件一起高亮 |

```rust
div()
    .track_focus(&self.focus_handle)
    .focus(|s| s.border_2().border_color(gpui::blue()))        // 聚焦: 蓝框
    .focus_visible(|s| s.border_4().border_color(gpui::red())) // 键盘聚焦: 红粗框
    .child("按钮")
```

**NOTE**: `focus_visible()` 区分的是**输入方式**(键盘还是鼠标), 不是"程序聚焦 vs 键盘聚焦". GPUI记录最近一次输入设备: 鼠标点击聚焦后不显示焦点框, Tab 导航聚焦才显示 —— 这正是 CSS `:focus-visible` 的语义.

三种样式对照的完整示例: [`cargo run --example focus_visible`](../examples/focus_visible.rs).

## 8.3 Tab 导航

GPUI 内置支持 Tab 键导航: 每帧渲染时收集所有焦点句柄排成"Tab序", `window.focus_next()`/`window.focus_prev()` 就在序列里前进/后退. 自定义顺序和行为的开关:

| 方法 | 说明 |
| :--- | :--- |
| `.tab_index(n)` | Tab 顺序编号, 从小到大; 相同则按元素出现顺序 |
| `.tab_stop(false)` | 保留顺序位置, 但 Tab 到不了(只能程序聚焦) |
| `.tab_group()` | 子树自成一组, 子元素的 tab_index 从 0 重新计 |

`.tab_index(n)`/`.tab_stop(b)` 既有元素版(`div().tab_index(1).tab_stop(true)`)也有句柄版(`cx.focus_handle().tab_index(1).tab_stop(true)`), 效果一致; `.tab_group()` **没有**句柄版, 只能用在元素上. 典型的 Tab 键绑定写法(动作定义见第 9 章):

```rust
actions!(example, [Tab, TabPrev]);

// 绑定: tab 前进, shift-tab 后退
cx.bind_keys([
    KeyBinding::new("tab", Tab, None),
    KeyBinding::new("shift-tab", TabPrev, None),
]);

// 根元素上监听并转成焦点移动
div().track_focus(&self.focus_handle)
    .on_action(cx.listener(|_this, _: &Tab, window, cx| window.focus_next(cx)))
    .on_action(cx.listener(|_this, _: &TabPrev, window, cx| window.focus_prev(cx)))
```

完整示例(六个不同 tab 设置的按钮 + tab group): [`cargo run --example tab_stop`](../examples/tab_stop.rs).

## 8.4 焦点在容器内: contains_focused

想知道"焦点是不是落在我这个容器里"(比如工具栏里任意控件聚焦时工具栏高亮), 用句柄的 `contains_focused`:

```rust
// 判断版: 焦点在容器内时改任意逻辑
if self.toolbar_focus.contains_focused(window, cx) {
    // ...
}
```

样式版 `.in_focus(|s| ...)` 是**反方向**(元素位于焦点元素内部, 见 8.2), 不是 `contains_focused` 的样式写法:

```rust
// 反方向: 元素位于焦点元素内部时高亮, 不是"焦点在容器内"
div().track_focus(&self.toolbar_focus)
    .in_focus(|s| s.bg(rgb(0x2a2a2a)))
```

## 8.5 焦点事件

需要在焦点进出时执行逻辑(而不仅是换样式), 用以下订阅方法. 它们都返回 `Subscription`, 要保存住(或 `.detach()`)才持续有效:

| 方法 | 说明 |
| :--- | :--- |
| `cx.on_focus_in(&handle, window, l)` | 焦点进入句柄或其子孙时触发 |
| `cx.on_blur(&handle, window, l)` | 该句柄失去焦点时触发 |
| `cx.on_focus_out(&handle, window, l)` | 焦点离开句柄及其子孙时触发 |
| `cx.on_focus_lost(window, l)` | 窗口内谁都没有焦点时触发(适合选默认焦点) |

## 8.6 焦点方法速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **句柄** | `cx.focus_handle()` `handle.focus()` `handle.is_focused()` `handle.contains_focused()` | 创建/移动/查询焦点 |
| **窗口** | `window.focus()` `window.blur()` `window.focused()` `window.focus_next()` `window.focus_prev()` | 窗口级焦点控制与 Tab 导航 |
| **绑定元素** | `.track_focus(&handle)` `.focusable()` | 元素可聚焦(`focusable` 需要 `.id`) |
| **焦点样式** | `.focus()` `.focus_visible()` `.in_focus()` | 聚焦/键盘聚焦/自身位于焦点元素内的样式 |
| **Tab 顺序** | `.tab_index(n)` `.tab_stop(b)` `.tab_group()` | 元素版与句柄版等价(`tab_group` 仅元素版) |
| **焦点事件** | `cx.on_focus_in()` `cx.on_blur()` `cx.on_focus_out()` `cx.on_focus_lost()` | 焦点进出订阅 |

## 8.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`focus_visible`](../examples/focus_visible.rs) | 三种样式对照: 只用 `.focus()`、只用 `.focus_visible()`、两者叠加(见 8.2) |
| [`tab_stop`](../examples/tab_stop.rs) | 六个不同 tab 设置的按钮 + tab group, 并显示按了 Tab 还是 Shift-Tab(见 8.3) |
