#GPUI教程目录

`docs/` 是一套GPUI中文教程, 按「先跑起来 → 样式与交互 → 状态与组件 → 进阶」的顺序编排, 每章都能独立查阅, 也适合从第 0 章顺着读.

所有 API 以 zed 仓库的GPUI为准(依赖见根目录 `Cargo.toml`), 代码片段贴合 `examples/` 下**能编译**的示例, 每章结尾给出对应的运行命令. 遇到「其他教程里有、这里说没有」的写法, 以各章的 `**NOTE**` 为准.

## 怎么跑示例

```bash
cargo run --example hello_world   # 跑第 0 章的最小程序
cargo run --example <示例名>       # <示例名> 见下表最后一列
```

示例源码都在 `examples/` 目录下, 资源文件在 `assets/`、`examples/image/` 等处.

## 入门

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [0](00-helloword.md) | Hello World — 你的第一个GPUI程序 | 三步骨架: 创建应用 → 打开窗口 → 运行事件循环; 认识 `Application`/`App`/`Window`/`Entity`/`Render` | `hello_world` |

## 基础: 描述界面

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [1](01-core-concept.md) | 核心概念 | `App`/`Window`/`Context`/`Entity`/`Element`/`Global`/`Render` 各是什么, 数据怎么流, 为什么必须 `cx.notify()` | `hello_world` |
| [2](02-layout.md) | 布局系统 | 长度单位、Flexbox、CSS Grid、定位、间距、尺寸 —— Taffy 布局 + Tailwind 风格链式 API | `grid_layout` |
| [3](03-style-appearance.md) | 样式与外观 | 颜色、边框、圆角、阴影、渐变、透明度、溢出、cursor | `shadow` `gradient` `pattern` `opacity` |
| [4](04-text.md) | 文字和排版 | 字号字重颜色、文本溢出、文本布局、自定义字体、字符网格 | `text` `text_layout` `text_wrapper` |

## 内容与交互

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [5](05-media.md) | 图像与媒体 | `img()` 三种来源、SVG 单色蒙版、GIF 动图、AssetSource、图片缓存、ObjectFit | `image_loading` `image_gallery` `gif_viewer` |
| [6](06-event.md) | 事件管理 | 点击、鼠标事件、悬停、点击外部检测、捕获/冒泡、鼠标压力 | `mouse_pressure` `popover` |
| [7](07-focus.md) | 焦点管理 | FocusHandle、focus/focus_visible、Tab 导航、tab_index/tab_stop、焦点事件 | `focus_visible` `tab_stop` |
| [8](08-action-shortcut.md) | 动作和快捷键 | `actions!` 宏、带数据的动作、键位绑定、动作分发、菜单栏 | `set_menus` `input` |

## 组件与状态

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [9](09-text-input.md) | 文本输入 | 自己拼一个输入框: `EntityInputHandler` + 自定义 Element 三阶段渲染 + 光标选区剪贴板 | `input` |
| [10](10-dag-drop.md) | 拖拽功能 | `.on_drag()`/`.on_drop()`/`.drag_over()` 与拖拽生命周期 | `drag_drop` |
| [11](11-entity-state.md) | Entity 状态管理 | 实体的创建读写、`observe`/`notify`/`listener`、所有权规则、WeakEntity | `ownership_post` `move_entity_between_windows` |
| [12](12-event-pubsub.md) | 事件发布订阅 | 实体之间的自定义事件: `EventEmitter` + `emit()` + `subscribe()`, 多级转发 | `ownership_post` |
| [13](13-global-state.md) | 全局状态 | `Global` trait 与 `set_global`/`global`/`update_global`/`observe_global`, 何时用全局、何时用实体 | `text` |

## 进阶

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [14](14-list-virtualization.md) | 列表与虚拟化 | `uniform_list`(等高)、`list`(变高)、`data_table`(大数据表格), 十万行流畅滚动 | `uniform_list` `list_example` `data_table` `scrollable` |
| [15](15-popover.md) | 浮层 Popover | `deferred()` + `anchored()`、点击外部关闭、多级嵌套、Popover vs Modal | `popover` `anchor` |
| [16](16-animation.md) | 动画系统 | `Animation` + `with_animation()`、缓动函数、弹簧动画、手动逐帧 | `animation` |
| [17](17-painting.md) | 底层绘图 | `canvas()`、`PathBuilder` 路径、曲线、`paint_path`/`paint_quad`、鼠标绘图 | `painting` `paths_bench` |
| [18](18-gradient-pattern.md) | 渐变与图案 | `linear_gradient`、色彩空间(Srgb/Oklab)、`pattern_slash`/`checkerboard` | `gradient` `pattern` |

## 窗口

| 章 | 标题 | 你会学到 | 示例 |
| :--- | :--- | :--- | :--- |
| [19](19-window.md) | 窗口管理 | `WindowOptions` 全字段、`WindowKind`、窗口操作、`window.prompt()` 系统对话框、监听窗口变化 | `window` `on_window_close_quit` `system_notifications` |
| [20](20-multi-display.md) | 多屏定位 | `cx.displays()`、`DisplayId`、`Bounds` 方位方法、窗口居中与摆放 | `window_positioning` |
| [21](21-window-decorations.md) | 自定义窗口装饰 | `WindowDecorations`、自绘标题栏、`ResizeEdge` 缩放手柄、客户区内边距、窗口阴影 | `window_shadow` `window_movable` |

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
