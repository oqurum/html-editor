#[macro_use]
extern crate log;

use wasm_bindgen::JsValue;

pub mod component;
mod document;
mod gui;
mod helper;
mod listener;
mod migration;
mod selection;
mod store;
mod text;
mod toolbar;
mod util;

pub type Result<V, E = JsValue> = std::result::Result<V, E>;

pub use component::{Component, ComponentFlag};
pub use listener::{
    register, register_with_data, ListenerEvent, ListenerHandle, ListenerId, MouseListener,
};
pub use store::{load_and_register, save, SaveState, SavedNode, SavedNodeFlag};
pub use text::{TextContainer, WrappedText};
pub use util::{LinePoint, RangeBox};

pub(crate) use listener::SharedListenerData;
