use dioxus::prelude::*;

#[allow(dead_code)]
pub enum ScreenSize {
    Desktop,
    Tablet,
    Mobile,
}

pub fn use_screen_size() -> Signal<ScreenSize> {
    let screen_size = use_signal(|| ScreenSize::Desktop);

    // TODO This returns 0.0 but should return the actual width
    // use_future(move || async move {
    //     let js_code = r#"
    //         (function() {
    //             return window.innerWidth;
    //         })()
    //     "#;
    //     let width = eval(js_code).await;
    //     if let Ok(width) = width {
    //         let width = width.as_f64().unwrap_or(0.0);
    //         let new_screen_size = if width < 768.0 {
    //             ScreenSize::Mobile
    //         } else if width < 1024.0 {
    //             ScreenSize::Tablet
    //         } else {
    //             ScreenSize::Desktop
    //         };
    //         screen_size.set(new_screen_size);
    //     }
    // });

    screen_size
}
