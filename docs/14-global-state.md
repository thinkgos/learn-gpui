# 14. 全局状态

有些数据(主题、排版参数、用户偏好)不属于任何单个组件, 而是整个应用共享一份. GPUI用 `Global` trait + `App` 上的全局存储实现这种单例状态: 每个类型只有一份, 任何地方都能读写, 还能订阅它的变化.

## 14.1 Global trait: 跨组件共享状态

`Global` 是一个**空的标记 trait**, 只声明"这个类型可以作为全局状态存进 `App`", 没有任何方法要实现 —— 所以实现体就是空的:

```rust
use gpui::*;

pub struct MySettings {
    pub theme: String,
    pub font_size: f32,
}

/// 空实现, 只为满足 Global trait 的标记:
/// 声明这个类型可以作为全局状态存进 App
impl Global for MySettings {}
```

全局存储按**类型**去重:  一个类型全局只有一份, `cx.set_global()` 再调一次就是整体替换. 想要"多个同类型实例", 用 newtype 包一层(见 14.4), 或者干脆改用 Entity.

## 14.2 cx.set_global()/cx.global()/cx.try_global()/cx.update_global()

`App`/`Context` 上的全局状态访问方法:

| 方法 | 说明 |
| :--- | :--- |
| `cx.set_global(g)` | 设置/替换全局值, 没有就创建 |
| `cx.global::<G>()` | 只读引用, **未初始化会 panic** |
| `cx.try_global::<G>()` | 只读, 返回 `Option<&G>`, 不会 panic |
| `cx.has_global::<G>()` | 是否已初始化(返回 `bool`) |
| `cx.global_mut::<G>()` | 可变引用, 未初始化会 panic |
| `cx.default_global::<G>()` | 可变引用, 没有就用 `Default` 先创建 |
| `cx.update_global(\|g, cx\| ...)` | 闭包里可变访问, 闭包还能顺带用上下文 |
| `cx.update_default_global(\|g, cx\| ...)` | 同上, 没有就先按 `Default` 创建 |
| `cx.remove_global::<G>()` | 移除并返回全局值 |

```rust
// 启动时注册全局状态(没有就创建, 有就整体替换)
cx.set_global(MySettings {
    theme: "dark".into(),
    font_size: 14.0,
});

// 任意组件里读取(未初始化会 panic, 所以要先 set_global)
let settings = cx.global::<MySettings>();

// 只是想确认存在时用 try_global, 不 panic
if let Some(settings) = cx.try_global::<MySettings>() {
    let _ = &settings.theme;
}

// 更新: 闭包拿到 &mut 全局值和上下文
cx.update_global(|settings: &mut MySettings, _cx| {
    settings.theme = "high_contrast".into();
});
```

**NOTE**: turbofish 的正确写法是 `cx.global::<G>()`/`cx.observe_global::<G>(...)`. 另外 `set_global`、`global_mut`、`update_global`、`default_global`、`remove_global` 改完(或访问完)都会触发全局观察者(见 14.3), `update_global` **即使闭包里什么都没改也会触发**.

## 14.3 cx.observe_global(): 观察全局状态变化

`cx.observe_global::<G>()` 订阅某个全局类型的变化, 返回 `Subscription`(保存或 `.detach()` 的规则和第 12 章一样). 实体上和 App 上的回调形状不同:

| 方法 | 回调形状 |
| :--- | :--- |
| `cx.observe_global::<G>(h)` (`Context<T>`) | `(\|this, cx\| ...)` 拿到订阅者自身 |
| `cx.observe_global::<G>(h)` (`App`) | `(\|cx\| ...)` 不绑定订阅者 |

```rust
pub struct ThemeAware {
    current_theme: String,
    // 订阅存进字段: 组件释放时自动退订
    _global_subscription: Subscription,
}

impl ThemeAware {
    fn new(cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe_global::<MySettings>(|this, cx| {
            // 任何地方 set_global/update_global 修改 MySettings 后触发
            this.current_theme = cx.global::<MySettings>().theme.clone();
            cx.notify(); // 通知 GPUI: 状态变了, 需要重新渲染
        });

        ThemeAware {
            current_theme: cx.global::<MySettings>().theme.clone(),
            _global_subscription: subscription,
        }
    }
}
```

和事件一样, 观察者回调是**延迟派发**的: 修改全局状态只是把通知排进 effect 队列, 本轮 update 结束时才统一触发.

## 14.4 实战: [`text.rs`](../examples/text.rs) 里的字体全局状态管理

本仓库的 [`examples/text.rs`](../examples/text.rs) 用全局状态管理整套排版参数(字号、行高、字号阶梯), 是 Global 的经典用法. 关键片段:

```rust
/// 一组全局排版参数
#[derive(Clone, Debug)]
pub struct TextContext {
    /// 基础字号(像素). 对应 font-size: 16px
    font_size: f32,
    /// 行高倍数. 对应 line-height: 1.3(无单位, 表示字号的倍数)
    line_height: f32,
    /// 字号阶梯比例, 每升一级字号就乘以它
    type_scale: f32,
}

/// 把 TextContext 包一层, 好注册成GPUI的全局状态(Global).
/// Arc 让多处共享同一份设置, 不用反复克隆.
#[derive(Clone, Debug)]
pub struct GlobalTextContext(pub Arc<TextContext>);

/// 空实现, 只为满足 Global trait 的标记: 声明这个类型可以作为全局状态存进 App
impl Global for GlobalTextContext {}
```

启动时注册, 任意组件里读取:

```rust
// 启动时: 把排版设置注册成全局状态, 供所有组件读取
cx.set_global(GlobalTextContext(Arc::new(TextContext::default())));

// 读取: text.rs 还给 GlobalTextContext 实现了 Deref,
// 所以能直接 .font_size 取字段, 不用写 .0.font_size
let font_size = cx.global::<GlobalTextContext>().font_size;
```

[`text.rs`](../examples/text.rs) 里还包了一个便捷 trait, 让 `App` 能直接写 `cx.text_context()`:

```rust
pub trait ActiveTextContext {
    fn text_context(&self) -> &Arc<TextContext>;
}

impl ActiveTextContext for App {
    fn text_context(&self) -> &Arc<TextContext> {
        &self.global::<GlobalTextContext>().0
    }
}
```

## 14.5 全局状态 vs Entity 的选择

| 维度 | Entity | Global |
| :--- | :--- | :--- |
| 数量 | 任意多个实例 | 每个类型只有一份 |
| 变化通知 | `cx.observe()`/事件订阅(第 12 章) | `cx.observe_global()` |
| 生命周期 | 随句柄释放而销毁 | 存活到应用退出(或手动 `remove_global`) |
| 典型用途 | 组件自身状态、列表项、窗口内容 | 主题、排版参数、用户偏好、服务单例 |

选择原则:  **优先用 Entity**, 只有"确实全应用只有一份、且不挂靠任何组件"的数据才用 Global. 全局状态的滥用风险:

- 紧耦合:  任何地方都能 `cx.global()` 直读, 组件之间形成隐式依赖, 可维护性下降.
- 测试困难:  单测之间共享同一份全局值, 需要额外的设置/清理.
- 一类型一实例:  多个模块想各存一份同类型数据就会互相覆盖, 要么 newtype 包装, 要么改用 Entity.

## 14.6 全局状态速查表

| 类别 | 方法 | 说明 |
| :--- | :--- | :--- |
| **声明** | `impl Global for X {}` | 空的标记实现, X 可作为全局状态 |
| **写入** | `cx.set_global(g)` | 设置/替换, 触发观察者 |
| **读取** | `cx.global::<G>()` `cx.try_global::<G>()` `cx.has_global::<G>()` | panic 版/Option 版/是否存在 |
| **修改** | `cx.global_mut::<G>()` `cx.update_global(f)` `cx.default_global::<G>()` | 可变访问, 后两者支持"没有就建默认" |
| **移除** | `cx.remove_global::<G>()` | 移除并取出值 |
| **观察** | `cx.observe_global::<G>(h)` | 变化时回调, 返回 `Subscription` |

完整示例:  [`cargo run --example text`](../examples/text.rs)(顶部按钮切换字体族, 字号阶梯和字符网格共用一份全局排版参数).

## 14.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`text`](../examples/text.rs) | `GlobalTextContext` 统一管理字号、行高、字号阶梯, 顶部按钮切换字体族(见 14.4) |
