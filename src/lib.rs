use wasm_bindgen::JsValue;

pub mod component;
mod gui;
mod helper;
mod listener;
mod node;
mod selection;
pub mod toolbar;

pub type Result<V, E = JsValue> = std::result::Result<V, E>;

pub use component::{Component, ComponentFlag};
pub use listener::{register, SharedListenerData};
pub use node::ComponentNode;
