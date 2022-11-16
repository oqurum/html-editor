use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node, Text};

use crate::{Result, component::FlagsWithData, ComponentFlag};

#[derive(Clone)]
pub struct TextContainer {
    /// The non-split Text `Node` or split `Node`s
    pub(crate) text: Vec<ComponentNode>,
}

impl TextContainer {
    pub fn new(text: Text) -> Result<Self> {
        Ok(Self {
            text: vec![ComponentNode::wrap(text, 0, FlagsWithData::empty())?],
        })
    }

    pub fn get_by_text_index(&mut self, index: u32) -> Option<(u32, Text)> {
        self.text.iter().find_map(|v| {
            if index < v.offset + v.node.length() {
                Some((index - v.offset, v.node.clone()))
            } else {
                None
            }
        })
    }

    pub fn get_all_data_ids(&self) -> Vec<(ComponentFlag, u32)> {
        self.text.iter()
            .flat_map(|v| v.flag.data.clone())
            .collect()
    }

    pub fn has_flag(&self, flag: &FlagsWithData) -> bool {
        self.text.iter().any(|v| v.has_flag(flag))
    }

    pub fn are_all_flags_empty(&self) -> bool {
        self.text.iter().all(|v| v.are_flags_empty())
    }

    pub fn contains_node(&self, node: &Text) -> bool {
        self.text.iter().any(|v| &v.node == node)
    }

    pub fn get_node_mut(&mut self, node: &Text) -> Option<&mut ComponentNode> {
        self.text.iter_mut().find(|v| &v.node == node)
    }

    pub fn add_flag_to(&mut self, node: &Text, flag: FlagsWithData) -> Result<()> {
        if let Some((index, comp)) = self
            .text
            .iter_mut()
            .enumerate()
            .find(|(_, v)| &v.node == node)
        {
            comp.add_flag(flag)?;

            try_join_component_into_surroundings(index, &mut self.text)?;
        }

        Ok(())
    }

    pub fn set_flag_for(&mut self, node: &Text, flag: FlagsWithData) -> Result<()> {
        if let Some((index, comp)) = self
            .text
            .iter_mut()
            .enumerate()
            .find(|(_, v)| &v.node == node)
        {
            comp.set_flag(flag)?;

            try_join_component_into_surroundings(index, &mut self.text)?;
        }

        Ok(())
    }

    pub fn remove_flag_from(&mut self, node: &Text, flag: &FlagsWithData) -> Result<()> {
        if let Some((index, comp)) = self
            .text
            .iter_mut()
            .enumerate()
            .find(|(_, v)| &v.node == node)
        {
            comp.remove_flag(flag)?;

            try_join_component_into_surroundings(index, &mut self.text)?;
        }

        Ok(())
    }

    /// Splits and inserts the new ComponentNode in the correct position in the array.
    pub fn split_node(&mut self, node: &Text, index: u32) -> Result<Text> {
        for (i, item) in self.text.iter().enumerate() {
            if &item.node == node {
                let comp = item.split(index)?;
                let new_node = comp.node.clone();

                self.text.insert(i + 1, comp);

                return Ok(new_node);
            }
        }

        unreachable!()
    }
}

/// Nodes which have display wrappers for the text(?) components
#[derive(Clone)]
pub struct ComponentNode {
    /// Span container around the text node.
    container: HtmlElement,

    // TODO: Private
    pub node: Text,

    pub offset: u32,

    pub flag: FlagsWithData,
}

impl ComponentNode {
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

    pub fn change_flags_data(&mut self, flag: ComponentFlag, last_data_pos: u32, new_data_pos: u32) {
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

    pub fn has_flag(&self, value: &FlagsWithData) -> bool {
        self.flag.contains(value)
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
        self.container.set_class_name(&self.flag.generate_class_name());

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

/// Find and join Component Nodes' of the same type.
///
/// Updates the
pub(crate) fn try_join_component_into_surroundings(
    mut index: usize,
    nodes: &mut Vec<ComponentNode>,
) -> Result<()> {
    // Compare current and previous component.
    if index != 0 && nodes[index].flag == nodes[index - 1].flag {
        let curr = nodes.remove(index);

        nodes[index - 1].join(curr)?;

        index -= 1;
    }

    // Compare current and next component.
    if index + 1 < nodes.len() && nodes[index].flag == nodes[index + 1].flag {
        let next = nodes.remove(index + 1);

        nodes[index].join(next)?;
    }

    Ok(())
}

/// Return all text `Node`s which are between the start_node and the end_node parameters.
pub(crate) fn get_all_text_nodes_in_container(
    container: Node,
    start_node: &Node,
    end_node: &Node,
) -> Vec<Text> {
    if start_node == end_node {
        if container.node_type() == Node::TEXT_NODE {
            return vec![container.unchecked_into()];
        } else {
            return return_all_text_nodes(&container);
        }
    }

    fn find_inner(
        container: Node,
        start_node: &Node,
        end_node: &Node,
        has_passed_go: &mut bool,
        nodes: &mut Vec<Text>,
    ) -> bool {
        let mut inside = Some(container);

        while let Some(container) = inside {
            if &container == start_node {
                *has_passed_go = true;
            }

            if *has_passed_go && container.node_type() == Node::TEXT_NODE {
                log::info!("- {:?}", container.text_content());
                // TODO: Remove clone
                nodes.push(container.clone().unchecked_into());
            }

            if &container == end_node {
                log::info!("end");
                return true;
            }

            if let Some(child) = container.first_child() {
                if find_inner(child, start_node, end_node, has_passed_go, nodes) {
                    return true;
                }
            }

            inside = container.next_sibling();
        }

        false
    }

    let mut nodes = Vec::new();

    find_inner(container, start_node, end_node, &mut false, &mut nodes);

    nodes
}

/// Returns all text nodes in the `Node`.
pub(crate) fn return_all_text_nodes(container: &Node) -> Vec<Text> {
    let mut found = Vec::new();

    let mut inside = container.first_child();

    while let Some(container) = inside {
        if container.node_type() == Node::TEXT_NODE {
            found.push(container.clone().unchecked_into());
        }

        found.append(&mut return_all_text_nodes(&container));

        inside = container.next_sibling();
    }

    found
}
