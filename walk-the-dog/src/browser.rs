use anyhow::anyhow;
use anyhow::Result;
use futures::Future;
use wasm_bindgen::closure::WasmClosure;
use wasm_bindgen::closure::WasmClosureFnOnce;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::CanvasRenderingContext2d;
use web_sys::Document;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlImageElement;
use web_sys::Response;
use web_sys::Window;

/// log マクロ
///
/// format! 関数などと同じ構文でコンソールに対してログを出力できる。  
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t)* ).into());
    };
}

/// ブラウザの window オブジェクト
pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("No window Found"))
}

/// ブラウザの document オブジェクト
pub fn document() -> Result<Document> {
    window()?
        .document()
        .ok_or_else(|| anyhow!("No document Found"))
}

/// canvas 要素
pub fn canvas() -> Result<HtmlCanvasElement> {
    //  ID の canvas がハードコードされているが、一旦このまま進める。
    document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| anyhow!("No Canvas Element found with ID 'canvas'"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|element| anyhow!("Error converting {:#?} to HtmlCanvasElement", element))
}

/// canvas の context
pub fn context() -> Result<CanvasRenderingContext2d> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("Error getting 2d context {:#?}", js_value))?
        .ok_or_else(|| anyhow!("No 2d context found"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|element| {
            anyhow!(
                "Error converting {:#?} to CanvasRenderingContext2d",
                element
            )
        })
}

pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

/// JS 側で resource にデータを取得しにいく
pub async fn fetch_with_str(resource: &str) -> Result<JsValue> {
    JsFuture::from(window()?.fetch_with_str(resource))
        .await
        .map_err(|err| anyhow!("error fetching {:#?}", err))
}

/// JS 側で JSON を取得する
pub async fn fetch_json(json_path: &str) -> Result<JsValue> {
    let response_value = fetch_with_str(json_path).await?;
    let response: Response = response_value
        .dyn_into()
        .map_err(|element| anyhow!("Error converting {:#?} to Response", element))?;

    JsFuture::from(
        response
            .json()
            .map_err(|err| anyhow!("Could not get JSON from response {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("error fetching JSON {:#?}", err))
}

/// HTML の image 要素を作成する
pub fn new_image() -> Result<HtmlImageElement> {
    HtmlImageElement::new().map_err(|err| anyhow!("Could not create HtmlImageElement: {:#?}", err))
}

/// Rust のクロージャを JS のクロージャにする
pub fn closure_once<F, A, R>(fn_once: F) -> Closure<F::FnMut>
where
    F: 'static + WasmClosureFnOnce<A, R>,
{
    Closure::once(fn_once)
}

pub type LoopClosure = Closure<dyn FnMut(f64)>;
/// requestAnimationFrame のラッパー関数
pub fn request_animation_frame(callback: &LoopClosure) -> Result<i32> {
    window()?
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .map_err(|err| anyhow!("Cannot request animation frame {:#?}", err))
}

/// closure を作る関数
pub fn create_raf_closure(f: impl FnMut(f64) + 'static) -> LoopClosure {
    closure_wrap(Box::new(f))
}

/// closure を作る関数のためのラッパー
fn closure_wrap<T: WasmClosure + ?Sized>(data: Box<T>) -> Closure<T> {
    Closure::wrap(data)
}
