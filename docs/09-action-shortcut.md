# 9. 动作和快捷键

动作(`Actions`)是GPUI中解耦组件间通信的核心机制. 通过动作系统, 可以将键盘快捷键、菜单项和按钮点击统一关联到同一个处理逻辑. 

## 9.1 actions! 宏

使用 `actions!` 宏定义动作类型. 它为每个名字生成一个**零大小的单元结构体**并自动注册: 

```rust
// 带命名空间的形式(推荐): 生成 example::Quit 等动作
actions!(example, [Quit, CloseWindow]);

// 不带命名空间的形式
actions!([Quit]);
```

展开后等价于: 

```rust
#[derive(Clone, PartialEq, Default, Debug, gpui::Action)]
#[action(namespace = example)]
pub struct Quit; // 单元结构体: 无字段, 无数据
```

**NOTE**: `actions!` 生成的是**不带数据**的单元结构体. 旧文档"可携带任意数据"、"生成 `new()` 构造函数"、"字段自动生成 getter"的说法都是错的 —— 它什么都不生成, 就是一个类型名. 

## 9.2 带数据的动作

需要携带数据的动作, 直接在普通结构体上 `#[derive(Action)]`, 字段就是普通字段, 构造用结构体字面量: 

```rust
#[derive(Clone, PartialEq, Debug, serde::Deserialize, schemars::JsonSchema, Action)]
pub struct SelectNext {
    pub replace_newest: bool, // 普通字段, 直接读写
}

// 构造就是普通结构体字面量
let action = SelectNext { replace_newest: true };
```

带字段的 derive 要求结构体同时实现 serde 与 schemars 相关 trait; 不需要 JSON 序列化时在结构体上加 `#[action(no_json)]`. 可用属性还有 `namespace = ...`、`name = "..."`、`deprecated = "..."` 等. 

## 9.3 键盘绑定

使用 `cx.bind_keys()` 和 `KeyBinding::new()` 将快捷键绑定到动作: 

```rust
cx.bind_keys([
    KeyBinding::new("cmd-w", CloseWindow, None), // 快捷键 -> 动作
    KeyBinding::new("cmd-q", Quit, None),
]);
```

`KeyBinding::new(keystrokes, action, context)` 收三个参数: 

| 参数 | 说明 |
| :--- | :--- |
| `keystrokes: &'static str` | 快捷键描述串(语法见下表) |
| `action: A` | 动作**值**(如 `Quit`), 不是引用 |
| `context: Option<&str>` | 键位生效的上下文谓词, 如 `Some("editor")`, `None` 为全局 |

快捷键语法是 `[修饰键-...]键名`, 修饰键可以多个连写: 

| 修饰键 | 格式 | 说明 |
| :--- | :--- | :--- |
| **Ctrl** | `ctrl-` | Control 键 |
| **Shift** | `shift-` | Shift 键 |
| **Alt** | `alt-` | Option/Alt 键 |
| **Cmd** | `cmd-` | 平台主键(macOS 是 Command) |
| **别名** | `super-` `win-` | 同 `cmd-` |
| **Fn** | `fn-` | Fn 键 |
| **跨平台** | `secondary-` | macOS 映射 `cmd-`, 其他平台映射 `ctrl-`(见下方 NOTE) |

键名直接写小写字符或键位名:  `"ctrl-c"`、`"cmd-s"`、`"shift-escape"`、`"f12"`、`"tab"`、`"enter"`、`"space"`、`"left"` 都合法; 大写字母隐含 Shift(`"ctrl-X"` 等价 `ctrl-shift-x`); 尾部的 `"-"` 表示减号键. **多键序列**用空格分隔:  `"ctrl-b h"` 先按 ctrl-b 再按 h. 

**NOTE**: `KeyBinding::new()` 里的快捷键串写错会**直接 panic**, 写完跑一下确认能启动. 上表的 `secondary-` 也**没有**办法用在 `KeyBinding::new()` 里(它按平台无关方式解析, 遇到 `secondary-` 直接报错 panic), 只有 `KeyBinding::load` 这类按宿主 OS 解析的入口才支持;  要跨平台就自己写 `cfg!(target_os = "macos")` 分支挑一套键位. 

上下文绑定配合元素上的 `.key_context()`: 

```rust
// 只在 key_context 为 "TextInput" 的子树里生效
cx.bind_keys([KeyBinding::new("backspace", Backspace, Some("TextInput"))]);

div().key_context("TextInput").track_focus(&self.focus_handle) ...
```

## 9.4 监听动作

使用 `.on_action()` 在组件内监听动作. 动作会沿焦点路径冒泡, 直到被处理: 

```rust
actions!(example, [CloseWindow]);

// 元素上监听: 闭包 (action, window, cx)
div()
    .track_focus(&self.focus_handle) // 焦点在哪, 动作就从哪冒泡
    .on_action(|_: &CloseWindow, window, _cx| {
        window.remove_window(); // 处理动作
    })
```

全局处理(不在元素上)用 `cx.on_action`, 闭包只有两个参数: 

```rust
cx.on_action(|_: &Quit, cx| cx.quit()); // 全局处理
```

事件的传播顺序(和第 7 章同一套捕获/冒泡机制):  捕获(根 → 焦点元素) → 冒泡(焦点元素 → 根) → 全局处理器. 

**NOTE**: 冒泡阶段的动作处理器**默认消费动作**(阻止继续冒泡). 想让父元素或全局 `cx.on_action` 也收到, 处理器里要调 `cx.propagate()`; 反过来 `cx.stop_propagation()` 可以提前截断. 全局处理器排在冒泡最后, 只有前面没人处理(或有人 `propagate()`)才轮到它. 

## 9.5 手动分发动作

使用 `cx.dispatch_action()` 可以手动分发动作到当前焦点元素, 动作会沿着焦点路径向上冒泡: 

```rust
// 发给当前焦点元素(最常用)
cx.dispatch_action(&CloseWindow);

// 另外两个入口:
// window.dispatch_action(Box::new(CloseWindow), cx)  —— 窗口级, 发给焦点元素
// focus_handle.dispatch_action(&CloseWindow, window, cx) —— 发给指定句柄的元素
```

`dispatch_action()` 的分发机制: 

- 动作从当前焦点元素开始
- 沿着视图树向上冒泡到父元素
- 第一个处理该动作的 `.on_action()` 会消费它
- 未被处理的动作会被丢弃

`cx.is_action_available(&action)` 可以探测当前焦点上下文里有没有处理器(用于菜单置灰). 

## 9.6 菜单与动作

菜单项就是"另一个动作触发器" —— 通过 `cx.set_menus()` 设置菜单栏, 每个菜单项挂一个动作, 点菜单等价于分发动作: 

```rust
use gpui::*;

actions!(set_menus, [Quit, ToggleCheck]);

cx.on_action(|_: &Quit, cx| cx.quit());
cx.on_action(|_: &ToggleCheck, _cx| { /* 切换状态 */ });

cx.set_menus([Menu::new("File").items([
    MenuItem::action("Toggle Check", ToggleCheck).checked(true), // 可勾选项
    MenuItem::separator(),                                     // 分隔线
    MenuItem::action("Disabled", gpui::NoAction).disabled(true), // 置灰项(NoAction 占位)
    MenuItem::submenu(Menu::new("Mode").items([                 // 子菜单
        MenuItem::action("List", ToggleCheck),
    ])),
    MenuItem::action("Quit", Quit),
])]);
```

`gpui::NoAction` 是一个"什么都不做"的内置动作, 常用来做置灰占位菜单项. 完整示例: [`cargo run --example set_menus`](../examples/set_menus.rs). 

💡 提示:  同一个动作接三个触发源 —— 快捷键(`KeyBinding`)、菜单(`MenuItem::action`)、按钮(`cx.dispatch_action`), 处理逻辑只写一份(`.on_action`), 这就是动作系统的设计初衷. 键盘 + 焦点 + 动作串起来的完整小应用见 [`cargo run --example focus_visible`](../examples/focus_visible.rs) 和 [`cargo run --example input`](../examples/input.rs)(文本编辑的全套动作绑定). 

## 9.7 动作方法速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **定义** | `actions!(ns, [A, B])` | 生成无数据单元动作并注册 |
| **带数据** | `#[derive(Action)]` 结构体 | 字段即数据, 结构体字面量构造 |
| **绑定** | `cx.bind_keys([KeyBinding::new(keys, action, ctx)])` | 快捷键绑定(三参数) |
| **键位语法** | `"cmd-q"` `"ctrl-b h"` `"shift-escape"` `"f12"` | 修饰键前缀 + 键名, 空格分隔多键序列 |
| **上下文** | `div().key_context("name")` + `Some("name")` | 限定键位生效范围 |
| **监听** | `.on_action(l)` `cx.on_action(l)` `.capture_action(l)` | 元素监听/全局监听/捕获阶段 |
| **分发** | `cx.dispatch_action(&action)` `window.dispatch_action()` `handle.dispatch_action()` | 手动分发到焦点/指定元素 |
| **传播** | `cx.propagate()` `cx.stop_propagation()` | 动作处理器默认消费, propagate 放行 |
| **菜单** | `cx.set_menus()` `MenuItem::action()` `.checked()` `.disabled()` | 菜单触发同一套动作 |

## 9.8 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`set_menus`](../examples/set_menus.rs) | 菜单栏(List/Grid 等)绑定动作, `NoAction` 置灰占位(见 9.6) |
| [`focus_visible`](../examples/focus_visible.rs) | 键盘、焦点和动作串起来的小应用(见 9.6) |
| [`input`](../examples/input.rs) | 文本编辑的全套动作绑定(见 9.6; 输入框本体见第 10 章) |
