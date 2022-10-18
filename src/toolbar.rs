use gloo_utils::document;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent};

use crate::{
    component::{Component, Highlight},
    listener::SharedListenerData,
    selection, Result,
};

pub struct Toolbar {
    pub popup: HtmlElement,
    mouse_down_listener: Option<Closure<dyn Fn(MouseEvent)>>,

    buttons: Vec<Button>,
}

impl Toolbar {
    pub fn new(data: &SharedListenerData) -> Result<Self> {
        let mut this = Self {
            popup: document().create_element("div")?.unchecked_into(),
            buttons: Vec::new(),
            mouse_down_listener: None,
        };

        this.create_popup(data)?;

        Ok(this)
    }

    pub fn open(&mut self, x: f64, y: f64, selected_width: f64) -> Result<()> {
        self.close();

        let style = self.popup.style();

        let width = 100.0;
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

    fn create_popup(&mut self, data: &SharedListenerData) -> Result<()> {
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

        // Style Toolbar
        let style = element.style();

        let width = 100.0;

        style.set_property("width", &format!("{width}px"))?;

        self.mouse_down_listener = Some(ignore_mouse_down);

        self.create_button(Highlight, data.clone())?;

        Ok(())
    }

    fn create_button<C: Component + 'static>(
        &mut self,
        component: C,
        data: SharedListenerData,
    ) -> Result<()> {
        let element: HtmlElement = document().create_element("div")?.unchecked_into();

        element.set_inner_text(C::TITLE);
        element.set_class_name("button");

        let function = Closure::wrap(Box::new(move || {
            if let Some(selection) = document()
                .get_selection()
                .unwrap_throw()
                .filter(|v| !v.is_collapsed())
            {
                let nodes = selection::get_nodes_in_selection(selection, &data).unwrap_throw();

                component.on_select(nodes).unwrap_throw();
            }
        }) as Box<dyn Fn()>);

        element.add_event_listener_with_callback("click", function.as_ref().unchecked_ref())?;

        // Add Button to Toolbar
        self.popup.append_child(&element)?;

        self.buttons.push(Button { element, function });

        Ok(())
    }
}

pub struct Button {
    element: HtmlElement,
    function: Closure<dyn Fn()>,
}
