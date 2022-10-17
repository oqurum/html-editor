use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Text};

use crate::{ComponentFlag, Result};

/// Nodes which have display wrappers for the text(?) components
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

    pub fn has_flag(&mut self, value: ComponentFlag) -> bool {
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

    container.append_child(&text_node)?;

    Ok(container.unchecked_into())
}
