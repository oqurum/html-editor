use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

use gloo_utils::document;
use lazy_static::lazy_static;
use serde::Serialize;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent, Text, Element, Node};

use crate::{
    helper::{parents_contains_class, TargetCast},
    node::{TextContainer, FoundWrappedTextRefMut},
    store,
    toolbar::Toolbar,
    Result, component::{FlagsWithData, ComponentDataStore, Context}, ComponentFlag, Component, selection, util::ElementEvent, WrappedText,
};

pub type SharedListenerType = Rc<RefCell<Listener>>;
pub type SharedListenerData = Weak<RefCell<ListenerData>>;
// TODO: Fix. I don't like all these Rc's

lazy_static! {
    static ref INCREMENT: AtomicUsize = AtomicUsize::default();
}

thread_local! {
    static LISTENERS: RefCell<Vec<SharedListenerType>> = RefCell::new(Vec::new());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListenerId(usize);

impl ListenerId {
    pub fn unset() -> Self {
        Self(0)
    }

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

        let save = store::save(&borrow2);

        Some(save).filter(|v| !v.nodes.is_empty())
    }
}

impl std::ops::Deref for ListenerId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ListenerData {
    pub(crate) listener_id: ListenerId,

    /// Specific Data stored for Components.
    pub(crate) data: Vec<ComponentDataStore>,
    /// The Text Nodes inside the listener Element. Along with flags for the Text.
    pub(crate) nodes: Vec<TextContainer>,
}

impl ListenerData {
    pub fn new(listener_id: ListenerId, nodes: Vec<Text>) -> Result<Self> {
        Ok(Self {
            listener_id,
            data: Vec::new(),
            nodes: nodes
                .into_iter()
                .map(TextContainer::new)
                .collect::<Result<_>>()?,
        })
    }

    pub fn store_data<S: Serialize>(&mut self, flag: ComponentFlag, data: &S) -> u32 {
        let len = self.data.len() as u32;

        self.data.push(ComponentDataStore::new(flag, data));

        len
    }

    // Flag is only used for assert.
    pub fn get_data(&self, flag: ComponentFlag, data_index: u32) -> ComponentDataStore {
        let data_item = self.data[data_index as usize].clone();

        assert_eq!(data_item.0, flag);

        data_item
    }

    pub fn update_data<S: Serialize>(&mut self, flag: ComponentFlag, data_index: u32, data: &S) {
        let data_item = &mut self.data[data_index as usize];

        assert_eq!(data_item.0, flag);

        *data_item = ComponentDataStore::new(flag, data);
    }

    pub fn remove_data(&mut self, flag: ComponentFlag, data_index: u32) {
        let value = self.data.swap_remove(data_index as usize);
        let last_data_pos = self.data.len() as u32;

        assert_eq!(value.0, flag);

        // TODO: Update other data nodes with new Data Position.
        for node in &mut self.nodes {
            for text in &mut node.text {
                text.change_flags_data(flag, last_data_pos, data_index);
            }
        }
    }

    // TODO: Optimize. We're iterating through two arrays.

    pub fn get_text_container_for_node(&self, node: &Text) -> Option<&TextContainer> {
        self.nodes.iter().find(|v| v.contains_node(node))
    }

    pub fn get_text_wrapper(&self, node: &Text) -> Option<&WrappedText> {
        self.nodes.iter().find_map(|v| v.get_wrapped_text(node))
    }

    pub fn get_text_container_mut(&mut self, node: &Text) -> Option<FoundWrappedTextRefMut<'_>> {
        self.nodes.iter_mut().find_map(|v| v.find_node_return_mut_ref(node))
    }

    pub fn update_container(&mut self, text: &Text, flag: FlagsWithData) -> Result<()> {
        if let Some(mut comp) = self.get_text_container_mut(text) {
            comp.add_flag_to(flag)?;
        } else {
            panic!("unable to find Text Container");
        }

        Ok(())
    }

    pub fn remove_component_node_flag(&mut self, node: &Text, flag: &FlagsWithData) -> Result<()> {
        if let Some(mut comp) = self.get_text_container_mut(node) {
            comp.remove_flag_from(flag)?;
        }

        Ok(())
    }
}


pub struct Listener {
    pub listener_id: ListenerId,

    pub on_event: Rc<RefCell<dyn Fn(ListenerId)>>,

    pub element: HtmlElement,
    functions: Vec<ElementEvent>,

    pub data: Rc<RefCell<ListenerData>>,

    toolbar: Toolbar,
}

/// Keeps the listener active until we're dropped.
pub struct ListenerHandle(ListenerId);

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();

            if let Some(index) = listeners.iter().position(|v| v.borrow().listener_id == self.0) {
                let listener = listeners.remove(index);

                let listener_class = create_listener_class(*self.0);

                let _ = listener.borrow().element.class_list().remove_1(&listener_class);

                log::debug!("Dropping Handle {:?}", self.0);
            }
        });
    }
}

pub fn register_with_data(
    element: HtmlElement,
    mut data: ListenerData,
    on_event: Rc<RefCell<dyn Fn(ListenerId)>>,
) -> Result<ListenerHandle> {
    LISTENERS.with(|listeners| -> Result<ListenerHandle> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let index = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = create_listener_class(*index);

        data.listener_id = index;
        let listener_data = Rc::new(RefCell::new(data));
        let toolbar = Toolbar::new(index, Rc::downgrade(&listener_data), &on_event)?;

        // Add class to container element
        element.class_list().add_1(&listener_class)?;

        let listener_rc = Rc::new(RefCell::new(Listener {
            listener_id: index,

            on_event,

            element,
            functions: Vec::new(),
            toolbar,

            data: listener_data,
        }));

        register_listener_events(&listener_rc, listener_class)?;

        listeners.push(listener_rc);

        Ok(ListenerHandle(index))
    })
}

/// Should be called AFTER page has fully loaded and finished any Element changes.
pub fn register(element: HtmlElement, on_event: Rc<RefCell<dyn Fn(ListenerId)>>) -> Result<ListenerHandle> {
    LISTENERS.with(|listeners| -> Result<ListenerHandle> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let listener_id = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = create_listener_class(*listener_id);

        let nodes = crate::node::return_all_text_nodes(&element);

        let listener_data = Rc::new(RefCell::new(ListenerData::new(listener_id, nodes)?));
        let toolbar = Toolbar::new(listener_id, Rc::downgrade(&listener_data), &on_event)?;

        // Add class to container element
        element.class_list().add_1(&listener_class)?;

        let listener_rc = Rc::new(RefCell::new(Listener {
            listener_id,

            on_event,

            element,
            functions: Vec::new(),
            toolbar,

            data: listener_data,
        }));

        register_listener_events(&listener_rc, listener_class)?;

        listeners.push(listener_rc);

        Ok(ListenerHandle(listener_id))
    })
}

fn register_listener_events(listener_rc: &Rc<RefCell<Listener>>, listener_class: String) -> Result<()> {
    let is_mouse_down = Rc::new(RefCell::new(false));

    // Create the mouse move listener
    {
        let listener = Rc::downgrade(listener_rc);
        let is_mouse_down = is_mouse_down.clone();

        let function: Closure<dyn FnMut(MouseEvent)> = Closure::new(move |_event: MouseEvent| {
            if *is_mouse_down.borrow() {
                display_toolbar(&listener).unwrap_throw();
            }
        });

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document().unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("mousemove", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("mousemove", f)),
        ));
    }

    // Create the mouse up listener
    {
        let listener = Rc::downgrade(listener_rc);
        let is_mouse_down = is_mouse_down.clone();

        let function = Closure::wrap(Box::new(move |_event: MouseEvent| {
            *is_mouse_down.borrow_mut() = false;

            display_toolbar(&listener).unwrap_throw();
        }) as Box<dyn Fn(MouseEvent)>);

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document().unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("mouseup", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("mouseup", f)),
        ));
    }

    // Create the mouse down listener
    {
        let listener_class = listener_class.clone();

        let function = Closure::wrap(Box::new(move |event: MouseEvent| {
            if parents_contains_class(event.target_unchecked_into(), &listener_class) {
                *is_mouse_down.borrow_mut() = true;
            }
        }) as Box<dyn Fn(MouseEvent)>);

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document().unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("mousedown", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("mousedown", f)),
        ));
    }

    // Create the on click listener
    {
        let listener = Rc::downgrade(listener_rc);
        let function: Closure<dyn FnMut(MouseEvent)> = Closure::new(move |event: MouseEvent| {
            handle_listener_mouseclick(event, &listener_class, &listener).unwrap_throw();
        });

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document().unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("click", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
        ));
    }

    Ok(())
}




fn handle_listener_mouseclick(
    event: MouseEvent,
    listening_class: &str,
    handler: &Weak<RefCell<Listener>>,
) -> Result<()> {
    if !parents_contains_class(event.target_unchecked_into(), listening_class) {
        return Ok(());
    }

    if document().get_selection()?.unwrap_throw().is_collapsed() {
        let handle = handler.upgrade().expect_throw("Upgrade Listener");
        let handle = handle.borrow();
        let data = handle.data.borrow();

        let mut flags = ComponentFlag::empty();

        let mut text_nodes = Vec::new();
        let mut inside = event.target_unchecked_into::<Element>().first_child();

        // TODO: Improve
        while let Some(container) = inside {
            inside = container.next_sibling();

            if container.node_type() == Node::TEXT_NODE {
                if let Some(cont) = data.get_text_container_for_node(container.unchecked_ref()) {
                    for comp_node in &cont.text {
                        if comp_node.node.unchecked_ref::<Node>() == &container {
                            text_nodes.push(container.unchecked_into());
                            flags.insert(comp_node.flag.flag);

                            break;
                        }
                    }
                }
            }
        }

        let ctx = Context {
            nodes: Rc::new(RefCell::new(selection::create_container(text_nodes, Rc::downgrade(&handle.data)).unwrap_throw()))
        };

        // TODO: Improve
        for flag in flags.separate_bits() {
            match flag {
                ComponentFlag::ITALICIZE => crate::component::Italicize.on_click(&ctx).unwrap_throw(),
                ComponentFlag::HIGHLIGHT => crate::component::Highlight.on_click(&ctx).unwrap_throw(),
                ComponentFlag::UNDERLINE => crate::component::Underline.on_click(&ctx).unwrap_throw(),
                ComponentFlag::NOTE => crate::component::Note.on_click(&ctx).unwrap_throw(),

                _ => unreachable!(),
            }
        }
    }

    Ok(())
}


fn display_toolbar(handler: &Weak<RefCell<Listener>>) -> Result<()> {
    let handler = handler.upgrade().expect_throw("Upgrade Listener");
    let mut handler = handler.borrow_mut();

    if let Some(selection) = document().get_selection()?.filter(|v| !v.is_collapsed()) {
        let range = selection.get_range_at(0).unwrap_throw();

        let bb = range.get_bounding_client_rect();

        handler.toolbar.open(bb.x(), bb.y(), bb.width())?;
    } else {
        handler.toolbar.close();
    }

    Ok(())
}

fn create_listener_class(index: usize) -> String {
    format!("edit-listener-{index}",)
}
