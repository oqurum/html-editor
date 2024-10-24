use std::{cell::RefCell, rc::Rc};

use chrono::{Duration, Utc};
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

const HEIGHT: f64 = 30.0;

pub struct Toolbar {
    pub popup: Option<HtmlElement>,
    listeners: Vec<ElementEvent>,

    listener_id: ListenerId,
    data: SharedListenerData,
    func: ListenerEvent,

    buttons: Vec<Button>,
    expanded_index: Option<usize>,
}

impl Toolbar {
    pub fn new(listener_id: ListenerId, data: SharedListenerData, func: &ListenerEvent) -> Self {
        Self {
            data,
            listener_id,
            popup: None,
            buttons: Vec::new(),
            listeners: Vec::new(),
            func: func.clone(),
            expanded_index: None,
        }
    }

    pub fn reposition(&mut self, selection: Selection) -> Result<()> {
        let range = selection.get_range_at(0)?;
        let bb = range.get_bounding_client_rect();
        let (x, y, selected_width) = (bb.x(), bb.y(), bb.width());

        if let Some(popup) = self.popup.as_ref() {
            let style = popup.style();

            style.set_property("left", &format!("{}px", x + selected_width / 2.0,))?;

            style.set_property("top", &format!("{}px", y - HEIGHT - 3.0,))?;
        }

        Ok(())
    }

    pub fn open(&mut self, selection: Selection) -> Result<()> {
        self.close();

        self.create_popup()?;

        self.reposition(selection)?;

        if let Some(popup) = self.popup.as_ref() {
            self.listener_id
                .document()
                .body()
                .unwrap_throw()
                .append_child(popup)?;
        }

        Ok(())
    }

    fn reload(&mut self, selection: Selection) -> Result<()> {
        if let Some(popup) = self.popup.take() {
            popup.remove();
        }

        self.open(selection)
    }

    pub fn close(&mut self) {
        if let Some(popup) = self.popup.take() {
            popup.remove();
            self.expanded_index = None;
        }
    }

    fn create_popup(&mut self) -> Result<()> {
        let popup_element: HtmlElement = self
            .listener_id
            .document()
            .create_element("div")?
            .unchecked_into();
        popup_element.set_class_name("editor-toolbar");

        self.buttons.clear();
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
                popup_element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mousedown", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mousedown", f)),
            ));
        }

        // Create the mouse up listener
        {
            let listener_id = self.listener_id;
            let func = self.func.clone();
            let data = self.data.clone();
            let document = self.listener_id.document();

            let function = Closure::wrap(Box::new(move |e: MouseEvent| {
                let click_element: HtmlElement = e.target_unchecked_into();
                let is_held = Utc::now().signed_duration_since(*last_clicked.borrow())
                    >= Duration::milliseconds(500);

                // Get the selection
                if let Some(selection) = document
                    .get_selection()
                    .unwrap_throw()
                    .filter(|v| !v.is_collapsed())
                {
                    {
                        let listener = listener_id.try_get().unwrap();
                        let mut borrow = listener.borrow_mut();

                        for (idx, button) in borrow.toolbar.buttons.iter().enumerate() {
                            if parents_contains_element(
                                click_element.unchecked_ref(),
                                &button.element,
                            ) {
                                match button.type_of {
                                    ComponentFlag::HIGHLIGHT => {
                                        if is_held {
                                            borrow.toolbar.expanded_index = Some(idx);
                                            drop(borrow);
                                        } else {
                                            drop(borrow);
                                            let context = Context::new(
                                                Rc::new(RefCell::new(
                                                    selection::get_nodes_in_selection(
                                                        selection.clone(),
                                                        data.clone(),
                                                    )
                                                    .unwrap_throw(),
                                                )),
                                                document.clone(),
                                            );

                                            Highlight.on_click_button(&context).unwrap_throw();
                                        }
                                    }

                                    ComponentFlag::NOTE => {
                                        if is_held {
                                            borrow.toolbar.expanded_index = Some(idx);
                                            drop(borrow);
                                        } else {
                                            drop(borrow);
                                            let context = Context::new(
                                                Rc::new(RefCell::new(
                                                    selection::get_nodes_in_selection(
                                                        selection.clone(),
                                                        data.clone(),
                                                    )
                                                    .unwrap_throw(),
                                                )),
                                                document.clone(),
                                            );

                                            Note.on_click_button(&context).unwrap_throw();
                                        }
                                    }

                                    ComponentFlag::LIST => {
                                        drop(borrow);

                                        let context = Context::new(
                                            Rc::new(RefCell::new(
                                                selection::get_nodes_in_selection(
                                                    selection.clone(),
                                                    data.clone(),
                                                )
                                                .unwrap_throw(),
                                            )),
                                            document.clone(),
                                        );

                                        List.on_click_button(&context).unwrap_throw();
                                    }

                                    _ => drop(borrow),
                                }

                                // Reload the toolbar
                                {
                                    let listener = listener_id.try_get().unwrap();
                                    let mut borrow = listener.borrow_mut();

                                    if let Err(e) = borrow.toolbar.reload(selection) {
                                        error!("Failed to open toolbar: {e:?}");
                                    }
                                }

                                break;
                            }
                        }
                    }

                    (func.borrow_mut())(listener_id);
                }
            }) as Box<dyn Fn(MouseEvent)>);

            self.listeners.push(ElementEvent::link(
                popup_element.clone().unchecked_into(),
                function,
                |t, f| t.add_event_listener_with_callback("mouseup", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("mouseup", f)),
            ));
        }

        self.popup = Some(popup_element);

        let selected = if let Some(selection) = self
            .listener_id
            .document()
            .get_selection()
            .unwrap_throw()
            .filter(|v| !v.is_collapsed())
        {
            let node_sel =
                selection::get_nodes_in_selection(selection, self.data.clone()).unwrap_throw();
            node_sel
                .get_selected_data_ids()
                .into_iter()
                .map(|(v, _)| v)
                .collect()
        } else {
            Vec::new()
        };

        self.create_button::<Highlight>(&selected)?;
        self.create_button::<Note>(&selected)?;
        self.create_button::<List>(&selected)?;

        Ok(())
    }

    fn create_button<C: Component + 'static>(&mut self, selected: &[ComponentFlag]) -> Result<()> {
        if let Some(idx) = self.expanded_index {
            if idx == self.buttons.len() {
                // TODO: Create the expanded button

                // return Ok(());
            }
        }

        let element: HtmlElement = self
            .listener_id
            .document()
            .create_element("div")?
            .unchecked_into();

        element.set_inner_text(C::TITLE);
        element.set_class_name("editor-button");

        self.popup.as_ref().unwrap().append_child(&element)?;

        let button = Button {
            element,
            type_of: C::FLAG,
        };

        button.set_selected(selected.iter().any(|&f| f == button.type_of))?;

        self.buttons.push(button);

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
