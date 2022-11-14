use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use gloo_utils::document;
use lazy_static::lazy_static;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent, Text};

use crate::{
    helper::{parents_contains_class, TargetCast},
    node::TextContainer,
    store,
    toolbar::Toolbar,
    ComponentFlag, Result,
};

pub type SharedListenerType = Rc<RefCell<Listener>>;
pub type SharedListenerData = Rc<RefCell<ListenerData>>;
// TODO: Fix. I don't like all these Rc's

lazy_static! {
    static ref INCREMENT: AtomicUsize = AtomicUsize::default();
}

thread_local! {
    static LISTENERS: RefCell<Vec<SharedListenerType>> = RefCell::new(Vec::new());
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ListenerId(usize);

impl ListenerId {
    pub fn try_get(&self) -> Option<SharedListenerType> {
        LISTENERS.with(|listeners| {
            let listeners = listeners.borrow();

            for item in &*listeners {
                if &item.borrow().listener_id == self {
                    return Some(item.clone());
                }
            }

            None
        })
    }

    /// Returns None if Listener was not found OR if SaveState is empty.
    pub fn try_save(&self) -> Option<store::SaveState> {
        let listener = self.try_get()?;

        let borrow = listener.borrow();

        let borrow2 = borrow.data.borrow();

        let save = store::save(&*borrow2);

        Some(save).filter(|v| !v.nodes.is_empty())
    }
}

impl std::ops::Deref for ListenerId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct ListenerData {
    /// The Text Nodes inside the listener Element. Along with flags for the Text.
    pub(crate) nodes: Vec<TextContainer>,
}

impl ListenerData {
    pub fn new(nodes: Vec<Text>) -> Result<Self> {
        Ok(Self {
            nodes: nodes
                .into_iter()
                .map(TextContainer::new)
                .collect::<Result<_>>()?,
        })
    }

    // TODO: Optimize. We're iterating through two arrays.
    pub fn get_text_container(&self, node: &Text) -> Option<&TextContainer> {
        self.nodes.iter().find(|v| v.contains_node(node))
    }

    pub fn get_text_container_mut(&mut self, node: &Text) -> Option<&mut TextContainer> {
        self.nodes.iter_mut().find(|v| v.contains_node(node))
    }

    pub fn update_container(&mut self, text: &Text, flag: ComponentFlag) -> Result<()> {
        if let Some(comp) = self.get_text_container_mut(text) {
            comp.add_flag_to(text, flag)?;
        } else {
            panic!("unable to find Text Container");
        }

        Ok(())
    }

    pub fn remove_component_node_flag(&mut self, node: &Text, flag: ComponentFlag) -> Result<()> {
        if let Some(comp) = self.get_text_container_mut(node) {
            comp.remove_flag_from(node, flag)?;
        }

        Ok(())
    }
}

pub struct Listener {
    pub listener_id: ListenerId,

    pub on_event: Rc<RefCell<fn(ListenerId)>>,

    pub element: HtmlElement,
    pub function: Option<Closure<dyn Fn(MouseEvent)>>,

    pub data: SharedListenerData,

    toolbar: Toolbar,
}

pub fn register_with_data(
    element: HtmlElement,
    data: ListenerData,
    on_event: Rc<RefCell<fn(ListenerId)>>,
) -> Result<()> {
    LISTENERS.with(|listeners| -> Result<()> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let index = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = create_listener_class(*index);

        let listener_data = Rc::new(RefCell::new(data));
        let toolbar = Toolbar::new(index, &listener_data, &on_event)?;

        // Add class to container element
        element.class_list().add_1(&listener_class)?;

        let listener_rc = Rc::new(RefCell::new(Listener {
            listener_id: index,

            on_event,

            element,
            function: None,
            toolbar,

            data: listener_data,
        }));

        // Create the initial listener
        let listener = listener_rc.clone();
        let function = Closure::wrap(Box::new(move |event: MouseEvent| {
            handle_listener_mouseup(event, &listener_class, &listener).unwrap_throw();
        }) as Box<dyn Fn(MouseEvent)>);

        document()
            .add_event_listener_with_callback("mouseup", function.as_ref().unchecked_ref())?;

        listener_rc.borrow_mut().function = Some(function);

        listeners.push(listener_rc);

        Ok(())
    })?;

    Ok(())
}

/// Should be called AFTER page has fully loaded and finished any Element changes.
pub fn register(element: HtmlElement, on_event: Rc<RefCell<fn(ListenerId)>>) -> Result<()> {
    LISTENERS.with(|listeners| -> Result<()> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let listener_id = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = create_listener_class(*listener_id);

        let nodes = crate::node::return_all_text_nodes(&element);

        let listener_data = Rc::new(RefCell::new(ListenerData::new(nodes)?));
        let toolbar = Toolbar::new(listener_id, &listener_data, &on_event)?;

        // Add class to container element
        element.class_list().add_1(&listener_class)?;

        let listener_rc = Rc::new(RefCell::new(Listener {
            listener_id,

            on_event,

            element,
            function: None,
            toolbar,

            data: listener_data,
        }));

        // Create the initial listener
        let listener = listener_rc.clone();
        let function = Closure::wrap(Box::new(move |event: MouseEvent| {
            handle_listener_mouseup(event, &listener_class, &listener).unwrap_throw();
        }) as Box<dyn Fn(MouseEvent)>);

        document()
            .add_event_listener_with_callback("mouseup", function.as_ref().unchecked_ref())?;

        listener_rc.borrow_mut().function = Some(function);

        listeners.push(listener_rc);

        Ok(())
    })?;

    Ok(())
}

fn handle_listener_mouseup(
    event: MouseEvent,
    listening_class: &str,
    handler: &SharedListenerType,
) -> Result<()> {
    if !parents_contains_class(event.target_unchecked_into(), listening_class) {
        return Ok(());
    }

    if let Some(selection) = document().get_selection()?.filter(|v| !v.is_collapsed()) {
        let range = selection.get_range_at(0).unwrap_throw();

        let bb = range.get_bounding_client_rect();

        handler
            .borrow_mut()
            .toolbar
            .open(bb.x(), bb.y(), bb.width())?;
    } else {
        handler.borrow_mut().toolbar.close();
    }

    Ok(())
}

fn create_listener_class(index: usize) -> String {
    format!("edit-listener-{index}",)
}
