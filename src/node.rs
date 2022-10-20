use gloo_utils::document;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Node, Text};

use crate::{ComponentFlag, Result, selection::SplitText};

/// Nodes which have display wrappers for the text(?) components
#[derive(Clone)]
pub struct ComponentNode {
    /// Span container around the text node.
    container: HtmlElement,


    pub node: Text,

    pub offset: u32,

    flag: ComponentFlag,
}

impl ComponentNode {
    pub fn wrap(text: Text, offset: u32, flag: ComponentFlag) -> Result<Self> {
        Ok(Self {
            container: create_container(&text, flag)?,
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
            container: create_container(&text_split, self.flag)?,
            node: text_split,
            flag: self.flag,
            offset: self.offset + index,
        })
    }

    pub fn unwrap(self) -> Result<Text> {
        // TODO: If only the Text is selected (get_selection()) then the outer parent of self.container will be selected for a tick or two.
        self.container.replace_with_with_node_1(&self.node)?;

        Ok(self.node)
    }

    pub fn join(&mut self, other: Self) -> Result<()> {
        other.remove();

        if let Some(text) = other.node.text_content() {
            self.node.append_data(&text)?;
        }

        Ok(())
    }

    pub fn remove(&self) {
        self.container.remove();
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

/// Find and join Nodes' of the same type.
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

/// Find and join Component Nodes' of the same type.
///
/// Updates the
pub(crate) fn try_join_component_into_surroundings(
    mut value: ComponentNode,
    nodes: &mut Vec<ComponentNode>,
) -> Result<()> {
    // Check the previous sibling
    if let Some(prev_sib) = value.container.previous_sibling() {
        if let Some(index) = nodes
            .iter()
            .position(|v| v.container.unchecked_ref::<Node>() == &prev_sib && value.flag == v.flag)
        {
            let mut prev_sib = nodes.remove(index);

            prev_sib.join(value)?;

            value = prev_sib;
        }
    }

    // Check the next sibling
    if let Some(next_sib) = value.container.next_sibling() {
        if let Some(index) = nodes
            .iter()
            .position(|v| v.container.unchecked_ref::<Node>() == &next_sib && value.flag == v.flag)
        {
            value.join(nodes.remove(index))?;
        }
    }

    nodes.push(value);

    Ok(())
}

/// Return all text `Node`s which are between the start_node and the end_node parameters.
pub(crate) fn get_all_text_nodes_in_container(
    container: Node,
    start_node: &Node,
    end_node: &Node,
) -> Vec<SplitText> {
    if start_node == end_node {
        if container.node_type() == Node::TEXT_NODE {
            return vec![SplitText { text: container.unchecked_into(), offset: 0 }];
        } else {
            return return_all_text_nodes(&container);
        }
    }

    fn find_inner(
        container: Node,
        start_node: &Node,
        end_node: &Node,
        has_passed_go: &mut bool,
        nodes: &mut Vec<SplitText>,
    ) -> bool {
        let mut inside = Some(container);

        while let Some(container) = inside {
            if &container == start_node {
                *has_passed_go = true;
            }

            if *has_passed_go && container.node_type() == Node::TEXT_NODE {
                log::info!("- {:?}", container.text_content());
                // TODO: Remove clone
                nodes.push(SplitText { text: container.clone().unchecked_into(), offset: 0 });
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
pub(crate) fn return_all_text_nodes(container: &Node) -> Vec<SplitText> {
    let mut found = Vec::new();

    let mut inside = container.first_child();

    while let Some(container) = inside {
        if container.node_type() == Node::TEXT_NODE {
            found.push(SplitText { text: container.clone().unchecked_into(), offset: 0 });
        }

        found.append(&mut return_all_text_nodes(&container));

        inside = container.next_sibling();
    }

    found
}
