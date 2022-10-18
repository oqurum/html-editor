use gloo_utils::window;
use wasm_bindgen::JsCast;
use web_sys::{Node, Range, Selection, Text};

use crate::{ComponentFlag, Result, SharedListenerData};

pub struct NodeContainer<'a> {
    data: &'a SharedListenerData,

    nodes: Vec<Text>,

    start_offset: u32,
    end_offset: u32,
}

impl<'a> NodeContainer<'a> {
    pub fn does_selected_contain_flags(&self, flag: ComponentFlag) -> bool {
        let page_data = self.data.borrow();

        self.nodes.iter().all(|node| {
            page_data
                .get_component_node(node)
                .filter(|v| v.has_flag(flag))
                .is_some()
        })
    }

    pub fn toggle_selection(mut self, flag: ComponentFlag) -> Result<()> {
        self.acquire_selected_nodes()?;

        if self.does_selected_contain_flags(flag) {
            log::debug!("unset component");

            let mut page_data = self.data.borrow_mut();

            let mut nodes = Vec::new();

            for text in self.nodes.drain(..) {
                if let Some(node) = page_data.remove_component_node_flag(&text, flag)? {
                    nodes.push(node);
                }
            }

            self.nodes = nodes;
        } else {
            log::debug!("set component");

            let mut page_data = self.data.borrow_mut();

            for text in self.nodes.clone() {
                page_data.insert_or_update_component(text, flag)?;
            }
        }

        self.reload_selection()?;

        Ok(())
    }

    fn reload_selection(&self) -> Result<()> {
        let selection = window().get_selection()?.unwrap();

        selection.remove_all_ranges()?;

        let range = Range::new()?;

        if self.nodes.len() == 1 {
            log::debug!("Range::select_node");

            let start = &self.nodes[0];

            // range.select_node_contents(start)?;

            range.set_start(start, 0)?;
            range.set_end(start, start.length())?;
        } else if self.nodes.len() > 1 {
            log::debug!("Range::set_start");

            let start = &self.nodes[0];
            let end = &self.nodes[self.nodes.len() - 1];

            range.set_start(start, 0)?;
            range.set_end(end, end.length())?;
        }

        selection.add_range(&range)?;

        Ok(())
    }

    // TODO: Range::end_offset() can equal 0. If it does that means we should NOT push the Node to self.nodes.

    fn acquire_selected_nodes(&mut self) -> Result<()> {
        // We've selected inside a single node.
        if self.nodes.is_empty() {
            return Ok(());
        } else if self.nodes.len() == 1 {
            let node = self.nodes.remove(0);

            // TODO: Determine if we should remove white-space from the end of a text node.

            let mut page_data = self.data.borrow_mut();

            // TODO: Remove cloned. Quick mutable & immutable borrow error fix
            // If component node already exists.
            if let Some(comp_node) = page_data.get_component_node(&node).cloned() {
                log::info!("Node cached");

                if self.end_offset != node.length() {
                    log::info!(" - splitting end: {} != {}", self.end_offset, node.length());
                    let next_node = comp_node.split(self.end_offset)?;
                    page_data.nodes.push(next_node);
                }

                if self.start_offset != 0 {
                    log::info!(" - splitting start: {}", self.start_offset);
                    let right_text = comp_node.split(self.start_offset)?;

                    self.nodes.push(right_text.node.clone());
                    page_data.nodes.push(right_text);
                } else {
                    self.nodes.push(node);
                }
            } else {
                // If we didn't select the full Text Node.
                if self.end_offset != node.length() {
                    // We don't need the Text after the split
                    node.split_text(self.end_offset)?;
                }

                if self.start_offset != 0 {
                    self.nodes.push(node.split_text(self.start_offset)?);

                    // TODO: prefix Node Text
                } else {
                    self.nodes.push(node);
                }
            }
        } else {
            if self.start_offset != 0 {
                //
            }
        }

        Ok(())
    }
}

pub fn get_nodes_in_selection(
    selection: Selection,
    data: &'_ SharedListenerData,
) -> Result<NodeContainer<'_>> {
    let range = selection.get_range_at(0)?;

    let start_node = range.start_container()?;
    let end_node = range.end_container()?;

    // Container which contains the start node -> end node
    let container = range.common_ancestor_container()?;

    let nodes = get_all_text_nodes_in_container(container, &start_node, &end_node);

    Ok(NodeContainer {
        data,
        nodes,

        start_offset: range.start_offset()?,
        end_offset: range.end_offset()?,
    })
}

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

fn return_all_text_nodes(container: &Node) -> Vec<Text> {
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
