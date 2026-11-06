# GPUI教程

`docs/` 是一套GPUI中文教程, 按「先跑起来 → 样式与交互 → 状态与组件 → 进阶」的顺序编排, 每章都能独立查阅, 也适合从第 0 章顺着读.

所有 API 以 zed 仓库的GPUI为准(依赖见根目录 `Cargo.toml`), 代码片段贴合 [`examples/`](../examples/) 下**能编译**的示例, 每章结尾给出对应的运行命令. 遇到「其他教程里有、这里说没有」的写法, 以各章的 `**NOTE**` 为准.

## 怎么跑示例

```bash
cargo run --example hello_world   # 跑第 0 章的最小程序
cargo run --example <示例名>       # <示例名> 见下表最后一列
```

示例源码都在 [`examples/`](../examples/) 目录下, 资源文件在 `assets/`、[`examples/image/`](../examples/image/) 等处.

## 入门

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [0](01-helloword.md) | Hello World — 你的第一个GPUI程序 | 三步骨架: 创建应用 → 打开窗口 → 运行事件循环; 认识 `Application`/`App`/`Window`/`Entity`/`Render` | [`hello_world`](../examples/hello_world.rs) |

## 基础: 描述界面

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [1](02-core-concept.md) | 核心概念 | `App`/`Window`/`Context`/`Entity`/`Element`/`Global`/`Render` 各是什么, 数据怎么流, 为什么必须 `cx.notify()` | [`hello_world`](../examples/hello_world.rs) |
| [2](03-layout.md) | 布局系统 | 长度单位、Flexbox、CSS Grid、定位、间距、尺寸 —— Taffy 布局 + Tailwind 风格链式 API | [`grid_layout`](../examples/grid_layout.rs) |
| [3](04-style-appearance.md) | 样式与外观 | 颜色、边框、圆角、阴影、渐变、透明度、溢出、cursor | [`shadow`](../examples/shadow.rs) [`gradient`](../examples/gradient.rs) [`pattern`](../examples/pattern.rs) [`opacity`](../examples/opacity.rs) |
| [4](05-text.md) | 文字和排版 | 字号字重颜色、文本溢出、文本布局、自定义字体、字符网格 | [`text`](../examples/text.rs) [`text_layout`](../examples/text_layout.rs) [`text_wrapper`](../examples/text_wrapper.rs) |

## 内容与交互

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [5](06-media.md) | 图像与媒体 | `img()` 三种来源、SVG 单色蒙版、GIF 动图、AssetSource、图片缓存、ObjectFit | [`image_loading`](../examples/image_loading.rs) [`image_gallery`](../examples/image_gallery.rs) [`gif_viewer`](../examples/gif_viewer.rs) |
| [6](07-event.md) | 事件管理 | 点击、鼠标事件、悬停、点击外部检测、捕获/冒泡、鼠标压力 | [`mouse_pressure`](../examples/mouse_pressure.rs) [`popover`](../examples/popover.rs) |
| [7](08-focus.md) | 焦点管理 | FocusHandle、focus/focus_visible、Tab 导航、tab_index/tab_stop、焦点事件 | [`focus_visible`](../examples/focus_visible.rs) [`tab_stop`](../examples/tab_stop.rs) |
| [8](09-action-shortcut.md) | 动作和快捷键 | `actions!` 宏、带数据的动作、键位绑定、动作分发、菜单栏 | [`set_menus`](../examples/set_menus.rs) [`input`](../examples/input.rs) |

## 组件与状态

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [9](10-text-input.md) | 文本输入 | 自己拼一个输入框: `EntityInputHandler` + 自定义 Element 三阶段渲染 + 光标选区剪贴板 | [`input`](../examples/input.rs) |
| [10](11-dag-drop.md) | 拖拽功能 | `.on_drag()`/`.on_drop()`/`.drag_over()` 与拖拽生命周期 | [`drag_drop`](../examples/drag_drop.rs) |
| [11](12-entity-state.md) | Entity 状态管理 | 实体的创建读写、`observe`/`notify`/`listener`、所有权规则、WeakEntity | [`ownership_post`](../examples/ownership_post.rs) [`move_entity_between_windows`](../examples/move_entity_between_windows.rs) |
| [12](13-event-pubsub.md) | 事件发布订阅 | 实体之间的自定义事件: `EventEmitter` + `emit()` + `subscribe()`, 多级转发 | [`ownership_post`](../examples/ownership_post.rs) |
| [13](14-global-state.md) | 全局状态 | `Global` trait 与 `set_global`/`global`/`update_global`/`observe_global`, 何时用全局、何时用实体 | [`text`](../examples/text.rs) |

## 进阶

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [14](15-list-virtualization.md) | 列表与虚拟化 | `uniform_list`(等高)、`list`(变高)、`data_table`(大数据表格), 十万行流畅滚动 | [`uniform_list`](../examples/uniform_list.rs) [`list_example`](../examples/list_example.rs) [`data_table`](../examples/data_table.rs) [`scrollable`](../examples/scrollable.rs) |
| [15](16-popover.md) | 浮层 Popover | `deferred()` + `anchored()`、点击外部关闭、多级嵌套、Popover vs Modal | [`popover`](../examples/popover.rs) [`anchor`](../examples/anchor.rs) |
| [16](17-animation.md) | 动画系统 | `Animation` + `with_animation()`、缓动函数、弹簧动画、手动逐帧 | [`animation`](../examples/animation.rs) |
| [17](18-painting.md) | 底层绘图 | `canvas()`、`PathBuilder` 路径、曲线、`paint_path`/`paint_quad`、鼠标绘图 | [`painting`](../examples/painting.rs) [`paths_bench`](../examples/paths_bench.rs) |
| [18](19-gradient-pattern.md) | 渐变与图案 | `linear_gradient`、色彩空间(Srgb/Oklab)、`pattern_slash`/`checkerboard` | [`gradient`](../examples/gradient.rs) [`pattern`](../examples/pattern.rs) |

## 窗口

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [19](20-window.md) | 窗口管理 | `WindowOptions` 全字段、`WindowKind`、窗口操作、`window.prompt()` 系统对话框、监听窗口变化 | [`window`](../examples/window.rs) [`on_window_close_quit`](../examples/on_window_close_quit.rs) [`system_notifications`](../examples/system_notifications.rs) |
| [20](21-multi-display.md) | 多屏定位 | `cx.displays()`、`DisplayId`、`Bounds` 方位方法、窗口居中与摆放 | [`window_positioning`](../examples/window_positioning.rs) |
| [21](22-window-decorations.md) | 自定义窗口装饰 | `WindowDecorations`、自绘标题栏、`ResizeEdge` 缩放手柄、客户区内边距、窗口阴影 | [`window_shadow`](../examples/window_shadow.rs) [`window_movable`](../examples/window_movable.rs) |

## 阅读路线

- **只想尽快做出界面**: 0 → 1 → 2 → 3 → 6
- **做列表/表格类应用**: 0 → 1 → 11 → 14, 需要拖拽排序再看 10
- **做编辑器/输入框**: 0 → 7 → 8 → 9
- **做定制外观的应用**: 3 → 16 → 17 → 19 → 21

## 术语对照

| 文档用语 |GPUI里的东西 |
| :--- | :--- |
| 链式调用 | `Styled` 等 trait 上的 Tailwind 风格方法, 逐个调用拼出元素 |
| 实体(Entity) | 框架托管的强类型状态容器, 改状态后要 `cx.notify()` |
| 视图(View) | 实现了 `Render` 的实体 |
| 元素(Element) | `render` 每帧构建的一次性值, 用完即弃 |
| 句柄(Handle) | `FocusHandle` 这类轻量标识, 不持有数据 |
| 上下文(cx) | `App`/`Context<T>`/`AsyncApp`, 操作实体和全局状态的入口 |
