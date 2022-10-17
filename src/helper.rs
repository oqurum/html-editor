// Coped from YEW

use wasm_bindgen::JsCast;
use web_sys::{Element, Event, EventTarget};

pub fn parents_contains_class(element: Element, class: &str) -> bool {
    if element.class_list().contains(class) {
        true
    } else if let Some(parent) = element.parent_element() {
        parents_contains_class(parent, class)
    } else {
        false
    }
}

pub trait TargetCast
where
    Self: AsRef<Event>,
{
    #[inline]
    fn target_dyn_into<T>(&self) -> Option<T>
    where
        T: AsRef<EventTarget> + JsCast,
    {
        self.as_ref()
            .target()
            .and_then(|target| target.dyn_into().ok())
    }

    #[inline]
    fn target_unchecked_into<T>(&self) -> T
    where
        T: AsRef<EventTarget> + JsCast,
    {
        self.as_ref().target().unwrap().unchecked_into()
    }
}

impl<E: AsRef<Event>> TargetCast for E {}
