#![allow(unused_imports)]

#[macro_use]
extern crate log;

use wasm_bindgen::prelude::*;

mod app;
mod pages;
mod routes;

pub fn main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<app::App>();

    Ok(())
}
