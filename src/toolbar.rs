use std::{cell::RefCell, rc::Rc};

use chrono::{Duration, Utc};
use gloo_utils::document;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent, Selection};

use crate::{
    component::{Component, Context, Highlight, List, Note},
    helper::{parents_contains_element, TargetCast},
    listener::{ListenerEvent, SharedListenerData},
    selection,
    util::ElementEvent,
    ComponentFlag, ListenerId, Result,
};

pub struct Toolbar {
    pub popup: HtmlElement,
    listeners: Vec<ElementEvent>,

    listener_id: ListenerId,
    data: SharedListenerData,

    buttons: Vec<Button>,
}

impl Toolbar {
    pub fn new(
        listener_id: ListenerId,
        data: SharedListenerData,
        func: &ListenerEvent,
    ) -> Result<Self> {
        let mut this = Self {
            popup: document().create_element("div")?.unchecked_into(),
            buttons: Vec::new(),
            listeners: Vec::new(),
            data: data.clone(),
            listener_id,
        };

        this.create_popup(data, func)?;

        Ok(this)
    }

    pub fn open(&mut self, selection: Selection) -> Result<()> {
        let range = selection.get_range_at(0)?;
        let bb = range.get_bounding_client_rect();
        let (x, y, selected_width) = (bb.x(), bb.y(), bb.width());

        let node_sel =
            selection::get_nodes_in_selection(selection, self.data.clone()).unwrap_throw();
        let selected = node_sel.get_selected_data_ids();

        const HEIGHT: f64 = 30.0;

        self.close();

        for button in &self.buttons {
            button.set_selected(selected.iter().any(|&(f, _)| f == button.type_of))?;
        }

        let style = self.popup.style();

        style.set_property("left", &format!("{}px", x + selected_width / 2.0,))?;

        style.set_property("top", &format!("{}px", y - HEIGHT - 3.0,))?;

        document().body().unwrap_throw().append_child(&self.popup)?;

        Ok(())
    }

    pub fn close(&mut self) {
        self.popup.remove();
    }

    fn create_popup(&mut self, data: SharedListenerData, func: &ListenerEvent) -> Result<()> {
        let element = &self.popup;
        element.set_class_name("toolbar");

        self.listeners.clear();

        let last_clicked = Rc::new(RefCell::new(Utc::now()));

        // Create the mouse down listener
        {
            let last_clicked = last_clicked.clone();

            let function = Closure::wrap(Box::new(move |e: MouseEvent| {
                e.prevent_default();
                *last_clicked.borrow_mut() = Utc::now();
            }) as Box<dyn Fn(MouseEvent)>);

            self.listeners.push(ElementEvent::link(
                element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mousedown", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mousedown", f)),
            ));
        }

        // Create the mouse up listener
        {
            let listener_id = self.listener_id;
            let func = func.clone();

            let function = Closure::wrap(Box::new(move |e: MouseEvent| {
                let click_element: HtmlElement = e.target_unchecked_into();

                if Utc::now().signed_duration_since(*last_clicked.borrow())
                    >= Duration::milliseconds(500)
                {
                    // TODO
                } else if let Some(selection) = document()
                    .get_selection()
                    .unwrap_throw()
                    .filter(|v| !v.is_collapsed())
                {
                    {
                        let listener = listener_id.try_get().unwrap();
                        let borrow = listener.borrow();

                        for button in &borrow.toolbar.buttons {
                            if parents_contains_element(
                                click_element.unchecked_ref(),
                                &button.element,
                            ) {
                                match button.type_of {
                                    ComponentFlag::HIGHLIGHT => {
                                        drop(borrow);

                                        let context = Context::new(Rc::new(RefCell::new(
                                            selection::get_nodes_in_selection(
                                                selection,
                                                data.clone(),
                                            )
                                            .unwrap_throw(),
                                        )));
                                        Highlight.on_click_button(&context).unwrap_throw();
                                    }

                                    ComponentFlag::NOTE => {
                                        drop(borrow);

                                        let context = Context::new(Rc::new(RefCell::new(
                                            selection::get_nodes_in_selection(
                                                selection,
                                                data.clone(),
                                            )
                                            .unwrap_throw(),
                                        )));
                                        Note.on_click_button(&context).unwrap_throw();
                                    }

                                    _ => (),
                                }

                                break;
                            }
                        }
                    }

                    (func.borrow_mut())(listener_id);
                }
            }) as Box<dyn Fn(MouseEvent)>);

            self.listeners.push(ElementEvent::link(
                element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mouseup", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mouseup", f)),
            ));
        }

        self.create_button::<Highlight>()?;
        self.create_button::<Note>()?;
        self.create_button::<List>()?;
        // self.create_button(Underline, data.clone(), func.clone())?;
        // self.create_button(Italicize, data.clone(), func.clone())?;

        Ok(())
    }

    fn create_button<C: Component + 'static>(&mut self) -> Result<()> {
        let element: HtmlElement = document().create_element("div")?.unchecked_into();

        element.set_inner_text(C::TITLE);
        element.set_class_name("button");

        self.popup.append_child(&element)?;

        self.buttons.push(Button {
            element,
            type_of: C::FLAG,
        });

        Ok(())
    }
}

pub struct Button {
    type_of: ComponentFlag,
    element: HtmlElement,
}

impl Button {
    pub fn set_selected(&self, value: bool) -> Result<()> {
        let class_list = self.element.class_list();

        if value {
            if !class_list.contains("selected") {
                class_list.add_1("selected")?;
            }
        } else {
            class_list.remove_1("selected")?;
        }

        Ok(())
    }
}

impl Drop for Button {
    fn drop(&mut self) {
        self.element.remove();
    }
}
