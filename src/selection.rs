use std::mem;

use gloo_utils::window;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{Range, Selection, Text};

use crate::{
    component::FlagsWithData, text::get_all_text_nodes_in_container, Component, ComponentFlag,
    Result, SharedListenerData,
};

pub struct NodeContainer {
    pub(crate) data: SharedListenerData,

    nodes: Vec<Text>,

    start_offset: u32,
    end_offset: u32,
}

impl NodeContainer {
    pub fn get_selected_data_ids(&self) -> Vec<(ComponentFlag, u32)> {
        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let page_data = page_data.borrow();

        self.nodes
            .iter()
            .filter_map(|text| {
                let cont = page_data.get_text_container_for_node(text)?;
                Some(cont.get_wrapped_text(text).unwrap().flag.data.clone())
            })
            .flatten()
            .collect()
    }

    pub fn does_selected_intersect(&self, flag: ComponentFlag) -> bool {
        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let page_data = page_data.borrow();

        self.nodes.iter().any(|text| {
            page_data
                .get_text_wrapper(text)
                .filter(|v| v.intersects_flag(flag))
                .is_some()
        })
    }

    pub fn does_selected_contain(&self, flag: &FlagsWithData) -> bool {
        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let page_data = page_data.borrow();

        self.nodes.iter().any(|text| {
            page_data
                .get_text_wrapper(text)
                .filter(|v| v.has_flag(flag))
                .is_some()
        })
    }

    pub fn insert_selection<D: Component>(&mut self, data: Option<u32>) -> Result<bool> {
        let flag =
            FlagsWithData::new_with_data(D::FLAG, data.unwrap_or_else(D::get_default_data_id));

        if self.does_selected_intersect(D::ALLOWED_SIBLINGS.complement()) {
            log::error!("Not Allowed");
            return Ok(false);
        }

        log::debug!("set component");

        self.split_and_acq_text_nodes()?;

        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let mut page_data = page_data.borrow_mut();

        for text in &self.nodes {
            page_data.update_container(text, flag.clone())?;
        }

        self.reload_selection()?;

        Ok(true)
    }

    pub fn remove_selection<D: Component>(&mut self, data: Option<u32>) -> Result<bool> {
        let flag =
            FlagsWithData::new_with_data(D::FLAG, data.unwrap_or_else(D::get_default_data_id));

        log::debug!("unset component");

        // TODO: Should I split when removing selection?
        // self.split_and_acq_text_nodes()?;

        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let mut page_data = page_data.borrow_mut();

        for text in &self.nodes {
            page_data.remove_component_node_flag(text, &flag)?;
        }

        self.reload_selection()?;

        Ok(true)
    }

    // TODO: Add Optional Data
    pub fn toggle_selection<D: Component>(&mut self) -> Result<bool> {
        let flag = FlagsWithData::new_with_data(D::FLAG, D::get_default_data_id());

        if self.does_selected_intersect(D::ALLOWED_SIBLINGS.complement()) {
            log::error!("Not Allowed");
            return Ok(false);
        }

        self.split_and_acq_text_nodes()?;

        if self.does_selected_contain(&flag) {
            log::debug!("unset component");

            let page_data = self.data.upgrade().expect_throw("data upgrade");
            let mut page_data = page_data.borrow_mut();

            for text in &self.nodes {
                page_data.remove_component_node_flag(text, &flag)?;
            }
        } else {
            log::debug!("set component");

            let page_data = self.data.upgrade().expect_throw("data upgrade");
            let mut page_data = page_data.borrow_mut();

            for text in &self.nodes {
                page_data.update_container(text, flag.clone())?;
            }
        }

        self.reload_selection()?;

        Ok(true)
    }

    pub fn reload_selection(&self) -> Result<()> {
        let selection = window().get_selection()?.unwrap();

        selection.remove_all_ranges()?;

        let range = Range::new()?;

        match self.nodes.len().cmp(&1) {
            std::cmp::Ordering::Equal => {
                log::debug!("Range::select_node");

                range.select_node_contents(&self.nodes[0])?;
            }

            std::cmp::Ordering::Greater => {
                log::debug!("Range::set_start");

                let start = &self.nodes[0];
                let end = &self.nodes[self.nodes.len() - 1];

                range.set_start(start, 0)?;
                range.set_end(end, end.length())?;
            }

            std::cmp::Ordering::Less => (),
        }

        selection.add_range(&range)?;

        Ok(())
    }

    // TODO: Range::end_offset() can equal 0. If it does that means we should NOT push the Node to self.nodes.

    fn split_and_acq_text_nodes(&mut self) -> Result<()> {
        // We've selected inside a single node.
        if self.nodes.is_empty() {
            return Ok(());
        } else if self.nodes.len() == 1 {
            let split = self.nodes.remove(0);
            self.separate_text_node(split, Some(self.start_offset), Some(self.end_offset))?;
        } else {
            let nodes = mem::take(&mut self.nodes);
            let node_count = nodes.len();

            for (i, split) in nodes.into_iter().enumerate() {
                self.separate_text_node(
                    split,
                    (i == 0).then_some(self.start_offset),
                    (i + 1 == node_count).then_some(self.end_offset),
                )?;
            }
        }

        Ok(())
    }

    fn separate_text_node(
        &mut self,
        text: Text,
        start_offset: Option<u32>,
        end_offset: Option<u32>,
    ) -> Result<()> {
        // TODO: Determine if we should remove white-space from the end of a text node.

        let page_data = self.data.upgrade().expect_throw("data upgrade");
        let mut page_data = page_data.borrow_mut();

        // If component node already exists.
        let mut comp_node = page_data.get_text_container_mut(&text).unwrap_throw();

        log::debug!("Node cached");

        if let Some(end_offset) = end_offset {
            if end_offset != text.length() {
                log::debug!(" - splitting end: {} != {}", end_offset, text.length());

                comp_node.split_node(&text, end_offset)?;
            }
        }

        if let Some(start_offset) = start_offset.filter(|v| *v != 0) {
            log::debug!(" - splitting start: {}", start_offset);

            let right_text = comp_node.split_node(&text, start_offset)?;

            self.nodes.push(right_text);
        } else {
            self.nodes.push(text);
        }

        Ok(())
    }
}

pub fn get_nodes_in_selection(
    selection: Selection,
    data: SharedListenerData,
) -> Result<NodeContainer> {
    let range = selection.get_range_at(0)?;

    let start_node = range.start_container()?;
    let end_node = range.end_container()?;

    let mut start_offset = range.start_offset()?;
    let end_offset = range.end_offset()?;

    // Container which contains the start node -> end node
    let container = range.common_ancestor_container()?;

    let mut nodes = get_all_text_nodes_in_container(container, &start_node, &end_node);

    // Check for our start_offset the same length as the first node.
    // I don't know what causes it but it does happen randomly.
    if !nodes.is_empty() && start_offset == nodes[0].length() {
        log::debug!(
            "Removing Text Node which was included though our start_offset was at the end of it"
        );
        nodes.remove(0);
        start_offset = 0;
    }

    Ok(NodeContainer {
        data,
        nodes,

        start_offset,
        end_offset,
    })
}

pub fn create_container(nodes: Vec<Text>, data: SharedListenerData) -> Result<NodeContainer> {
    Ok(NodeContainer {
        data,
        nodes,

        start_offset: 0,
        end_offset: 0,
    })
}
