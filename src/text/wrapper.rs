use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Text};

use crate::{component::FlagsWithData, ComponentFlag, Result};

/// Contains the Text Node which can be changed with certain flags.
#[derive(Debug, Clone)]
pub struct WrappedText {
    /// Span container around the text node.
    container: HtmlElement,

    // TODO: Private
    pub node: Text,

    pub offset: u32,

    pub flag: FlagsWithData,
}

impl WrappedText {
    pub fn wrap(text: Text, offset: u32, flag: FlagsWithData) -> Result<Self> {
        Ok(Self {
            container: create_container(&text, flag.clone())?,
            node: text,
            offset,
            flag,
        })
    }

    pub fn split(&self, index: u32) -> Result<Self> {
        let text_split = self.node.split_text(index)?;
        // Move Text Split to outer container layer. It'll we wrapped with container
        self.container.after_with_node_1(&text_split)?;

        Ok(Self {
            container: create_container(&text_split, self.flag.clone())?,
            node: text_split,
            flag: self.flag.clone(),
            offset: self.offset + index,
        })
    }

    pub fn unwrap(&self) -> Result<()> {
        // TODO: If only the Text is selected (get_selection()) then the outer parent of self.container will be selected for a tick or two.
        self.container.replace_with_with_node_1(&self.node)?;

        Ok(())
    }

    pub fn join(&mut self, other: Self) -> Result<()> {
        if let Some(text) = other.node.text_content() {
            self.node.append_data(&text)?;
        }

        other.remove();

        Ok(())
    }

    pub fn change_flags_data(
        &mut self,
        flag: ComponentFlag,
        last_data_pos: u32,
        new_data_pos: u32,
    ) {
        if self.flag.flag.contains(flag) {
            for (comp_flag, data) in &mut self.flag.data {
                if flag == *comp_flag && last_data_pos == *data {
                    *data = new_data_pos;
                    break;
                }
            }
        }
    }

    pub fn remove(&self) {
        self.container.remove();
        self.node.remove();
    }

    pub fn are_flags_empty(&self) -> bool {
        self.flag.is_empty()
    }

    pub fn intersects_flag(&self, value: ComponentFlag) -> bool {
        self.flag.intersects_flag(value)
    }

    pub fn has_flag(&self, value: &FlagsWithData) -> bool {
        self.flag.contains(value)
    }

    pub fn remove_all_flag(&mut self) -> Result<()> {
        self.flag.clear();
        self.update_container()
    }

    pub fn remove_flag(&mut self, value: &FlagsWithData) -> Result<()> {
        self.flag.remove(value);
        self.update_container()
    }

    pub fn add_flag(&mut self, value: FlagsWithData) -> Result<()> {
        self.flag.insert(value);
        self.update_container()
    }

    pub fn set_flag(&mut self, value: FlagsWithData) -> Result<()> {
        self.flag = value;
        self.update_container()
    }

    fn update_container(&self) -> Result<()> {
        self.container
            .set_class_name(&self.flag.generate_class_name());

        if self.flag.is_empty() && self.container.parent_element().is_some() {
            // Unwrap the container.
            self.unwrap()?;
        } else if !self.flag.is_empty() && self.container.parent_element().is_none() {
            // Wrap the Text Node
            self.node.before_with_node_1(&self.container)?;
            self.container.append_child(&self.node)?;
        }

        Ok(())
    }
}

fn create_container(text_node: &Text, flag: FlagsWithData) -> Result<HtmlElement> {
    let container = document().create_element("span")?;
    container.set_class_name(&flag.generate_class_name());

    if !flag.is_empty() {
        text_node.before_with_node_1(&container)?;
        container.append_child(text_node)?;
    }

    Ok(container.unchecked_into())
}
