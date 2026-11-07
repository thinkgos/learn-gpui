# 17. 动画系统

GPUI 的动画围绕 `Animation`(时长 + 缓动)和 `AnimationExt::with_animation()`(把动画绑到元素)展开, 一行链式声明就能跑起来. 另有弹簧动画(`SpringAnimation`)和手动逐帧(`request_animation_frame`)两条进阶路线. 

## 17.1 Animation 结构体

`Animation` 描述"怎么动": 播多久、是否循环、用什么缓动. 创建后通过 `with_animation()` 挂到元素上: 

```rust
use gpui::*;
use std::time::Duration;

// 定义动画: 持续 2 秒, 无限循环
let animation = Animation::new(Duration::from_secs(2)) // 只收时长; 默认播放一次, linear 缓动
    .repeat()                      // 放完再来一轮(无限循环)
    .with_easing(bounce(ease_in_out)); // 缓动: bounce 包一层 ease_in_out, 来回弹
```

挂到元素上(取自 `animation` 示例的 SVG 旋转): 

```rust
svg()
    .size_20() // 宽高都是 5 号尺寸的正方形: width: 5rem; height: 5rem (80px)
    .path(ARROW_CIRCLE_SVG) // SVG 资源(见 17.4 的 AssetSource)
    .text_color(gpui::black()) // 图标着色: color: #000000
    .with_animation(
        "image_circle", // 动画 id: 同一元素多个动画靠 id 区分
        Animation::new(Duration::from_secs(2))
            .repeat()                      // 无限循环
            .with_easing(bounce(ease_in_out)), // 缓动函数
        |svg, delta| {
            // delta: 归一化进度 0.0 -> 1.0, 每帧回调一次
            svg.with_transformation(Transformation::rotate(percentage(delta)))
        },
    ),
```

`Animation` 的配置方法: 

| 方法 | 说明 |
| :--- | :--- |
| `Animation::new(duration)` | 创建动画, 只收时长 |
| `.repeat()` | 无限循环(默认只播一次) |
| `.repeat_synced()` | 循环且与整个 App 共享时钟(多个动画相位同步) |
| `.with_easing(fn)` | 缓动函数: `Fn(f32) -> f32`, 输入输出都是归一化进度 |
| `.with_max_fps(f32)` | 限制每秒重绘次数, 省电 |

`with_animation(id, animation, animator)` 的 `animator` 收 `(元素, delta)`, 返回改好的元素; 动画元素挂载后自动逐帧调用它, 播完一次(one-shot)停在结束状态, `repeat()` 则一直转下去. 不想要某个动画时, 用 `.when(self.active, ...)` 把整个元素(或动画)从树里拿掉. 

**NOTE**: `Animation::new` 只收 `Duration`, **没有** `Animation::new(cx.entity(), active)` 这种"实体 + 开关"签名, 也**没有** `.with_duration()`/`.ease_out()`/`.linear()`/`.repeat(AnimationRepeat::Forever)` 这些方法 —— 时长在 `Animation::new()` 里给, 循环是 `.repeat()`, 缓动统一走 `.with_easing(闭包)`. 

**NOTE**: `with_animation` 的第一个参数是**动画 id**(`impl Into<ElementId>`), 三参数缺一不可; `with_animation(animation, |div, progress| ...)` 这种两参写法 **没有**. 还要 `use gpui::AnimationExt as _;` 把 trait 引进来(它**不在** `prelude` 里). 

## 17.2 Easing 函数

缓动函数就是普通的 `fn(f32) -> f32`: 输入线性时间(0..1), 输出动画进度(可以超出 0..1, 比如弹簧过冲).GPUI内置这些: 

| 函数 | 效果 |
| :--- | :--- |
| `linear` | 匀速, 就是 delta 本身 |
| `quadratic` | 平方加速: delta * delta |
| `ease_in_out` | 缓入缓出, 最常用 |
| `ease_out_quint()` | 五次方缓出(注意带括号, 返回的是闭包) |
| `bounce(easing)` | 把任意缓动正放一遍再倒放一遍(来回弹) |
| `pulsating_between(min, max)` | 呼吸/脉动, 输出在 min..max 之间 |

**NOTE**: 缓动就是函数, **没有** `Easing` 枚举或结构体, 也**没有** `ease_in()`/`ease_out()`/`spring()` 这三个函数 —— 只有 `ease_in_out` 和 `ease_out_quint()`; "spring()" 不是缓动, 弹簧是下面这套独立 API. 

**弹簧动画**: 要"惯性过冲、中途改目标能接上速度"的物理手感, 用 `SpringAnimation`(完整演示在 [`cargo run --example animation`](../examples/animation.rs)): 

```rust
div()
    .absolute()          // 绝对定位: position: absolute
    .top_1()             // 顶部偏移: top: 0.25rem (4px)
    .size(px(24.0))      // 宽高 24px: width: 24px; height: 24px
    .rounded_full()      // 圆形: border-radius: 9999px
    .with_spring(
        "spring-position", // 动画 id: id 会记住弹簧的位置和速度, 改目标值平滑接力
        SpringAnimation::new(SpringConfig::new(170.0, spring_damping, 1.0)) // 刚度、阻尼、质量
            .to(spring_position) // 目标位置
            .with_epsilon(0.25), // 收敛阈值: 距目标小于它就停
        |this, left| this.left(left), // 每帧把弹簧输出的位置写进样式
    ),
```

## 17.3 变换 Transformation

`Transformation` 是 SVG 的绘制变换, 通过 `.with_transformation()` 应用: 只影响绘制, **不改布局也不影响命中测试**(鼠标判定还是按原位置). 

| 构造 | 说明 |
| :--- | :--- |
| `Transformation::scale(size(1.2, 1.2))` | 缩放: 两个轴的倍率 |
| `Transformation::translate(point(px(10.), px(0.)))` | 平移: (x, y) 像素偏移 |
| `Transformation::rotate(percentage(delta))` | 旋转: 角度是 `Radians` |
| `.with_scaling()/.with_translation()/.with_rotation()` | 在一个变换上继续组合其他分量 |

`percentage(x)` 把 0..1 映射成整圈的比例(0.25 是 90°, 1.0 是 360°), 因此动画里 `Transformation::rotate(percentage(delta))` 正好转一整圈. 旋转、缩放、平移可以经 `with_*` 组合在一个变换里. 

对 div 而言没有 `Transformation`, 动画闭包里改的是**样式**: 

```rust
// 动画驱动的是样式值: 每帧把进度换算成 left
|div, delta| div.left(px(delta * 200.0)), // 位移 200px: left: 0px -> 200px
```

**NOTE**: `Transformation` 一族是 `Svg` 的方法(`.with_transformation()`), **没有**通用元素上的 `.scale()`/`.rotate()`/`.translate()` 链式变换; div 动画请在闭包里改样式(`.left()`/`.w()`/`.bg()` 等, 弹簧示例就是这么做的). 

## 17.4 AssetSource 加载动画资源

动画里的 SVG(转圈图标、加载指示器)通过 `AssetSource` 加载, 约定见第 6 章. `animation` 示例从文件系统读资源: 

```rust
struct Assets {}

impl AssetSource for Assets {
    // 加载资源: 返回文件字节(图片/SVG 都是二进制, 不是字符串)
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        std::fs::read(path)
            .map(Into::into)
            .map_err(Into::into)
            .map(Some)
    }

    // 列出某目录下的资源名
    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(std::fs::read_dir(path)?
            .filter_map(|entry| {
                Some(SharedString::from(entry.ok()?.path().to_string_lossy().into_owned()))
            })
            .collect())
    }
}

// 应用启动时注册: 之后 svg().path("...") 就会走 Assets::load
application().with_assets(Assets {}).run(|cx: &mut App| { /* ... */ });
```

**NOTE**: `AssetSource` 只有 `load(&self, path)`/`list(&self, path)` 两个方法(见 6.4), **没有** `load_asset(&mut self, path, cx) -> Task<...>` 这种异步签名. 资源加载是同步取字节, 异步解码由GPUI内部处理. 

## 17.5 手动动画: window.request_animation_frame()

复杂的逐帧逻辑(粒子、游戏循环)可以自己驱动. 两个 API 配合: 

| 方法 | 说明 |
| :--- | :--- |
| `window.request_animation_frame()` | 请求下一帧重绘(**无参数, 没有回调**) |
| `window.on_next_frame(闭包)`/`cx.on_next_frame(window, 闭包)` | 当前帧渲染完后执行闭包, 在里面预约下一帧 |

手动循环的骨架: 在回调里推进状态、`cx.notify()` 触发重绘, 再预约下一帧: 

```rust
impl Spinner {
    /// 每帧把角度推进一点, 然后预约下一帧, 形成动画循环
    fn tick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.running {
            return; // 开关关掉就不再续约, 动画自然结束
        }
        self.angle = (self.angle + 6.0) % 360.0; // 每帧转 6 度
        cx.notify(); // 通知 GPUI: 状态变了, 需要重新渲染
        cx.on_next_frame(window, |this, window, cx| {
            this.tick(window, cx); // 下一帧继续
        });
    }
}
```

渲染时直接用 `self.angle` 换算样式即可; `request_animation_frame()` 只在"需要一帧重绘但没有自己的回调"时用(它内部就是通知当前视图刷新). 

**NOTE**: `window.request_animation_frame()` **没有** `(cx.entity(), 闭包)` 这种带回调的签名 —— 回调请用 `cx.on_next_frame(window, ...)` 或 `window.on_next_frame(...)`. 另外装饰性动画(spinner、脉动)优先用 `with_animation`, 它会自动尊重系统的"减少动态效果"设置(`App::reduce_motion`); 手动循环时要自己检查这个状态, 跳过帧请求. 

💡 提示: 动画时长的常见选择: 界面过渡 150-300ms, 加载指示 800-1200ms 循环. 同时运行的动画越多帧率压力越大, 长列表里避免给每行都挂动画(配合第 15 章的虚拟化只让可见行动起来).

## 17.6 动画方法速查表

| 类别 | 方法/类型 | 说明 |
| :--- | :--- | :--- |
| **动画定义** | `Animation::new(duration)` `.repeat()` `.repeat_synced()` | 时长与循环 |
| **动画配置** | `.with_easing(fn)` `.with_max_fps(f32)` | 缓动与帧率上限 |
| **挂到元素** | `AnimationExt::with_animation(id, anim, \|el, delta\| ...)` | 元素动画(需 `use gpui::AnimationExt as _`) |
| **弹簧** | `with_spring(id, SpringAnimation, \|el, v\| ...)` | 物理弹簧动画 |
| **弹簧配置** | `SpringConfig::new(stiffness, damping, mass)` `.to(target)` `.with_epsilon(f32)` | 刚度/阻尼/目标/收敛阈值 |
| **缓动函数** | `linear` `quadratic` `ease_in_out` `ease_out_quint()` `bounce()` `pulsating_between()` | `fn(f32) -> f32` |
| **变换** | `Transformation::scale/translate/rotate` + `svg().with_transformation()` | SVG 绘制变换 |
| **角度** | `percentage(x)` | 0..1 整圈比例 -> `Radians` |
| **资源** | `impl AssetSource` + `application().with_assets()` | 加载 SVG 等资源(见 6.4) |
| **手动逐帧** | `window.request_animation_frame()` `cx.on_next_frame()` `window.on_next_frame()` | 自己驱动动画循环 |

完整示例: [`cargo run --example animation`](../examples/animation.rs)(SVG 旋转 + 弹簧演示 + 可拖动阻尼).

## 17.7 配套示例

本章涉及的示例都在仓库 [`examples/`](../examples/) 目录下, 在仓库根目录 `cargo run --example <示例名>` 运行(桌面 + wasm 双入口):

| 示例 | 说明 |
| :--- | :--- |
| [`animation`](../examples/animation.rs) | SVG 旋转动画 + 弹簧动画演示 + 可拖动阻尼控制(见 17.1–17.4) |
