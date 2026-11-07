# 16. 浮层 Popover

下拉菜单、悬浮提示、选择器这类"浮在正文之上"的内容都属于 Popover.GPUI的浮层由两个核心 API 搭配而成: `deferred()` 把内容抬到独立的绘制层, `anchored()` 负责把内容锚定到触发元素的某个角上. 

## 16.1 deferred(): 延迟渲染容器

`deferred()` 的语义是**布局留在原地、绘制推迟到所有祖先之后**: 子元素照常参与当前元素树的布局, 但绘制被挪到整个界面画完之后, 所以浮层永远盖在正文上, 不会被后渲染的内容盖住. 

```rust
deferred(
    anchored()
        .anchor(Anchor::TopLeft) // 锚点: 浮层挂在触发元素的左上角
        .snap_to_window_with_margin(px(8.)) // 贴边收回时留 8px 余量, 保证浮层不出窗口
        .child(popover()), // 浮层内容
)
.priority(1), // 浮层绘制优先级: 数字大的画在上层
```

`deferred()` 常用配置: 

| 方法 | 说明 |
| :--- | :--- |
| `deferred(child)` | 把 child 变成浮层(延迟绘制) |
| `.priority(n)` | 多个浮层之间的堆叠顺序, 大的在上(默认 0) |
| `.with_priority(n)` | 与 `.priority(n)` 完全等价 |

**NOTE**: `deferred()` **没有**"条件渲染"参数, `deferred(move |_cx| show).child(...)` 这种写法不存在 —— 显隐要自己控制, 常用 `.when(self.open, |this| this.child(deferred(...)))`: 状态开着才把浮层挂进元素树. 

**NOTE**: 优先级是链式方法 `.priority(1)`(或 `.with_priority(1)`), **没有** `with_priority(1).child(...)` 这个自由函数. 

## 16.2 anchored(): 锚定定位(Anchor 枚举)

`anchored()` 把浮层钉在父元素包围盒的参考点上, 下拉菜单通常锚定在按钮下方. `Anchor` 有 8 个变体, 覆盖四角、上下边中点、左右边中点: 

| 变体 | 参考点 | 典型用途 |
| :--- | :--- | :--- |
| `Anchor::TopLeft`/`TopCenter`/`TopRight` | 上边(左/中/右) | 悬浮提示 |
| `Anchor::BottomLeft`/`BottomCenter`/`BottomRight` | 下边(左/中/右) | 下拉菜单(常用 BottomLeft) |
| `Anchor::LeftCenter`/`RightCenter` | 左右边中点 | 级联子菜单 |

**NOTE**: `Anchor` **没有** `LeftTop`/`LeftBottom`/`RightTop`/`RightBottom` 这些变体 —— 侧边锚点只有居中的 `LeftCenter`/`RightCenter`. 

`anchored()` 的定位方法: 

| 方法 | 说明 |
| :--- | :--- |
| `.anchor(Anchor)` | 锚点(浮层的哪条边贴着参考点) |
| `.position(Point<Pixels>)` | 锚点坐标(默认取父元素原点) |
| `.offset(point)` | 在锚点基础上再偏移 |
| `.position_mode(AnchoredPositionMode::Window/Local)` | 坐标相对窗口还是父元素 |
| `.attach(bounds, corner)` | 直接指定参考 bounds, 不用父元素 |
| `.snap_to_window()` | 浮层超出窗口时自动贴边收回 |
| `.snap_to_window_with_margin(px(8.))` | 贴边收回并留出余量 |

悬浮提示的典型写法(取自 `anchor` 示例): 

```rust
deferred(
    anchored()
        .anchor(corner) // 锚点: 按钮的对应角
        .position(position) // 锚点坐标: 按钮对应角的位置
        .position_mode(AnchoredPositionMode::Local) // 坐标相对父元素: 类似 position: absolute
        .snap_to_window() // 超出窗口自动贴边
        .child(
            div()
                .py_0p5() // 上下内边距: padding-top: 0.125rem; padding-bottom: 0.125rem
                .px_2()   // 左右内边距: padding-left: 0.5rem; padding-right: 0.5rem
                .bg(gpui::black().opacity(0.75)) // 半透明黑背景: background-color: rgb(0 0 0/0.75)
                .text_color(rgb(0xffffff)) // 文字颜色: color: #ffffff
                .rounded_sm() // 小圆角: border-radius: 0.125rem
                .shadow_sm()  // 小阴影: box-shadow: 0 1px 2px 0 rgb(0 0 0/0.05)
                .min_w(px(100.0)) // 最小宽度: min-width: 100px
                .text_sm() // 字号缩小: font-size: 0.875rem; line-height: 1.25rem
                .child(button.label.clone()),
        ),
)
```

## 16.3 on_mouse_down_out: 点击外部关闭

Popover 的标准交互是"点浮层外面就关掉", 用 `on_mouse_down_out` 实现: 它在鼠标**按下**事件的捕获阶段检查位置, 按下点不在元素范围内就触发回调. 

```rust
.when(self.open, |this| {
    this.child(
        deferred(
            anchored()
                .anchor(Anchor::TopLeft)
                .snap_to_window_with_margin(px(8.))
                .child(
                    popover()
                        .w_96() // 宽度: width: 24rem (384px)
                        .child("Click outside to close.")
                        // 关键: 鼠标在浮层外按下时关闭
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.open = false; // 收起浮层
                            cx.notify(); // 通知 GPUI: 状态变了, 需要重新渲染
                        })),
                ),
        )
        .priority(1),
    )
})
```

- 回调签名是 `Fn(&MouseDownEvent, &mut Window, &mut App)`; 用 `cx.listener(...)` 包一层就能在回调里拿到视图状态(`this`).
- 对应的 `on_mouse_up_out(button, listener)` 在鼠标**抬起**在元素外时触发.
- 只想在浮层内部做点关闭按钮时, 配合 `on_click` 改同一个 `open` 状态即可.

**NOTE**: 回调收三个参数(event、window、cx), `move |_event, cx| { ... }` 这种两参闭包 **没有**; 也不要写成 `cx.update_model(...)` —— 视图状态直接用 `cx.listener` 的 `this` 改. 

## 16.4 多级嵌套 Popover(priority 层级控制)

级联菜单(菜单里再开菜单)就是嵌套 Popover: 每层一个 `deferred`, 用 `priority` 控制堆叠顺序, 数字大的画在上面(取自 `popover` 示例, 一级菜单 `.priority(1)`, 二级菜单 `.priority(2)`): 

```rust
// 二级菜单: 嵌套在一级浮层里, 优先级更高, 画在一级之上
deferred(
    anchored()
        .anchor(Anchor::TopLeft) // 锚点: 挂在触发按钮左上角
        .snap_to_window_with_margin(px(8.))
        .child(
            popover()
                .child("This is second level Popover with nested deferred!")
                .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                    this.secondary_open = false;
                    cx.notify();
                })),
        ),
)
.priority(2), // 二级浮层层级: 比一级(1)高, 画在它上面
```

两个实战细节(示例里都踩过): 

1. **子浮层开着时要关掉父浮层的"点外面关闭"**, 否则点子菜单会被当成"点外面"把父级一起关掉: 

```rust
.when(!self.secondary_open, |this| {
    // 只有子浮层没开时才监听"点外面关闭"
    this.on_mouse_down_out(cx.listener(|this, _, _, cx| {
        this.open = false;
        cx.notify();
    }))
})
```

2. **浮层要放在触发器内容之后渲染**(`.child(...)` 接在按钮文字后面), 确保它在同层元素的上层.GPUI已支持多层嵌套 `deferred`, 级联几层都可以. 

## 16.5 典型场景

- **下拉菜单**: 点按钮展开选项, `Anchor::BottomLeft` 锚定, `on_mouse_down_out` 关闭(16.2 + 16.3 的组合).
- **工具提示 Tooltip**: 悬停显示提示, 用 `on_hover(|hovered, _, cx| ...)` 切换显隐, 浮层用悬浮提示的锚定写法(见 16.2).
- **选择器 Picker**: 输入时展示候选项, 结合文本输入(见第 10 章)和 `uniform_list`(见第 15 章)做搜索列表.
- **级联菜单**: 多层嵌套 `deferred`, `.priority()` 控制堆叠(见 16.4).

**NOTE**: **没有** `on_mouse_enter` 这个方法(整个GPUI里都不存在) —— 悬停进入/离开用 `on_hover(闭包)`(回调收一个 `bool`: 进入为 true、离开为 false), 只关心离开时用 `on_mouse_exit`. 

## 16.6 Popover vs Modal 的选择

| 维度 | Popover | Modal |
| :--- | :--- | :--- |
| 交互 | 非模态, 不打断当前操作 | 模态, 阻塞其他所有交互 |
| 位置 | 锚定在触发元素旁(anchored) | 通常屏幕居中 + 遮罩 |
| 关闭 | 点外部即可关闭 | 只能点按钮/遮罩 |
| 适合 | 下拉菜单、选择器、工具提示 | 确认对话框、表单、重要通知 |

选择原则: 操作是上下文相关的、不需要打断用户流程, 用 Popover; 需要用户专注完成某件事, 用 Modal. 

**NOTE**:GPUI**没有**内置 Modal 组件 —— 模态框一般用 `deferred()` 抬一层全屏遮罩 + 居中对话框自己搭, 在遮罩上处理点击/按键来关闭. 

## 16.7 浮层方法速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **浮层容器** | `deferred(child)` `.priority(n)` `.with_priority(n)` | 延迟绘制, 抬到浮层并排序 |
| **锚定** | `anchored()` `.anchor()` `.position()` `.offset()` `.attach()` | 锚定到参考元素 |
| **定位模式** | `.position_mode(Window/Local)` | 坐标相对窗口还是父元素 |
| **防出界** | `.snap_to_window()` `.snap_to_window_with_margin(edges)` | 浮层不出窗口 |
| **锚点** | `Anchor::TopLeft/TopCenter/TopRight/BottomLeft/BottomCenter/BottomRight/LeftCenter/RightCenter` | 8 个参考点 |
| **关闭交互** | `on_mouse_down_out()` `on_mouse_up_out()` | 点击浮层外部 |
| **显隐控制** | `.when(cond, ...)` `cx.listener(...)` `cx.notify()` | 状态驱动显隐 |

完整示例: [`cargo run --example popover`](../examples/popover.rs)(两级嵌套浮层 + 点外部关闭), [`cargo run --example anchor`](../examples/anchor.rs)(8 个锚点的悬浮提示演示).

## 16.8 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`popover`](../examples/popover.rs) | `deferred()` + `anchored()` 两级嵌套浮层 + 点击外部关闭(见 16.1–16.4) |
| [`anchor`](../examples/anchor.rs) | 8 个锚点的悬浮提示演示(见 16.2) |
