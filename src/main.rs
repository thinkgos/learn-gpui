use gpui::{
    App, Context, Render, SharedString, TitlebarOptions, Window, WindowOptions, prelude::*,
};
use gpui::{div, rgb};

struct HelloWorld {
    text: SharedString,
}

impl HelloWorld {
    fn new(text: &str) -> Self {
        Self {
            text: SharedString::from(text),
        }
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full() // 设置宽高为100%
            .bg(rgb(0x505050)) // 背景色为深灰色
            .flex() // 设置 display: flex
            .flex_col() // 设置 flex-direction: column
            .justify_center() // justify-content: center(主轴居中)
            .items_center() // align-items: center(交叉轴居中)
            .text_xl() // 字体大小为 xl
            .child(format!("Hello, {}!", self.text)) // 添加一个子元素
    }
}

fn main() {
    gpui_platform::application().run(|app: &mut App| {
        app.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions::default()),
                ..Default::default()
            },
            |_windows, app| app.new(|_cx| HelloWorld::new("hello")),
        )
        .unwrap();
    });
    println!("Hello, world!");
}
