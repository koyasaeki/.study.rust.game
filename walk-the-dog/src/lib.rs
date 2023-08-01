#[macro_use]
mod browser;
mod engine;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use browser::LoopClosure;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let context = browser::context().expect("No Context Found");

    browser::spawn_local(async move {
        let sheet: Sheet = serde_wasm_bindgen::from_value(
            browser::fetch_json("rhb.json")
                .await
                .expect("Could not fetch rhb.json"),
        )
        .expect("Could not convert rhb.json into a Sheet structure");

        let image = engine::load_iamge("rhb.png")
            .await
            .expect("Could not load rhb.png");

        let f: Rc<RefCell<Option<LoopClosure>>> = Rc::new(RefCell::new(None));

        let mut frame = -1;
        // wrap とか Box とか FnMut とかわからんけど一旦進めておく。
        let interval_callback = Closure::wrap(Box::new(move || {
            frame = (frame + 1) % 8;
            let frame_name = format!("Run ({}).png", frame + 1);

            context.clear_rect(0.0, 0.0, 600.0, 600.0);

            let sprite = sheet.frames.get(&frame_name).expect("Cell not found");
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
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            );
        interval_callback.forget();
    });

    Ok(())
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
