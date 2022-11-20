use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use web_sys::{HtmlElement, Text};

use crate::{listener::{ListenerData, ListenerHandle, register_with_data}, WrappedText, ListenerId, Result, component::{FlagsWithData, SingleFlagWithData, ComponentDataStore}, migration::CURRENT_VERSION};

pub fn load_and_register(
    container: HtmlElement,
    state: SaveState,
    on_event: Rc<RefCell<dyn Fn(ListenerId)>>,
) -> Result<ListenerHandle> {
    let nodes = crate::node::return_all_text_nodes(&container);

    register_with_data(container, state.into_listener_data(nodes)?, on_event)
}

pub fn save(state: &ListenerData) -> SaveState {
    SaveState {
        version: CURRENT_VERSION,
        data: state.data.clone(),
        nodes: state
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, v)| {
                if v.are_all_flags_empty() {
                    None
                } else {
                    Some(SavedNode::from_node(index, &v.text))
                }
            })
            .collect(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveState {
    pub version: usize,
    pub(crate) data: Vec<ComponentDataStore>,
    pub(crate) nodes: Vec<SavedNode>,
}

impl SaveState {
    pub(crate) fn into_listener_data(self, nodes: Vec<Text>) -> Result<ListenerData> {
        // ListenerId is set in the listener function.
        let mut listener = ListenerData::new(ListenerId::unset(), nodes)?;

        listener.data = self.data;

        for saved_node in self.nodes {
            let list_node = &mut listener.nodes[saved_node.index];

            let mut curr_node = list_node.text[0].node.clone();

            let mut text_offset = 0;

            for text_split in saved_node.flags.iter() {
                if text_split.offset == 0 {
                    list_node.set_flag_for_node(&curr_node, FlagsWithData::from_singles(&text_split.flags))?;
                } else {
                    let node = list_node.split_node(&curr_node, text_split.offset - text_offset)?;
                    list_node.set_flag_for_node(&node, FlagsWithData::from_singles(&text_split.flags))?;
                    curr_node = node;
                }

                text_offset += text_split.offset;
            }

            list_node.add_flag_to_node(&curr_node, FlagsWithData::from_singles(&saved_node.flags[0].flags))?;
        }

        Ok(listener)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNode {
    /// Node Index
    index: usize,

    flags: Vec<SavedNodeFlag>,
}

impl SavedNode {
    pub(crate) fn from_node(index: usize, components: &[WrappedText]) -> Self {
        Self {
            index,
            flags: components
                .iter()
                .map(|v| SavedNodeFlag {
                    offset: v.offset,
                    flags: v.flag.into_singles_vec(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNodeFlag {
    offset: u32,
    // TODO: Should I change to vec with SingleFlagWithData?
    flags: Vec<SingleFlagWithData>,
}