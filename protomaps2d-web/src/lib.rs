use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, console};
use piet_web::WebRenderContext;
use protomaps2d::{render_tile,Style,Result};

use std::panic;
extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn wasm_render_tile(tile_id: &str,buf: &[u8], zoom:u32,total:u32,dx:u32,dy:u32, style_js:&JsValue) -> JsValue {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let style: Style = style_js.into_serde().unwrap();

    let empty = JsValue::from_serde(&Result{feature_count:0}).unwrap();

    let window_opt = window();
    if window_opt.is_none() {
        return empty;
    }
    let window = window_opt.unwrap();
    let document = window.document();
    if document.is_none() {
        return empty;
    }
    let handle = document.unwrap().get_element_by_id(tile_id);
    if handle.is_none() {
        return empty;
    }
    let canvas = handle.unwrap().dyn_into::<HtmlCanvasElement>();
    if canvas.is_err() {
        return empty;
    }
    let c1 = canvas.unwrap().get_context("2d");
    if c1.is_err() {
        return empty;
    }
    let c2 = c1.unwrap();
    if c2.is_none() {
        return empty;
    }
    let c3 = c2.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>();
    if c3.is_err() {
        return empty;
    }

    let c4 = c3.unwrap();
    let mut rc = WebRenderContext::new(c4, window);

    fn logger(s: &String) {
        console::log_1(&s.into());
    }

    let result = render_tile(&mut rc,&buf,zoom,total,dx,dy,&style,&logger);
    return JsValue::from_serde(&result).unwrap();
}