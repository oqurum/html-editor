use wasm_bindgen::JsValue;

pub mod component;
mod gui;
mod helper;
mod listener;
mod node;
mod selection;
mod store;
pub mod toolbar;

pub type Result<V, E = JsValue> = std::result::Result<V, E>;

pub use component::{Component, ComponentFlag};
pub use listener::{register, register_with_data, ListenerId};
pub use node::{ComponentNode, TextContainer};
pub use store::{load_and_register, save, SaveState, SavedNode, SavedNodeFlag};

pub(crate) use listener::SharedListenerData;
