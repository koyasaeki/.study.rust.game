use std::rc::Rc;
use std::sync::Mutex;

use futures::channel::oneshot::channel;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let context = get_canvas_context();

    spawn_local(async move {
        let (sender, receiver) = channel::<Result<(), JsValue>>();

        // 複数のスレッド間で sender を共有するために Mutex 型にする。
        // 複数のスレッドで sender を使いたいので Rc 型にする。
        let success_sender = Rc::new(Mutex::new(Some(sender)));
        // sender をエラー発生時にも使いたいのでカウンタを増やす。
        let error_sender = Rc::clone(&success_sender);
        let image = HtmlImageElement::new().unwrap();

        let success_callback = Closure::once(move || {
            if let Some(success_sender) = success_sender.lock().ok().and_then(|mut opt| opt.take())
            {
                success_sender.send(Ok(()));
            }
        });
        let error_callback = Closure::once(move |err| {
            if let Some(error_sender) = error_sender.lock().ok().and_then(|mut opt| opt.take()) {
                error_sender.send(Err(err));
            }
        });

        image.set_src("Idle (1).png");

        // 画像の設定に成功したときの処理
        image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
        // 画像の設定に失敗したときの処理
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        receiver.await;
        context.draw_image_with_html_image_element(&image, 300.0, 300.0);
    });

    Ok(())
}

fn get_canvas_context() -> CanvasRenderingContext2d {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap()
}
