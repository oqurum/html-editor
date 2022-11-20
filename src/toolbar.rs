use std::{cell::RefCell, rc::Rc};

use chrono::{Duration, Utc};
use gloo_utils::document;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent};

use crate::{
    component::{Component, Context, Highlight, Note},
    listener::SharedListenerData,
    selection,
    util::ElementEvent,
    ListenerId, Result,
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
        data: SharedListenerData,
        func: &Rc<RefCell<dyn Fn(ListenerId)>>,
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
        const HEIGHT: f64 = 30.0;

        self.close();

        let style = self.popup.style();

        style.set_property("left", &format!("{}px", x + selected_width / 2.0,))?;

        style.set_property("top", &format!("{}px", y - HEIGHT - 3.0,))?;

        document().body().unwrap_throw().append_child(&self.popup)?;

        Ok(())
    }

    pub fn close(&mut self) {
        self.popup.remove();
    }

    fn create_popup(
        &mut self,
        data: SharedListenerData,
        func: &Rc<RefCell<dyn Fn(ListenerId)>>,
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

        self.mouse_down_listener = Some(ignore_mouse_down);

        self.create_button(Highlight, data.clone(), func.clone())?;
        self.create_button(Note, data, func.clone())?;
        // self.create_button(Underline, data.clone(), func.clone())?;
        // self.create_button(Italicize, data.clone(), func.clone())?;

        Ok(())
    }

    fn create_button<C: Component + 'static>(
        &mut self,
        component: C,
        data: SharedListenerData,
        func: Rc<RefCell<dyn Fn(ListenerId)>>,
    ) -> Result<()> {
        let listener_id = self.listener_id;

        let element: HtmlElement = document().create_element("div")?.unchecked_into();

        element.set_inner_text(C::TITLE);
        element.set_class_name("button");

        self.popup.append_child(&element)?;

        let last_clicked = Rc::new(RefCell::new(Utc::now()));

        let mut events = Vec::new();

        // Create the mouse down listener
        {
            let last_clicked = last_clicked.clone();

            let function = Closure::wrap(Box::new(move |_event: MouseEvent| {
                *last_clicked.borrow_mut() = Utc::now();
            }) as Box<dyn Fn(MouseEvent)>);

            events.push(ElementEvent::link(
                element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mousedown", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mousedown", f)),
            ));
        }

        // Create the mouse up listener
        {
            let function = Closure::wrap(Box::new(move || {
                if Utc::now().signed_duration_since(*last_clicked.borrow())
                    >= Duration::milliseconds(500)
                {
                    // TODO
                } else if let Some(selection) = document()
                    .get_selection()
                    .unwrap_throw()
                    .filter(|v| !v.is_collapsed())
                {
                    let context = Context {
                        nodes: Rc::new(RefCell::new(
                            selection::get_nodes_in_selection(selection, data.clone())
                                .unwrap_throw(),
                        )),
                    };

                    component.on_click_button(&context).unwrap_throw();

                    (func.borrow_mut())(listener_id);
                }
            }) as Box<dyn Fn()>);

            events.push(ElementEvent::link(
                element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mouseup", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mouseup", f)),
            ));
        }

        self.buttons.push(Button { element, events });

        Ok(())
    }
}

#[allow(dead_code)]
pub struct Button {
    element: HtmlElement,
    events: Vec<ElementEvent>,
}
