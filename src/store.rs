use bytes::Buf;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlElement, Text};

use crate::{
    component::{ComponentDataStore, FlagsWithData, SingleFlagWithData},
    listener::{register_with_data, ListenerData, ListenerEvent, ListenerHandle, MouseListener},
    migration::CURRENT_VERSION,
    text::return_all_text_nodes,
    ComponentFlag, ListenerId, Result, WrappedText,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.version.to_be_bytes());

        // Data
        bytes.extend_from_slice(&(self.data.len() as u32).to_be_bytes());

        for node in &self.data {
            bytes.extend_from_slice(&node.0.bits().to_be_bytes());
            bytes.extend_from_slice(&(node.1.len() as u32).to_be_bytes());
            bytes.extend_from_slice(node.1.as_bytes());
        }

        // Nodes
        bytes.extend_from_slice(&(self.nodes.len() as u32).to_be_bytes());

        for node in &self.nodes {
            bytes.append(&mut node.into_bytes());
        }

        bytes
    }

    pub fn from_bytes<B: Buf>(bytes: &mut B) -> Self {
        Self {
            version: bytes.get_u64() as usize,
            data: {
                let mut array = Vec::new();

                for _ in 0..bytes.get_u32() {
                    let flag = ComponentFlag::from_bits_truncate(bytes.get_u32());

                    let str_len = bytes.get_u32();
                    let value = bytes.copy_to_bytes(str_len as usize);
                    let value = String::from_utf8(value.to_vec()).unwrap();

                    array.push(ComponentDataStore(flag, value));
                }

                array
            },
            nodes: {
                let mut array = Vec::new();

                for _ in 0..bytes.get_u32() {
                    array.push(SavedNode::from_bytes(bytes));
                }

                array
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.index.to_be_bytes());

        bytes.extend_from_slice(&(self.flags.len() as u32).to_be_bytes());

        for flags in &self.flags {
            bytes.append(&mut flags.into_bytes());
        }

        bytes
    }

    pub fn from_bytes<B: Buf>(bytes: &mut B) -> Self {
        Self {
            index: bytes.get_u64() as usize,
            flags: {
                let mut array = Vec::new();

                for _ in 0..bytes.get_u32() {
                    array.push(SavedNodeFlag::from_bytes(bytes));
                }

                array
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedNodeFlag {
    offset: u32,
    length: Option<u32>,
    // TODO: Should I change to vec with SingleFlagWithData?
    flags: Vec<SingleFlagWithData>,
}

impl SavedNodeFlag {
    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.offset.to_be_bytes());

        if let Some(len) = self.length {
            bytes.push(1);
            bytes.extend_from_slice(&len.to_be_bytes());
        } else {
            bytes.push(0);
        }

        // We should never have a length greater than 255.
        bytes.push(self.flags.len() as u8);

        for flags in &self.flags {
            bytes.extend_from_slice(&flags.0.to_be_bytes());
        }

        bytes
    }

    pub fn from_bytes<B: Buf>(bytes: &mut B) -> Self {
        Self {
            offset: bytes.get_u32(),
            length: {
                if bytes.get_u8() == 1 {
                    Some(bytes.get_u32())
                } else {
                    None
                }
            },
            flags: {
                let mut array = Vec::new();

                for _ in 0..bytes.get_u8() {
                    array.push(SingleFlagWithData(bytes.get_u64()));
                }

                array
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::ComponentFlag;

    use super::*;

    #[test]
    fn save_node_flag_to_from_bytes() {
        let save = SavedNodeFlag {
            offset: 1234,
            length: Some(5678),
            flags: vec![SingleFlagWithData::new(ComponentFlag::ITALICIZE, 11)],
        };

        let bytes = save.into_bytes();

        let save2 = SavedNodeFlag::from_bytes(&mut Bytes::from(bytes));

        assert_eq!(save, save2);
    }

    #[test]
    fn save_node_to_from_bytes() {
        let save = SavedNode {
            index: 0,
            flags: vec![SavedNodeFlag {
                offset: 1234,
                length: Some(5678),
                flags: vec![SingleFlagWithData::new(ComponentFlag::ITALICIZE, 11)],
            }],
        };

        let bytes = save.into_bytes();

        let save2 = SavedNode::from_bytes(&mut Bytes::from(bytes));

        assert_eq!(save, save2);
    }

    #[test]
    fn save_state_to_from_bytes() {
        let save = SaveState {
            version: 12345,
            data: vec![ComponentDataStore::new(ComponentFlag::ITALICIZE, &100)],
            nodes: vec![SavedNode {
                index: 0,
                flags: vec![SavedNodeFlag {
                    offset: 1234,
                    length: Some(5678),
                    flags: vec![SingleFlagWithData::new(ComponentFlag::ITALICIZE, 11)],
                }],
            }],
        };

        let bytes = save.into_bytes();

        let save2 = SaveState::from_bytes(&mut Bytes::from(bytes));

        assert_eq!(save, save2);
    }
}
