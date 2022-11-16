use std::{cell::{RefCell, Cell}, rc::Rc};

use gloo_utils::document;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent};

use crate::{
    component::{Component, Highlight, Italicize, Underline, Context},
    listener::SharedListenerData,
    selection, ListenerId, Result,
};

pub struct Toolbar {
    pub popup: HtmlElement,
    mouse_down_listener: Option<Closure<dyn Fn(MouseEvent)>>,

    listener_id: ListenerId,

    buttons: Vec<Button>,
}

impl Toolbar {
    pub fn new(
        listener_id: ListenerId,
        data: &SharedListenerData,
        func: &Rc<RefCell<fn(ListenerId)>>,
    ) -> Result<Self> {
        let mut this = Self {
            popup: document().create_element("div")?.unchecked_into(),
            buttons: Vec::new(),
            mouse_down_listener: None,
            listener_id,
        };

        this.create_popup(data, func)?;

        Ok(this)
    }

    pub fn open(&mut self, x: f64, y: f64, selected_width: f64) -> Result<()> {
        self.close();

        let style = self.popup.style();

        let width = 90.0; // TODO: Remove static width
        let height = 30.0;

        style.set_property(
            "left",
            &format!(
                "calc({x}px + {}px - {}px)",
                selected_width / 2.0,
                width / 2.0
            ),
        )?;
        style.set_property("top", &format!("calc({y}px - {height}px - 3px)"))?;

        document().body().unwrap_throw().append_child(&self.popup)?;

        Ok(())
    }

    pub fn close(&mut self) {
        self.popup.remove();
    }

    fn create_popup(
        &mut self,
        data: &SharedListenerData,
        func: &Rc<RefCell<fn(ListenerId)>>,
    ) -> Result<()> {
        let element = &self.popup;
        element.set_class_name("toolbar");

        // Ignore Mouse Down Event
        let ignore_mouse_down = Closure::wrap(Box::new(move |e: MouseEvent| {
            e.prevent_default();
        }) as Box<dyn Fn(MouseEvent)>);

        element.add_event_listener_with_callback(
            "mousedown",
            ignore_mouse_down.as_ref().unchecked_ref(),
        )?;

        let style = element.style();

        self.mouse_down_listener = Some(ignore_mouse_down);

        self.create_button(Highlight, data.clone(), func.clone())?;
        self.create_button(Underline, data.clone(), func.clone())?;
        self.create_button(Italicize, data.clone(), func.clone())?;

        // Style Toolbar
        style.set_property("width", &format!("{}px", self.buttons.len() * 30))?;

        Ok(())
    }

    fn create_button<C: Component + 'static>(
        &mut self,
        component: C,
        data: SharedListenerData,
        func: Rc<RefCell<fn(ListenerId)>>,
    ) -> Result<()> {
        let listener_id = self.listener_id;

        let element: HtmlElement = document().create_element("div")?.unchecked_into();

        element.set_inner_text(C::TITLE);
        element.set_class_name("button");

        let function = Closure::wrap(Box::new(move || {
            if let Some(selection) = document()
                .get_selection()
                .unwrap_throw()
                .filter(|v| !v.is_collapsed())
            {
                let context = Context {
                    nodes: Rc::new(RefCell::new(selection::get_nodes_in_selection(selection, data.clone()).unwrap_throw()))
                };

                component.on_select(&context).unwrap_throw();

                (func.borrow_mut())(listener_id);
            }
        }) as Box<dyn Fn()>);

        element.add_event_listener_with_callback("click", function.as_ref().unchecked_ref())?;
        // TODO: On Hold Event

        // Add Button to Toolbar
        self.popup.append_child(&element)?;

        self.buttons.push(Button { element, function });

        Ok(())
    }
}

#[allow(dead_code)]
pub struct Button {
    element: HtmlElement,
    function: Closure<dyn Fn()>,
}
