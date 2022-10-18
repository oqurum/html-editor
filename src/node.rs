use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node, Text};

use crate::{ComponentFlag, Result};

/// Nodes which have display wrappers for the text(?) components
#[derive(Clone)]
pub struct ComponentNode {
    /// Span container around the text node.
    container: HtmlElement,

    pub node: Text,

    flag: ComponentFlag,
}

impl ComponentNode {
    pub fn wrap(value: Text, flag: ComponentFlag) -> Result<Self> {
        Ok(Self {
            container: create_container(&value, flag)?,
            node: value,
            flag,
        })
    }

    pub fn split(&self, index: u32) -> Result<Self> {
        let text_split = self.node.split_text(index)?;

        Ok(Self {
            container: create_container(&text_split, self.flag)?,
            node: text_split,
            flag: self.flag,
        })
    }

    pub fn unwrap(self) -> Result<Text> {
        self.container.replace_with_with_node_1(&self.node)?;

        Ok(self.node)
    }

    pub fn are_flags_empty(&self) -> bool {
        self.flag.is_empty()
    }

    pub fn has_flag(&self, value: ComponentFlag) -> bool {
        self.flag.contains(value)
    }

    pub fn remove_flag(&mut self, value: ComponentFlag) {
        self.flag.remove(value);
        self.update_class();
    }

    pub fn add_flag(&mut self, value: ComponentFlag) {
        self.flag.insert(value);
        self.update_class();
    }

    pub fn set_flag(&mut self, value: ComponentFlag) {
        self.flag = value;
        self.update_class();
    }

    fn update_class(&self) {
        self.container.set_class_name(&self.flag.into_class_names());
    }
}

fn create_container(text_node: &Text, flag: ComponentFlag) -> Result<HtmlElement> {
    let container = document().create_element("span")?;
    container.set_class_name(&flag.into_class_names());

    text_node.before_with_node_1(&container)?;

    container.append_child(text_node)?;

    Ok(container.unchecked_into())
}

/// Find and join Nodes' surroundings for Nodes of the same type.
///
/// Returns the NEW text node or INSERTED text node
pub(crate) fn join_node_into_surroundings(mut value: Text) -> Result<Text> {
    // Check the previous sibling
    if let Some(prev_sib) = value.previous_sibling() {
        if prev_sib.node_type() == Node::TEXT_NODE {
            let prev_sib: Text = prev_sib.unchecked_into();

            if let Some(text) = value.text_content() {
                prev_sib.append_data(&text)?;
            }

            value.remove();
            value = prev_sib;
        }
    }

    // Check the next sibling
    if let Some(next_sib) = value.next_sibling() {
        if next_sib.node_type() == Node::TEXT_NODE {
            let next_sib: Text = next_sib.unchecked_into();

            if let Some(text) = next_sib.text_content() {
                value.append_data(&text)?;
            }

            next_sib.remove();
        }
    }

    Ok(value)
}
