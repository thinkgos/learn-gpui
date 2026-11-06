# 11. 拖拽功能

拖拽(drag and drop)是列表、看板类界面的常见交互.GPUI的拖拽是声明式的: 源元素用 `.on_drag()` 说清「拖的是什么、拖起来长什么样」, 目标元素用 `.on_drop()` 决定「收不收、怎么处理」, 再用 `.drag_over()` 给悬停反馈. 拖拽数据可以是任意 `'static` 类型, 不需要实现特殊 trait.

## 11.1 拖拽源: .on_drag()

`.on_drag()` 把元素标记为可拖拽源, 一次收两个参数: 拖拽数据 `value: T` 和预览构造器 `constructor`. 按住左键移动超过 2 像素后拖拽自动开始, 构造器被调用一次, 返回一个实现 `Render` 的实体(Entity)作为跟随光标的拖拽预览:

```rust
let drag_info = DragInfo::new(ix, color); // 拖拽携带的数据, 普通结构体即可

div()
    .id(("item", ix)) // 每个可拖元素要先有唯一 id: on_drag 只在带 id 的元素上可用
    .size_32() // 32 号尺寸的正方形: width: 8rem; height: 8rem (128px)
    .cursor_move() // 拖动光标: cursor: move
    .child(format!("Item ({})", ix))
    .on_drag(drag_info, |info: &DragInfo, position, _, cx| {
        // 收 4 个参数: 拖拽数据、按下点相对元素原点的偏移、window、cx
        cx.new(|_| info.position(position)) // 构造预览实体, 跟随光标绘制
    })
```

源码里的完整签名(两部分都要给):

```rust
fn on_drag<T, W>(
    value: T,
    constructor: impl Fn(&T, Point<Pixels>, &mut Window, &mut App) -> Entity<W>,
) where
    T: 'static,
    W: 'static + Render;
```

- `T` 是拖拽数据的类型, 任意 `'static` 类型都行; `W` 是预览实体的类型, 必须实现 `Render`.
- `on_drag` 是**带 id 元素**(`StatefulInteractiveElement`)的方法, 元素上没写 `.id()` 就用不了.
- 同一个元素只能调一次 `on_drag`, 重复调用在 debug 下直接断言失败.
- 拖拽期间可用 `cx.has_active_drag()` 查询「现在有没有拖拽」.

**NOTE**: 启动拖拽 **没有** `cx.start_drag()` 这个方法. 其他教程里的 `cx.listener(|view, e, cx| { cx.start_drag(data) })` 写法在 zed 的GPUI中编译不过: `on_drag` 收的是「数据 + 预览构造器」两个参数, 拖拽由「按住左键 + 移动」的手势自动启动, 不用手动开始.

## 11.2 放置目标: .on_drop()

`.on_drop()` 把元素标记为放置目标. 回调只在「拖拽数据的类型」和 `on_drop::<T>` 的 `T` 一致时才会触发, 类型匹配由GPUI按 TypeId 自动完成:

```rust
div()
    .id("drop-target")
    .w_128() // 宽: width: 32rem (512px)
    .h_32()  // 高: height: 8rem (128px)
    .on_drop(cx.listener(|this, info: &DragInfo, _, _| {
        // 回调直接拿到拖拽数据 &DragInfo
        this.drop_on = Some(*info); // 记录放进来的数据, 驱动边框/底色变化
    }))
    .child("Drop items here")
```

- 回调收 `(&T, &mut Window, &mut App)`; 用 `cx.listener()` 包一层(见 11.5)就能在回调里读写自己的状态.
- `on_drop`/`can_drop` 是普通元素方法, 不要求 `.id()`.

`.can_drop()` 给放置目标加一道谓词, 决定这个拖拽数据收不收:

```rust
div()
    .on_drop(cx.listener(|this, info: &DragInfo, _, _| {
        this.drop_on = Some(*info);
    }))
    .can_drop(|data, _window, _cx| {
        // 收 &dyn Any: 返回 false 时 on_drop 不触发, drag_over 样式也保持原样
        data.downcast_ref::<DragInfo>().is_some_and(|info| info.ix != 0)
    })
```

**NOTE**: `on_drop` 回调直接拿到拖拽数据 `&T`, **没有** `drop_event.data.downcast_ref::<DragInfo>()` 这种取法 —— 其他教程里靠 `Any` + `downcast_ref` 取数据是多此一举, 类型匹配在派发前就做完了.

## 11.3 DragInfo: 拖拽数据载体

**NOTE**:GPUI**没有** 内置的 `DragInfo` 类型. 拖拽数据就是你在 `.on_drag()` 里传的那个 `value`, 什么结构体都行; 下面示例里的 `DragInfo` 是示例自己定义的(见 [`examples/drag_drop.rs`](../examples/drag_drop.rs)), 一物两用: 既当拖拽数据, 又实现 `Render` 用作拖拽预览. 数据和预览也可以拆成两个类型, 只要预览那个实现 `Render`:

```rust
/// 拖拽数据: 记录拖的是第几项、什么颜色, 以及按下点的偏移
#[derive(Clone, Copy)]
struct DragInfo {
    ix: usize,
    color: Hsla,
    position: Point<Pixels>,
}

impl Render for DragInfo {
    fn render(&mut self, _: &mut Window, _: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex() // 弹性布局: display: flex
            .justify_center() // 主轴居中: justify-content: center
            .items_center() // 交叉轴居中: align-items: center
            .bg(self.color.opacity(0.5)) // 半透明项目色: background-color: rgb(.../50%)
            .text_color(gpui::white()) // 文字颜色: color: #ffffff
            .child(format!("Item {}", self.ix))
    }
}
```

**NOTE**: `Render::render` 的签名是 `fn render(&mut self, window: &mut Window, cx: &mut Context<Self>)`, **没有** `ViewContext` 这个类型 —— 那是旧版GPUI的叫法, 早改名成 `Context` 了. 收 `&self` 和 `cx: &mut ViewContext<Self>` 的写法编译不过.

预览实体由 `.on_drag()` 的构造器创建(`cx.new(|_| ...)`)、由GPUI释放, 每次拖拽只创建一次;GPUI把它画在「鼠标位置 - 按下点偏移」处, 所以预览看起来停在当初抓取的位置.

## 11.4 drag_over 样式: 拖拽悬停时的视觉反馈

`.drag_over::<S>()` 在「类型为 `S` 的数据悬停在元素上」时追加一段样式, 挪走自动恢复. 闭包收「基础样式 + 拖拽数据 + window + cx」, 把改好的样式返回出去:

```rust
div()
    .id("drop-target")
    .on_drop(cx.listener(|this, info: &DragInfo, _, _| {
        this.drop_on = Some(*info);
    }))
    .drag_over::<DragInfo>(|style, _info, _window, _cx| {
        style
            .bg(gpui::blue().opacity(0.2)) // 悬停时淡蓝底: background-color: #0000ff33
            .border_2() // 边框加粗到 2px: border-width: 2px
            .border_color(gpui::blue()) // 边框颜色: border-color: #0000ff
    })
```

同族方法:

| 方法 | 说明 |
| :--- | :--- |
| `.drag_over::<S>(f)` | 类型为 `S` 的拖拽悬停在**本元素**上时改样式 |
| `.group_drag_over::<S>(组名, f)` | 悬停在**同组元素**里任何一个上时, 本元素也改样式 |

**NOTE**: 方法叫 `drag_over()`, **没有** `on_drag_over()` 这个方法; 闭包签名是 `(StyleRefinement, &S, &mut Window, &mut App) -> StyleRefinement`, **没有** bounds 参数. 其他教程里的 `on_drag_over::<DragInfo>(|style, data, bounds, cx| ...)` 在 zed 的GPUI中编译不过.

## 11.5 拖拽生命周期

一次完整拖拽经历的阶段:

1. **开始拖拽**: 在源元素上按住左键、移动超过 2 像素 →GPUI调用 `.on_drag()` 的构造器生成预览实体, 拖拽进入活跃状态(`cx.has_active_drag()` 变为 `true`).
2. **拖拽移动**: 鼠标移动 → 预览跟随光标, 窗口持续重绘; 需要在移动时做事(如调整滚动)就订阅 `.on_drag_move::<T>()`:

   ```rust
   div()
       .on_drag_move::<DragInfo>(cx.listener(|this, e, _window, cx| {
           // e 是 DragMoveEvent<DragInfo>:
           //   e.event    鼠标移动事件(MouseMoveEvent)
           //   e.bounds   本元素的区域
           //   e.drag(cx) 取拖拽数据 &DragInfo
           this.hover_pos = Some(e.event.position); // 例如记录悬停位置
           cx.notify();
       }))
   ```

   该事件在拖拽期间的每次鼠标移动都会触发, 不管光标在不在元素内.
3. **进入目标**: 光标悬停到元素上且拖拽数据类型匹配 → `.drag_over()`/`.group_drag_over()` 样式生效(`can_drop` 谓词为 `false` 时保持原样).
4. **离开目标**: 光标挪走, 悬停样式自动失效, 元素恢复原样.
5. **放置**: 松开左键 → 命中元素的 `on_drop` 按类型匹配触发(先过 `can_drop` 谓词), 拖拽数据交给目标处理, 活跃拖拽随之结束.
6. **取消**: 松手时下方没有匹配的放置目标(或 `can_drop` 返回 `false`) → 活跃拖拽直接丢弃, 数据不给任何人; 也可以随时调 `cx.stop_active_drag(window)` 主动取消, 返回 `bool` 表示是否真有拖拽被取消.

**NOTE**: 按 `Escape` 取消拖拽 **没有**内置实现 —— 框架只在「松手落空/`stop_active_drag`」时清掉拖拽. 想要 Esc 取消, 自己绑一个动作(见第 8 章)在回调里调 `cx.stop_active_drag(window)`.

💡 提示: 跨窗口与跨应用. 活跃拖拽存在 `App` 上, 同一应用的多个窗口都能命中放置目标. 拖出窗口时, 若源元素注册了 `.external_drag_payload()`, 拖拽会升级成平台原生拖拽(目前支持向外拖文件); 从系统拖文件进窗口则走 `FileDropEvent` 事件(`Entered`/`Pending`/`Submit`/`Exited`/`Ended`), 元素级钩子有 `.on_file_drop_exit()`.

## 11.6 拖拽方法速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **拖拽源** | `.id()` + `.on_drag(value, 构造器)` | 标记可拖, 携带数据 + 预览实体(必须带 id) |
| **放置目标** | `.on_drop(listener)` `.can_drop(谓词)` | 按类型收数据/决定是否可放 |
| **悬停反馈** | `.drag_over::<T>(f)` `.group_drag_over::<T>(组名, f)` | 拖拽悬停时追加样式 |
| **移动事件** | `.on_drag_move::<T>(listener)` | 拖拽期间的鼠标移动(`DragMoveEvent<T>`) |
| **手动控制** | `cx.has_active_drag()` `cx.stop_active_drag(window)` | 查询/取消当前拖拽 |
| **跨窗口/跨应用** | `.external_drag_payload(resolver)` `FileDropEvent` | 把拖拽交给平台/接收系统文件拖放 |

完整示例: [`cargo run --example drag_drop`](../examples/drag_drop.rs).

## 11.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`drag_drop`](../examples/drag_drop.rs) | `DragInfo` 一物两用(拖拽数据 + `Render` 预览), `.on_drag()`/`.on_drop()`/`.drag_over()` 全流程(见 11.1–11.5) |
