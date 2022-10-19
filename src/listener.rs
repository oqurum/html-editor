use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use gloo_utils::document;
use lazy_static::lazy_static;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{HtmlElement, MouseEvent, Text};

use crate::{
    helper::{parents_contains_class, TargetCast},
    node::{join_node_into_surroundings, try_join_component_into_surroundings},
    toolbar::Toolbar,
    ComponentFlag, ComponentNode, Result,
};

type SharedListenerType = Rc<RefCell<Listener>>;
pub type SharedListenerData = Rc<RefCell<ListenerData>>;
// TODO: Fix. I don't like all these Rc's

lazy_static! {
    static ref INCREMENT: AtomicUsize = AtomicUsize::default();
}

thread_local! {
    static LISTENERS: RefCell<Vec<SharedListener>> = RefCell::new(Vec::new());
}

#[derive(Clone)]
struct SharedListener(SharedListenerType);

impl SharedListener {
    pub fn write(&self) -> RefMut<Listener> {
        self.0.borrow_mut()
    }
}

#[derive(Default)]
pub struct ListenerData {
    /// Registered Component Nodes
    pub nodes: Vec<ComponentNode>,
}

impl ListenerData {
    pub fn get_component_node(&self, node: &Text) -> Option<&ComponentNode> {
        self.nodes.iter().find(|v| &v.node == node)
    }

    pub fn get_component_node_mut(&mut self, node: &Text) -> Option<&mut ComponentNode> {
        self.nodes.iter_mut().find(|v| &v.node == node)
    }

    pub fn insert_or_update_component(&mut self, value: Text, flag: ComponentFlag) -> Result<()> {
        if let Some(index) = self.nodes.iter().position(|v| v.node == value) {
            let mut comp = self.nodes.remove(index);
            comp.add_flag(flag);

            try_join_component_into_surroundings(comp, &mut self.nodes)?;
        } else {
            try_join_component_into_surroundings(
                ComponentNode::wrap(value, flag)?,
                &mut self.nodes,
            )?;
        }

        Ok(())
    }

    pub fn remove_component_node_flag(
        &mut self,
        node: &Text,
        flag: ComponentFlag,
    ) -> Result<Option<Text>> {
        if let Some(index) = self.nodes.iter().position(|v| &v.node == node) {
            let mut comp = self.nodes.swap_remove(index);

            comp.remove_flag(flag);

            if comp.are_flags_empty() {
                let node = Self::handle_component_unwrap(comp)?;
                return Ok(Some(node));
            } else {
                try_join_component_into_surroundings(comp, &mut self.nodes)?;
            }
        }

        Ok(None)
    }

    /// Unwrap the component and join Text Node into surrounding nodes of same type.
    fn handle_component_unwrap(value: ComponentNode) -> Result<Text> {
        let node = value.unwrap()?;

        // Check the Nodes' surroundings for Nodes of the same type.
        let node = join_node_into_surroundings(node)?;

        Ok(node)
    }
}

pub struct Listener {
    pub index: usize,

    pub element: HtmlElement,
    pub function: Option<Closure<dyn Fn(MouseEvent)>>,

    pub data: SharedListenerData,

    toolbar: Toolbar,
}

pub fn register(element: HtmlElement) -> Result<()> {
    LISTENERS.with(|listeners| -> Result<()> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.write().element == element) {
            panic!("Already Listening on Element!");
        }

        let index = INCREMENT.fetch_add(1, Ordering::Relaxed);
        let listener_class = create_listener_class(index);

        let listener_data = Rc::new(RefCell::new(ListenerData::default()));
        let toolbar = Toolbar::new(&listener_data)?;

        // Add class to container element
        element.class_list().add_1(&listener_class)?;

        let listener_rc = SharedListener(Rc::new(RefCell::new(Listener {
            index,

            element,
            function: None,
            toolbar,

            data: listener_data,
        })));

        // Create the initial listener
        let listener = listener_rc.clone();
        let function = Closure::wrap(Box::new(move |event: MouseEvent| {
            handle_listener_mouseup(event, &listener_class, &listener).unwrap_throw();
        }) as Box<dyn Fn(MouseEvent)>);

        document()
            .add_event_listener_with_callback("mouseup", function.as_ref().unchecked_ref())?;

        listener_rc.write().function = Some(function);

        listeners.push(listener_rc);

        Ok(())
    })?;

    Ok(())
}

fn handle_listener_mouseup(
    event: MouseEvent,
    listening_class: &str,
    handler: &SharedListener,
) -> Result<()> {
    if !parents_contains_class(event.target_unchecked_into(), listening_class) {
        return Ok(());
    }

    if let Some(selection) = document().get_selection()?.filter(|v| !v.is_collapsed()) {
        let range = selection.get_range_at(0).unwrap_throw();

        let bb = range.get_bounding_client_rect();

        handler.write().toolbar.open(bb.x(), bb.y(), bb.width())?;
    } else {
        handler.write().toolbar.close();
    }

    Ok(())
}

fn create_listener_class(index: usize) -> String {
    format!("edit-listener-{index}",)
}
