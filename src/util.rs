use js_sys::Function;
use wasm_bindgen::{JsValue, JsCast, UnwrapThrowExt};
use web_sys::EventTarget;



type Destructor = Box<dyn FnOnce(&EventTarget, &Function) -> std::result::Result<(), JsValue>>;

/// Allows for easier creation and destruction of event listener functions.
///
/// Copied from Main Oqurum Module
pub struct ElementEvent {
    element: EventTarget,
    function: Box<dyn AsRef<JsValue>>,

    destructor: Option<Destructor>,
}

impl ElementEvent {
    pub fn link<
        C: AsRef<JsValue> + 'static,
        F: FnOnce(&EventTarget, &Function) -> std::result::Result<(), JsValue>,
    >(
        element: EventTarget,
        function: C,
        creator: F,
        destructor: Destructor,
    ) -> Self {
        let this = Self {
            element,
            function: Box::new(function),
            destructor: Some(destructor),
        };

        creator(&this.element, (*this.function).as_ref().unchecked_ref()).unwrap_throw();

        this
    }
}

impl Drop for ElementEvent {
    fn drop(&mut self) {
        if let Some(dest) = self.destructor.take() {
            dest(&self.element, (*self.function).as_ref().unchecked_ref()).unwrap_throw();
        }
    }
}