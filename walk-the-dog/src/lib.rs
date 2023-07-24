use futures::channel::oneshot::channel;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, Response};

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
        context.draw_image_with_html_image_element(&image, 0.0, 0.0);

        let json = fetch_json("rhb.json")
            .await
            .expect("Could not fetch rhb.json");
        // deprecated だけど一旦このまま進む。
        let sheet: Sheet = json
            .into_serde()
            .expect("Could not convert rhg.json into a Sheet structure");

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

        image.set_src("rhb.png");

        // 画像の設定に成功したときの処理
        image.set_onload(Some(success_callback.as_ref().unchecked_ref()));
        // 画像の設定に失敗したときの処理
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        receiver.await;

        let sprite = sheet.frames.get("Run (1).png").expect("Cell not found");
        context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
            &image,
            sprite.frame.x.into(),
            sprite.frame.y.into(),
            sprite.frame.w.into(),
            sprite.frame.h.into(),
            300.0,
            300.0,
            sprite.frame.w.into(),
            sprite.frame.h.into(),
        );
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

async fn fetch_json(json_path: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let response_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(json_path)).await?;
    let response: Response = response_value.dyn_into()?;

    wasm_bindgen_futures::JsFuture::from(response.json()?).await
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}
