use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement};
use piet_web::WebRenderContext;
use protomaps_alpha::render_tile;

use std::panic;
extern crate console_error_panic_hook;
use console_log;


#[wasm_bindgen]
pub fn wasm_render_tile(tile_id: &str,buf: Vec<u8>, zoom:u32) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    console_log::init();

    let window_opt = window();
    if window_opt.is_none() {
        return;
    }
    let window = window_opt.unwrap();
    let document = window.document();
    if document.is_none() {
        return;
    }
    let handle = document.unwrap().get_element_by_id(tile_id);
    if handle.is_none() {
        return;
    }
    let canvas = handle.unwrap().dyn_into::<HtmlCanvasElement>();
    if canvas.is_err() {
        return;
    }
    let c1 = canvas.unwrap().get_context("2d");
    if c1.is_err() {
        return;
    }
    let c2 = c1.unwrap();
    if c2.is_none() {
        return;
    }
    let c3 = c2.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>();
    if c3.is_err() {
        return;
    }

    let mut c4 = c3.unwrap();
    let mut rc = WebRenderContext::new(&mut c4, &window);

    render_tile(&mut rc,&buf,zoom);
}