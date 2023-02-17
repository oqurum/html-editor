use wasm_bindgen::JsCast;
use web_sys::{Node, Text};

mod container;
mod wrapper;

pub use container::*;
pub use wrapper::*;

use crate::ComponentFlag;

pub struct TextContentWithFlag {
    pub flag: ComponentFlag,
    pub first_text_node: Text,
    pub content: String,
}

/// Returns all text nodes in the `Node`.
pub fn return_all_text_nodes(container: &Node) -> Vec<Text> {
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

/// Return all text `Node`s which are between the start_node and the end_node parameters.
pub fn get_all_text_nodes_in_container(
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
                // log::info!("- {:?}", container.text_content());
                // TODO: Remove clone
                nodes.push(container.clone().unchecked_into());
            }

            if &container == end_node {
                // log::info!("end");
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
