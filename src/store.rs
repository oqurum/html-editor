use serde::{Deserialize, Serialize};
use web_sys::{HtmlElement, Text};

use crate::{
    component::{ComponentDataStore, FlagsWithData, SingleFlagWithData},
    listener::{register_with_data, ListenerData, ListenerEvent, ListenerHandle, MouseListener},
    migration::CURRENT_VERSION,
    text::return_all_text_nodes,
    ListenerId, Result, WrappedText,
};

pub fn load_and_register(
    container: HtmlElement,
    state: SaveState,
    listener: MouseListener,
    on_event: Option<ListenerEvent>,
) -> Result<ListenerHandle> {
    let nodes = return_all_text_nodes(&container);

    register_with_data(
        container,
        state.into_listener_data(nodes)?,
        listener,
        on_event,
    )
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
    /// Creates the listener tree
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
                    list_node.set_flag_for_node(
                        &curr_node,
                        FlagsWithData::from_singles(&text_split.flags),
                    )?;
                } else {
                    let node = list_node.split_node(&curr_node, text_split.offset - text_offset)?;
                    list_node
                        .set_flag_for_node(&node, FlagsWithData::from_singles(&text_split.flags))?;
                    curr_node = node;
                }

                // If it contains length (all of them should unless its' selecting until end of Node)
                // We'll split it, set tag empty, and increase offset since out current node will now be the split node.
                if let Some(length) = text_split.length {
                    curr_node = list_node.split_node(&curr_node, length)?;
                    list_node.set_flag_for_node(&curr_node, FlagsWithData::empty())?;

                    text_offset += length;
                }

                text_offset += text_split.offset;
            }

            // TODO: Determine why I was utilizing this.
            // list_node.add_flag_to_node(
            //     &curr_node,
            //     FlagsWithData::from_singles(&saved_node.flags[0].flags),
            // )?;
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
            flags: {
                let mut flags = Vec::new();

                for i in 0..components.len() {
                    let comp = &components[i];

                    if !comp.are_flags_empty() {
                        flags.push(SavedNodeFlag {
                            offset: comp.offset,
                            length: components.get(i + 1).map(|v| v.offset - comp.offset),
                            flags: comp.flag.into_singles_vec(),
                        })
                    }
                }

                flags
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedNodeFlag {
    offset: u32,
    length: Option<u32>,
    // TODO: Should I change to vec with SingleFlagWithData?
    flags: Vec<SingleFlagWithData>,
}
