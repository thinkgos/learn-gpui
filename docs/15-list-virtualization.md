# 15. 列表与虚拟化

数据量大的列表如果一次性渲染所有行, 滚动会严重卡顿. GPUI提供 `uniform_list`(等高行)和 `list`(变高行)两种虚拟化列表, 只渲染可见区域的行, 十万行也能流畅滚动.

> 💡 提示: 虚拟化把渲染复杂度从 O(n) 降到 O(k), k 是可见行数(通常不超过 50). 10 万行数据里, 任何一帧真正创建的元素只有几十个.

## 15.1 uniform_list: 等高行虚拟列表

`uniform_list` 适用于所有行高度相同的场景(文件列表、消息列表、数据表格). 它不走完整的 taffy 布局, 而是量出第一行的高度, 其余行按这个高度依次排列, 因此比 `list` 快得多:

```rust
use gpui::*;

struct UniformListExample {}

impl Render for UniformListExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(0xffffff)).child(
            uniform_list(
                "entries", // 列表 id,GPUI内部跟踪滚动状态用
                50,       // 行数, 可以是十万行
                cx.processor(|_this, range, _window, _cx| {
                    // 回调只收到可见区间的行号, 比如 12..24
                    let mut items = Vec::new();
                    for ix in range {
                        let item = ix + 1;

                        items.push(
                            div()
                                .id(ix)          // 每行要有 id, 才能响应点击等交互
                                .px_2()          // 左右内边距: padding-left: 0.5rem; padding-right: 0.5rem
                                .cursor_pointer() // 悬停显示手型: cursor: pointer
                                .on_click(move |_event, _window, _cx| {
                                    println!("clicked Item {item:?}");
                                })
                                .child(format!("Item {item}")),
                        );
                    }
                    items // 返回这个区间的所有行, 元素类型: Vec<R: IntoElement>
                }),
            )
            .h_full(), // 列表高度撑满父容器: height: 100%
        )
    }
}
```

**`uniform_list` 的签名**: `uniform_list(id, item_count, f)`, 其中 `f: Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>`.

**滚动状态 UniformListScrollHandle**: 要读写滚动位置(滚动条、"跳到底部"), 在视图里存一个句柄并绑上去:

```rust
scroll_handle: UniformListScrollHandle, // 滚动句柄: 记录/控制列表的滚动位置

// 渲染时绑定
uniform_list("items", count, cx.processor(...))
    .track_scroll(&self.scroll_handle) // 绑定句柄, 之后可以 scroll_to_item/scroll_to_bottom
```

句柄常用方法: `scroll_to_item(ix, ScrollStrategy::Top/Center/Bottom/Nearest)`(把某行滚进视野)、`scroll_to_bottom()`、`logical_scroll_top_index()`(当前第一可见行号).

**cx.processor() 回调生成列表项**: `cx.processor(|this, range, window, cx| ...)` 把「带视图状态的闭包」变成 `uniform_list` 需要的回调: 闭包第一个参数 `this` 就是 `&mut self`, 可以读写视图字段(比如顺手记下 `this.visible_range`). 不需要状态时, 直接传普通闭包也行.

**滚动优化原理**: 渲染每一帧时, `uniform_list` 根据滚动偏移算出可见行号区间, 回调只为这个区间创建元素; 滚出视野的行直接丢弃, 滚进来的新行按同一模板重建. 由于所有行等高, 行高 × 行号就是行的位置, 不需要逐行测量.

**NOTE**: 这版GPUI**没有** `UniformListState` 这个类型, 也不接收行高参数(如 `UniformListState::new(cx.entity(), 40.0)`) —— 行高由第一个元素的测量结果决定, 滚动状态用 `UniformListScrollHandle`(可选, 见上).

## 15.2 list: 变高行列表

行高不一致时(聊天消息、评论列表)用 `list`. 它通过 `ListState` 管理, 每行渲染出来后动态测量实际高度:

```rust
use gpui::{ListAlignment, ListState, ...};

const ITEM_COUNT: usize = 40;

struct BottomListDemo {
    list_state: ListState, // 变高行列表的滚动/测量状态, 存在视图里
}

impl BottomListDemo {
    fn new() -> Self {
        Self {
            // 参数: 行数 40, 底部对齐, overdraw 500px(视口外多渲染的余量, 防止滚动露白)
            list_state: ListState::new(ITEM_COUNT, ListAlignment::Bottom, px(500.)).measure_all(),
        }
    }
}
```

渲染回调一次只给**一行**:

```rust
list(self.list_state.clone(), |index, _window, _cx| {
    let height = px(30. + (index % 5) as f32 * 10.); // 每行高度不同, 模拟变高行
    div()
        .h(height)                // 行高: height: 30px ~ 70px
        .w_full()                 // 宽度撑满: width: 100%
        .flex()                   // 弹性布局: display: flex
        .items_center()           // 垂直居中: align-items: center
        .px_3()                   // 左右内边距: padding-left: 0.75rem; padding-right: 0.75rem
        .border_b_1()             // 底部 1px 边框: border-bottom-width: 1px
        .border_color(rgb(0xEEEEEE)) // 边框颜色: border-color: #eeeeee
        .child(format!("Item {index}"))
        .into_any() // list 要求每行返回 AnyElement
})
.flex_1() // 列表占满剩余高度: flex: 1 1 0%
```

**ListState 的创建与测量**:

| 方法 | 说明 |
| :--- | :--- |
| `ListState::new(item_count, alignment, overdraw)` | 创建: 行数、对齐方向、视口外预渲染余量 |
| `.measure_all()` | 首帧测量所有行, 滚动条尺寸才准(大列表首帧会慢) |
| `.with_uniform_item_height(px(30.))` | 只给高度提示, 滚动条先按提示算, 实测后逐步校准(比 `measure_all` 便宜) |
| `reset(n)` | 行数变了(增删)后重置 |
| `splice(old_range, count)` | 告诉状态"这些行换了内容", 重新测量 |
| `remeasure()`/`remeasure_items(range)` | 内容高度可能变了但行数没变时重测 |
| `scroll_to_end()`/`scroll_to_reveal_item(ix)` | 滚到底部/把某行滚进视野 |
| `set_follow_mode(FollowMode::Tail)` | 跟随尾部(聊天自动滚到新消息) |
| `is_scrolled_to_end()` | 是否已经滚到底 |

**ListAlignment::Bottom**: 起排方向从底部开始, 新消息出现在底部、历史消息向上排, 正是聊天窗口的行为; 默认 `Top` 是普通列表从上往下.

**NOTE**: `ListState::new(ListAlignment::Bottom, cx.entity())` 这种写法 **没有** —— 签名是 `ListState::new(item_count, alignment, overdraw)`, 三个参数缺一不可, 也不接收 entity; 第三个参数是视口外的预渲染余量, **不是**行高.

## 15.3 自定义滚动条

列表自带滚动, 但滚动条外观可以自己画. `ListState` 有一组滚动条辅助方法, 直接给出定位滑块所需的全部数据(取自 `list_example`):

```rust
let max_offset = self.list_state.max_offset_for_scrollbar().y; // 可滚动的最大偏移
let current_offset = -self.list_state.scroll_px_offset_for_scrollbar().y; // 当前滚动偏移
let viewport_height = self.list_state.viewport_bounds().size.height; // 视口高度

let raw_fraction = if max_offset > px(0.) {
    current_offset/max_offset // 滚动比例: 0.0 ~ 1.0
} else {
    0.
};
```

有了比例, 轨道 + 滑块就是两个普通 div: 滑块高度 = 视口高度 × 视口占内容的比例(至少 30px), 滑块顶距 = (轨道高 − 滑块高) × 比例.

要支持**拖拽滑块**, 参考 `data_table` 示例的做法: 用 `canvas()` 在绘制阶段挂鼠标事件, 拖动时改写滚动句柄的偏移:

```rust
canvas(
    |_, _, _| (), // prepaint 闭包: 布局阶段的准备工作, 这里不需要
    move |thumb_bounds, _, window, _| {
        // paint 闭包: 拿到滑块的 bounds, 在这里注册鼠标事件
        window.on_mouse_event(move |ev: &MouseDownEvent, _, _, cx| { /* 记下拖拽起点 */ });
        window.on_mouse_event(move |_: &MouseUpEvent, _, _, cx| { /* 清掉拖拽状态 */ });
        window.on_mouse_event(move |ev: &MouseMoveEvent, _, _, cx| {
            if !ev.dragging() {
                return; // 没在拖拽就不处理
            }
            // 按拖拽位置换算滚动偏移, 写回句柄
            scroll_handle.set_offset(point(px(0.), -offset_y));
        })
    },
)
.size_full(), // 画布铺满滑块: width: 100%; height: 100%
```

**NOTE**: `canvas()` 收**两个**闭包(prepaint, paint), 单闭包的 `canvas(move |bounds, _, cx| ...)` 配 `cx.paint_quad(...)` 这种写法 **没有** —— 画矩形是先用 `quad(bounds, 圆角, 背景, 边框宽, 边框色, 边框样式)` 构造 `PaintQuad`, 再 `window.paint_quad(...)` 绘制. 不需要自绘时, 像 `list_example` 那样用两个 div 拼滚动条更简单.

## 15.4 data_table: 大数据表格

`data_table` 示例是 `uniform_list` 的实战: 一万行行情数据 + 自绘滚动条. 两个关键技巧:

**Rc 共享数据避免克隆**: 大数据集用 `Rc` 让所有行引用同一份数据, 每行只克隆引用计数:

```rust
use std::rc::Rc;

struct DataTable {
    /// 用 Rc 共享同一份行情数据, 避免每行克隆整个结构
    quotes: Vec<Rc<Quote>>,
    scroll_handle: UniformListScrollHandle, // 滚动句柄: 自绘滚动条靠它读写位置
}
```

**RenderOnce 实现 TableRow**: 行元素只需要渲染一次(数据不可变), 实现 `RenderOnce`(消费 `self`)比持有状态的 `Render`(`&mut self`)更轻:

```rust
#[derive(IntoElement)] // 派生 IntoElement, TableRow 可以直接当元素塞进列表
struct TableRow {
    ix: usize,
    quote: Rc<Quote>, // 只持有 Rc 引用, 不拥有数据
}

impl RenderOnce for TableRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let color = self.quote.change_color();
        div()
            .flex()                    // 弹性布局: display: flex
            .flex_row()                // 横向排列: flex-direction: row
            .border_b_1()              // 底部 1px 边框: border-bottom-width: 1px
            .border_color(rgb(0xE0E0E0)) // 边框颜色: border-color: #e0e0e0
            .bg(if self.ix.is_multiple_of(2) {
                rgb(0xFFFFFF) // 斑马纹偶数行: background-color: #ffffff
            } else {
                rgb(0xFAFAFA) // 斑马纹奇数行: background-color: #fafafa
            })
            .py_0p5() // 上下内边距: padding-top: 0.125rem; padding-bottom: 0.125rem
            .px_2()   // 左右内边距: padding-left: 0.5rem; padding-right: 0.5rem
            .children(FIELDS.map(|(key, width)| self.render_cell(key, px(width), color)))
    }
}
```

表格体就是 14.1 的 `uniform_list`, 只是每行换成 `TableRow::new(i, quote.clone())`(克隆的是 `Rc`, 不是数据):

```rust
uniform_list(
    "items",          // 列表 id
    self.quotes.len(), // 10000 行
    cx.processor(move |this, range: Range<usize>, _, _| {
        this.visible_range = range.clone(); // 顺手记下可见区间, 显示在状态栏
        let mut items = Vec::with_capacity(range.end - range.start);
        for i in range {
            if let Some(quote) = this.quotes.get(i) {
                items.push(TableRow::new(i, quote.clone()));
            }
        }
        items
    }),
)
.size_full()               // 铺满容器: width: 100%; height: 100%
.track_scroll(&self.scroll_handle), // 绑定滚动句柄, 自绘滚动条用它读写偏移
```

## 15.5 uniform_list vs list 对比

| 特性 | uniform_list | list |
| :--- | :--- | :--- |
| 行高 | 必须等高(只量第一行) | 可变高度(逐行测量) |
| 回调粒度 | `Range<usize>` 一次一批 | `usize` 一次一行 |
| 性能 | 更高(不走完整布局) | 稍低(每行要测量) |
| 适用场景 | 文件列表、数据表格 | 聊天消息、评论、动态内容 |
| 起排方向 | 默认从上往下, `.y_flipped(true)` 让第 0 行在底部 | `ListAlignment::Top/Bottom` |
| 状态类型 | `UniformListScrollHandle`(可选) | `ListState`(必须) |
| 滚动条数据 | 从句柄里自己算 | `max_offset_for_scrollbar()` 等现成方法 |

💡 提示: 行高固定就优先 `uniform_list`, 它的性能明显更好; 只有真的需要变高行再上 `list`. 大数据配 `Rc` 共享 + `RenderOnce` 行元素, 避免每帧克隆和多余的状态管理.

## 15.6 列表方法速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **构造** | `uniform_list(id, item_count, f)` | 等高行虚拟列表 |
| **构造** | `list(state, render_item)` | 变高行列表 |
| **回调** | `cx.processor(\|this, range, window, cx\| ...)` | 带视图状态的列表项生成器 |
| **状态** | `UniformListScrollHandle` | uniform_list 滚动句柄 |
| **状态** | `ListState::new(n, align, overdraw)` `.measure_all()` `.with_uniform_item_height()` | list 状态与测量 |
| **ListState 操作** | `reset()` `splice()` `remeasure()` `scroll_to_end()` `scroll_to_reveal_item()` | 增删行与滚动 |
| **滚动条** | `max_offset_for_scrollbar()` `scroll_px_offset_for_scrollbar()` `viewport_bounds()` | 滑块定位数据 |
| **滚动句柄** | `.track_scroll(&h)` `h.scroll_to_item()` `h.scroll_to_bottom()` | uniform_list 滚动控制 |
| **对齐** | `ListAlignment::Top` `ListAlignment::Bottom` | list 起排方向 |
| **行元素** | `impl RenderOnce` + `#[derive(IntoElement)]` | 一次性渲染的轻量行 |
| **自绘** | `canvas(prepaint, paint)` | 自定义滚动条等 |

完整示例: [`cargo run --example uniform_list`](../examples/uniform_list.rs)(50 行等高列表)、[`cargo run --example list_example`](../examples/list_example.rs)(变高行 + 底部对齐 + 自绘滚动条)、[`cargo run --example data_table`](../examples/data_table.rs)(10000 行行情表格 + 可拖拽滚动条). 普通滚动容器(不需要虚拟化)见 [`cargo run --example scrollable`](../examples/scrollable.rs).

## 15.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`uniform_list`](../examples/uniform_list.rs) | 50 行等高虚拟列表(见 15.1) |
| [`list_example`](../examples/list_example.rs) | 变高行 + 底部对齐 + 自绘滚动条(见 15.2、15.3) |
| [`data_table`](../examples/data_table.rs) | 10000 行行情表格 + 可拖拽滚动条(见 15.4) |
| [`scrollable`](../examples/scrollable.rs) | 不需要虚拟化的普通滚动容器, 垂直 + 水平嵌套 |
