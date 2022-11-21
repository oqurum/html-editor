use wasm_bindgen::JsValue;

pub mod component;
mod gui;
mod helper;
mod listener;
mod migration;
mod selection;
mod store;
pub(crate) mod text;
pub mod toolbar;
mod util;

pub type Result<V, E = JsValue> = std::result::Result<V, E>;

pub use component::{Component, ComponentFlag};
pub use listener::{register, register_with_data, ListenerId};
pub use store::{load_and_register, save, SaveState, SavedNode, SavedNodeFlag};
pub use text::{TextContainer, WrappedText};

pub(crate) use listener::SharedListenerData;
