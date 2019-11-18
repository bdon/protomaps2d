use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, window, HtmlCanvasElement};
use piet_web::WebRenderContext;
use protomaps_alpha::render_tile;

use std::panic;
extern crate console_error_panic_hook;


#[wasm_bindgen]
pub fn wasm_render_tile(tile_id: &str,buf: Vec<u8>) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let window = window().unwrap();
    let handle = window
        .document()
        .unwrap()
        .get_element_by_id(tile_id);

    if handle == None {
        return;
    }

    let canvas = handle.unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    let mut context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mut rc = WebRenderContext::new(&mut context, &window);

    render_tile(&mut rc,&buf);
}