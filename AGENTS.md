# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 仓库概览

learn-gpui 是 GPUI(Zed 的 UI 框架)的中文学习仓库, 由两部分组成:

- `docs/` — 22 章中文教程(00–21)+ 目录 `docs/README.md`, 章节与示例一一对应.
- `examples/` — 与章节配套、能编译运行的示例, 教程里的代码片段都取自这里.

`gpui` / `gpui_platform` 是 zed 仓库的 git 依赖, 版本由 `Cargo.lock` 钉住.

## 常用命令

```bash
cargo run --example hello_world     # 跑一个示例(示例名见 docs/README.md 章节表)
cargo run                          # 跑 src/main.rs 里的最小程序
cargo check --example <名称>        # 只编译某个示例, 改完示例先跑这个

cargo fmt                          # 格式化(rustfmt 默认配置)
cargo clippy --examples            # lint

cargo test --example <名称> --features test-support         # 跑某个示例的测试
cargo test --example <名称> --features test-support <测试名> # 跑单个测试
```

示例测试写在各文件底部的 `#[cfg(all(test, feature = "test-support"))] mod tests` 里,
**必须带 `--features test-support`**, 否则测试模块整个不编译, `cargo test` 会安静地跑 0 个用例.
该 feature 转发到 `gpui/test-support`(提供 `TestAppContext` / `#[gpui::test]`),
测试用到的 `futures` 在 `[dev-dependencies]` 里.

**NOTE**: `image_gallery` 目前编译不过 —— 它用了 zed 内部的 `reqwest_client::ReqwestClient`,
而 `Cargo.toml` 引的注册表版 `reqwest_client` **没有** 这个 API. 受影响的命令:
`cargo run --example image_gallery` 和 `cargo test --examples`(整套跑会卡在它身上, 逐个示例跑没问题).

## 架构

### docs ↔ examples

章节与示例的映射表在 `docs/README.md`(每章结尾也给 `cargo run --example xxx`).
`docs/README.md` 末尾还有阅读路线和「文档用语 ↔ GPUI 概念」术语对照.

### examples/ 布局

- 每个 `examples/*.rs` 是独立 example target, 文件顶 `//!` 写用途、运行方式、界面分块.
- 共享字体加载在 `examples/example_support/fonts.rs`, 各示例用
  `#[path = "example_support/fonts.rs"] mod example_support;` 引入.
- 多数示例是桌面 + wasm 双入口: 顶部 `#![cfg_attr(target_family = "wasm", no_main)]`,
  桌面走 `fn main` + `gpui_platform::application()`, wasm 走 `#[wasm_bindgen(start)]` + `gpui_platform::web_init()`,
  两边共用 `run_example()`.
- `examples/image/`、`examples/svg/`、`examples/view_example/` 是多文件参考源码,
  **没有**注册成 cargo example target(`cargo run --example image` 会报找不到目标).
  `image/`、`svg/` 里的图片/GIF 被别的示例当资源引用(如 `opacity.rs` 以 `examples/` 为 AssetSource 资源根).
- `assets/` — 字体(ibm-plex-sans、lilex)、SVG 图标、badge 等.
- `src/main.rs` — 最小 hello-world, 用来看骨架.

## 示例代码注释约定

给示例加注释遵循下面的约定(完整版见 `comtent.md`, 这里是工作时要记住的部分):

1. **注释用中文.**
2. **只加注释, 不改代码.** 加完用文末「自检」确认代码没被动过.
3. **中文解释和 CSS 对照都要有**, 不是二选一.
4. **不写 `CSS:` 这种前缀标签**, 直接写 CSS 本身.
5. 密度以「好理解」为准: 讲清概念和意图, 不逐行复述代码在做什么.

### 样式调用: `// 中文含义: 等价 CSS`

GPUI 样式 API 是 Tailwind CSS 风格:

```rust
div() // 创建一个 div 元素, 类似 HTML 里的 <div>
    .flex() // 弹性布局: display: flex
    .gap_3() // 子元素之间的间距: gap: 0.75rem (12px)
    .bg(rgb(0x505050)) // 背景色: background-color: #505050
    .size(px(500.0)) // 宽高都是 500 像素: width: 500px; height: 500px
    .justify_center() // 主轴居中(这里是垂直居中): justify-content: center
```

- 尺寸写清换算值: `size_8()` 写成 `8 号尺寸的正方形: width: 2rem; height: 2rem (32px)`.
- 颜色写十六进制: `gpui::red()` 写成 `背景色: background-color: #ff0000`.
- 保留语境: `justify_center()` 在纵向 flex 里是垂直居中, 就写「主轴居中(这里是垂直居中)」.
- 一行写不下写关键值再省略, 如 `box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), ...`.

### GPUI 特有概念: 直接中文解释

没有 CSS 对应的东西(全局状态、生命周期、trait 差异等)**不硬套 CSS**, 用中文讲清是什么、为什么:

```rust
/// `RenderOnce` 和 `Render` 的区别:
///   - `Render`      用在「有自己状态、可以反复重绘」的组件上(&mut self)
///   - `RenderOnce`  用在「一次性消费掉自己」的元素上(self), 更轻量
cx.notify(); // 通知 GPUI: 状态变了, 需要重新渲染
```

### 文档注释 vs 行注释

- `///` 用在结构体、字段、函数、trait 上, 讲「这是什么、为什么存在」.
- `//` 用在语句和链式调用后面, 讲「这一行在干嘛」.
- `//!` 用在文件顶部, 讲整个示例的用途、运行方式、界面分块.
- 代码分块用分隔注释划章节:

```rust
// ===========================================================================
// 一、全局排版设置 TextContext
// ===========================================================================
```

### 链式调用的注释位置

注释**直接跟在同一行末尾**. 某段调用成组、逐行注释很啰嗦时, 可在链末尾集中说明:

```rust
div().p_4().gap_2().flex().flex_col().children(grid_rows)
// p_4():      padding: 1rem (16px)
// gap_2():    gap: 0.5rem (8px)
// flex():     display: flex
// flex_col(): flex-direction: column
```

### 自检

加完注释必须验证代码没被改动 —— 把注释和空行剔掉后与改动前逐行比对:

```bash
git show HEAD:<文件> | sed 's|//.*||' | sed 's/[[:space:]]*$//' | grep -v '^[[:space:]]*$' > /tmp/a.rs
sed 's|//.*||' <文件> | sed 's/[[:space:]]*$//' | grep -v '^[[:space:]]*$' > /tmp/b.rs
diff /tmp/a.rs /tmp/b.rs
```

输出为空即通过, 然后跑一次编译确认没写坏: `cargo check --example <名称>`.

## docs 写作约定

样板见 `docs/07-focus.md`、`docs/05-media.md`. 体例:

- 标题 `# N. 标题`, 小节 `## N.M 小节`; 篇幅长的章结尾一节「N.x xx 速查表」, 表头 `| 类别 | 方法 | 说明 |`.
- **中文句号用半角 `. `(点+空格), 逗号用 `, `(逗号+空格)**, 不用全角; 括号半角紧贴文字.
- 容易搞错/别的教程写错的地方用 `**NOTE**: xxx **没有** yyy 这个方法`; 补充背景用 `💡 提示:`.
- 章节互引写 `(见第 8 章)` / `(见 5.4)`; 每章结尾给对应的 `cargo run --example xxx`.
