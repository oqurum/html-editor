use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

use lazy_static::lazy_static;
use serde::Serialize;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Document, Element, HtmlElement, MouseEvent, Node, Range, Text};

use crate::{
    component::{ComponentDataStore, Context, FlagsWithData},
    document,
    helper::{parents_contains_class, TargetCast},
    selection, store,
    text::{return_all_text_nodes, FoundWrappedTextRefMut, TextContentWithFlag},
    toolbar::Toolbar,
    util::ElementEvent,
    Component, ComponentFlag, Result, TextContainer, WrappedText,
};

pub type SharedListenerType = Rc<RefCell<Listener>>;
pub type SharedListenerData = Weak<RefCell<ListenerData>>;
pub type ListenerEvent = Rc<RefCell<dyn Fn(ListenerId)>>;
// TODO: Fix. I don't like all these Rc's

lazy_static! {
    static ref INCREMENT: AtomicUsize = AtomicUsize::default();
}

thread_local! {
    static LISTENERS: RefCell<Vec<SharedListenerType>> = RefCell::new(Vec::new());
}

#[derive(PartialEq, Eq)]
pub enum MouseListener {
    All,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(usize);

impl ListenerId {
    pub fn document(&self) -> Document {
        document::get_document(*self).unwrap_or_else(gloo_utils::document)
    }

    pub fn unset() -> Self {
        Self(0)
    }

    fn to_class_string(self) -> String {
        format!("edit-listener-{}", self.0)
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

    // TODO: Put into a better location
    pub fn get_flagged_text(&self) -> Vec<TextContentWithFlag> {
        let mut found = Vec::new();

        let v = self.nodes.iter().flat_map(|c| c.text.iter()).fold(
            Option::<TextContentWithFlag>::None,
            |mut store, text| {
                if text.flag.is_empty() {
                    if let Some(value) = store.take() {
                        found.push(value);
                    }
                } else if let Some(value) = store.as_mut() {
                    value.flag.insert(text.flag.flag);
                    value.content += &text.node.text_content().unwrap();
                } else {
                    store = Some(TextContentWithFlag {
                        flag: text.flag.flag,
                        content: text.node.text_content().unwrap(),
                        first_text_node: text.node.clone(),
                    });
                }

                store
            },
        );

        if let Some(value) = v {
            found.push(value);
        }

        found
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
        self.nodes
            .iter_mut()
            .find_map(|v| v.find_node_return_mut_ref(node))
    }

    pub fn update_container(&mut self, text: &Text, flag: FlagsWithData) -> Result<()> {
        if let Some(mut comp) = self.get_text_container_mut(text) {
            comp.add_flag_to(flag)?;
            comp.rejoin_into_surrounding()?;
        } else {
            panic!("unable to find Text Container");
        }

        Ok(())
    }

    pub fn remove_component_node_flag(&mut self, node: &Text, flag: &FlagsWithData) -> Result<()> {
        if let Some(mut comp) = self.get_text_container_mut(node) {
            comp.remove_flag_from(flag)?;
            comp.rejoin_into_surrounding()?;
        }

        Ok(())
    }
}

pub struct Listener {
    pub listener_id: ListenerId,

    pub on_event: ListenerEvent,

    pub element: HtmlElement,
    functions: Vec<ElementEvent>,

    pub data: Rc<RefCell<ListenerData>>,

    pub(crate) toolbar: Toolbar,
}

/// Keeps the listener active until we're dropped.
pub struct ListenerHandle(ListenerId);

impl ListenerHandle {
    pub fn unset() -> Self {
        Self(ListenerId::unset())
    }

    /// Selects a word from the point you clicked and will also call `on_click` for a component.
    pub fn click_or_select(&self, x: f32, y: f32, target: Element) -> Result<()> {
        let Some(listener) = self.0.try_get() else {
            log::debug!("Unable to acquire listener. Does it still exist?");
            return Ok(());
        };

        handle_listener_mouseclick(
            target,
            &self.0.to_class_string(),
            &self.0.document(),
            &Rc::downgrade(&listener),
        )?;

        self.select_word_from_point(x, y, &self.0.document())
    }

    /// Highlights' the word you clicked and opens the toolbar.
    fn select_word_from_point(&self, x: f32, y: f32, document: &Document) -> Result<(), JsValue> {
        fn is_stop_break(value: &u8) -> bool {
            [
                b' ', b'.', b',', b'?', b'!', b'"', b'\'', b'*', b'(', b')', b';', b':',
            ]
            .contains(value)
        }

        let Some(caret) = document.caret_position_from_point(x, y) else {
            log::debug!("Unable to get Caret Position");
            return Ok(());
        };

        let Some(node) = caret.offset_node() else {
            log::debug!("Unable to find Caret Offset Node");
            return Ok(());
        };

        if !parents_contains_class(node.parent_element().unwrap(), &self.0.to_class_string()) {
            return Ok(());
        }

        let Some(node_value) = node.node_value() else {
            log::debug!("Unable to get Node Value");
            return Ok(());
        };

        let node_offset = caret.offset() as usize;

        let mut start_offset = node_offset;
        let mut length = node_value.len();

        // TODO: Optimize. We need to check other Nodes.
        if node_value
            .as_bytes()
            .get(start_offset)
            .map(is_stop_break)
            .unwrap_or_default()
        {
            start_offset += 1;

            for i in 0..length {
                if let Some(next_byte) = node_value.as_bytes().get(start_offset + i) {
                    if is_stop_break(next_byte) {
                        length = i;
                        break;
                    }
                } else {
                    break;
                }
            }
        } else {
            // Backtrack
            for i in (0..start_offset).rev() {
                if is_stop_break(node_value.as_bytes().get(i).unwrap()) {
                    start_offset = i + 1;
                    break;
                } else if i == 0 {
                    start_offset = 0;
                }
            }

            // Move until space
            for i in 0..length {
                if let Some(next_byte) = node_value.as_bytes().get(node_offset + i) {
                    if is_stop_break(next_byte) {
                        length = node_offset - start_offset + i;
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        let Some(selection) = document.get_selection()? else {
            log::debug!("Unable to get selection");
            return Ok(());
        };

        let Some(listener) = self.0.try_get() else {
            log::debug!("Unable to acquire listener. Does it still exist?");
            return Ok(());
        };

        let handler = Rc::downgrade(&listener);

        // TODO: If we disabled events and we double click OOBs this can be called and displays the toolbar.
        if selection.range_count() != 0 {
            display_toolbar(&handler)?;
        }

        if start_offset == start_offset + length {
            log::debug!("Unable to find word");
            return Ok(());
        }

        // Check if we're out of bounds
        if node_value.len() < start_offset + length {
            log::debug!("Invalid Selection");
            return Ok(());
        }

        // Selection Range Changes
        selection.remove_all_ranges()?;

        let range = Range::new()?;

        range.set_start(&node, start_offset as u32)?;
        range.set_end(&node, (start_offset + length) as u32)?;

        selection.add_range(&range)?;

        // Open Toolbar

        display_toolbar(&handler)?;

        Ok(())
    }
}

impl Default for ListenerHandle {
    fn default() -> Self {
        Self::unset()
    }
}

impl Drop for ListenerHandle {
    fn drop(&mut self) {
        LISTENERS.with(|listeners| {
            let mut listeners = listeners.borrow_mut();

            if let Some(index) = listeners
                .iter()
                .position(|v| v.borrow().listener_id == self.0)
            {
                let listener = listeners.remove(index);

                document::delete_document(listener.borrow().listener_id);

                let listener_class = self.0.to_class_string();

                let _ = listener
                    .borrow()
                    .element
                    .class_list()
                    .remove_1(&listener_class);

                log::debug!("Dropping Handle {:?}", self.0);
            }
        });
    }
}

pub fn register_with_data(
    element: HtmlElement,
    mut data: ListenerData,
    listener: MouseListener,
    document: Option<Document>,
    on_event: Option<ListenerEvent>,
) -> Result<ListenerHandle> {
    LISTENERS.with(|listeners| -> Result<ListenerHandle> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let on_event = match on_event {
            Some(v) => v,
            None => Rc::new(RefCell::new(|_| {})) as ListenerEvent,
        };

        let index = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = index.to_class_string();

        if let Some(document) = document {
            document::register_document(index, document);
        }

        data.listener_id = index;
        let listener_data = Rc::new(RefCell::new(data));
        let toolbar = Toolbar::new(index, Rc::downgrade(&listener_data), &on_event);

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

        if listener == MouseListener::All {
            register_listener_events(&listener_rc, listener_class)?;
        }

        listeners.push(listener_rc);

        Ok(ListenerHandle(index))
    })
}

/// Should be called AFTER page has fully loaded and finished any Element changes.
pub fn register(
    element: HtmlElement,
    listener: MouseListener,
    document: Option<Document>,
    on_event: Option<ListenerEvent>,
) -> Result<ListenerHandle> {
    LISTENERS.with(|listeners| -> Result<ListenerHandle> {
        let mut listeners = listeners.borrow_mut();

        // TODO: Check that we also aren't inside another listener.
        if listeners.iter().any(|l| l.borrow().element == element) {
            panic!("Already Listening on Element!");
        }

        let on_event = match on_event {
            Some(v) => v,
            None => Rc::new(RefCell::new(|_| {})) as ListenerEvent,
        };

        let listener_id = ListenerId(INCREMENT.fetch_add(1, Ordering::Relaxed));
        let listener_class = listener_id.to_class_string();

        if let Some(document) = document {
            document::register_document(listener_id, document);
        }

        let nodes = return_all_text_nodes(&element);

        let listener_data = Rc::new(RefCell::new(ListenerData::new(listener_id, nodes)?));
        let toolbar = Toolbar::new(listener_id, Rc::downgrade(&listener_data), &on_event);

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

        if listener == MouseListener::All {
            register_listener_events(&listener_rc, listener_class)?;
        }

        listeners.push(listener_rc);

        Ok(ListenerHandle(listener_id))
    })
}

fn register_listener_events(
    listener_rc: &Rc<RefCell<Listener>>,
    listener_class: String,
) -> Result<()> {
    let document = listener_rc.borrow().listener_id.document();
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
            document.clone().unchecked_into(),
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
            let mut md = is_mouse_down.borrow_mut();

            if *md {
                display_toolbar(&listener).unwrap_throw();
            }

            *md = false;
        }) as Box<dyn Fn(MouseEvent)>);

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document.clone().unchecked_into(),
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
            document.clone().unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("mousedown", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("mousedown", f)),
        ));
    }

    // Create the on click listener
    {
        let document2 = document.clone();
        let listener = Rc::downgrade(listener_rc);
        let function: Closure<dyn FnMut(MouseEvent)> = Closure::new(move |event: MouseEvent| {
            handle_listener_mouseclick(
                event.target_unchecked_into(),
                &listener_class,
                &document2,
                &listener,
            )
            .unwrap_throw();
        });

        listener_rc.borrow_mut().functions.push(ElementEvent::link(
            document.unchecked_into(),
            function,
            |t, f| t.add_event_listener_with_callback("click", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
        ));
    }

    Ok(())
}

fn handle_listener_mouseclick(
    target: Element,
    listening_class: &str,
    document: &Document,
    handler: &Weak<RefCell<Listener>>,
) -> Result<()> {
    if !parents_contains_class(target.clone(), listening_class) {
        return Ok(());
    }

    if document.get_selection()?.unwrap_throw().is_collapsed() {
        let handle = handler.upgrade().expect_throw("Upgrade Listener");
        let handle = handle.borrow();
        let data = handle.data.borrow();

        let mut flags = ComponentFlag::empty();

        let mut text_nodes = Vec::new();
        let mut inside = target.first_child();

        // TODO: Improve
        // Retrieve all text nodes inside the element we clicked on
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

        let nodes = Rc::new(RefCell::new(
            selection::create_container(text_nodes, Rc::downgrade(&handle.data)).unwrap_throw(),
        ));

        // TODO: Improve
        for flag in flags.separate_bits() {
            match flag {
                ComponentFlag::ITALICIZE => crate::component::Italicize
                    .on_click(&Context::new(nodes.clone(), document.clone()))
                    .unwrap_throw(),
                ComponentFlag::HIGHLIGHT => crate::component::Highlight
                    .on_click(&Context::new(nodes.clone(), document.clone()))
                    .unwrap_throw(),
                ComponentFlag::UNDERLINE => crate::component::Underline
                    .on_click(&Context::new(nodes.clone(), document.clone()))
                    .unwrap_throw(),
                ComponentFlag::NOTE => crate::component::Note
                    .on_click(&Context::new(nodes.clone(), document.clone()))
                    .unwrap_throw(),

                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

fn display_toolbar(handler: &Weak<RefCell<Listener>>) -> Result<()> {
    let handler = handler.upgrade().expect_throw("Upgrade Listener");
    let mut handler = handler.borrow_mut();

    if let Some(selection) = handler
        .listener_id
        .document()
        .get_selection()?
        .filter(|v| !v.is_collapsed())
    {
        handler.toolbar.open(selection)?;
    } else {
        handler.toolbar.close();
    }

    Ok(())
}
