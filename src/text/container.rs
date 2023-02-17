use web_sys::Text;

use crate::{component::FlagsWithData, ComponentFlag, Result, WrappedText};

/// Contains the Text Node we can split apart into smaller ones.
///
/// We use this struct to better show that if there are multiple items in the vec that means we have split it apart.
#[derive(Debug, Clone)]
pub struct TextContainer {
    /// The non-split Text `Node` or split `Node`s
    pub(crate) text: Vec<WrappedText>,
}

impl TextContainer {
    pub fn new(text: Text) -> Result<Self> {
        Ok(Self {
            text: vec![WrappedText::wrap(text, 0, FlagsWithData::empty())?],
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
        self.text.iter().flat_map(|v| v.flag.data.clone()).collect()
    }

    pub fn has_flag(&self, flag: &FlagsWithData) -> bool {
        self.text.iter().any(|v| v.has_flag(flag))
    }

    pub fn intersects_flag(&self, value: ComponentFlag) -> bool {
        self.text.iter().any(|v| v.intersects_flag(value))
    }

    pub fn are_all_flags_empty(&self) -> bool {
        self.text.iter().all(|v| v.are_flags_empty())
    }

    pub fn contains_node(&self, node: &Text) -> bool {
        self.text.iter().any(|v| &v.node == node)
    }

    pub fn get_wrapped_text(&self, node: &Text) -> Option<&WrappedText> {
        self.text.iter().find(|v| &v.node == node)
    }

    pub fn get_wrapped_text_mut(&mut self, node: &Text) -> Option<&mut WrappedText> {
        self.text.iter_mut().find(|v| &v.node == node)
    }

    pub fn find_node_return_mut_ref(&mut self, node: &Text) -> Option<FoundWrappedTextRefMut<'_>> {
        let node_index = self.text.iter().position(|v| &v.node == node)?;

        Some(FoundWrappedTextRefMut {
            container: self,
            node_index,
        })
    }

    pub fn add_flag_to_node(&mut self, node: &Text, flag: FlagsWithData) -> Result<()> {
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

    pub fn set_flag_for_node(&mut self, node: &Text, flag: FlagsWithData) -> Result<()> {
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

    pub fn remove_flag_from_node(&mut self, node: &Text, flag: &FlagsWithData) -> Result<()> {
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

impl Drop for TextContainer {
    fn drop(&mut self) {
        while let Some((index, comp)) = self
            .text
            .iter_mut()
            .enumerate()
            .find(|(_, v)| !v.are_flags_empty())
        {
            let _ = comp.remove_all_flag().map_err(|e| log::error!("{e:?}"));

            let _ = try_join_component_into_surroundings(index, &mut self.text)
                .map_err(|e| log::error!("{e:?}"));
        }
    }
}

/// Simple struct for finding a `WrappedText` while also allowing for container access if needed.
pub struct FoundWrappedTextRefMut<'a> {
    container: &'a mut TextContainer,
    node_index: usize,
}

impl<'a> FoundWrappedTextRefMut<'a> {
    pub fn get_text_mut(&mut self) -> &mut WrappedText {
        &mut self.container.text[self.node_index]
    }

    pub fn add_flag_to(&mut self, flag: FlagsWithData) -> Result<()> {
        self.get_text_mut().add_flag(flag)
    }

    pub fn set_flag_for(&mut self, flag: FlagsWithData) -> Result<()> {
        self.get_text_mut().set_flag(flag)
    }

    pub fn remove_flag_from(&mut self, flag: &FlagsWithData) -> Result<()> {
        self.get_text_mut().remove_flag(flag)
    }

    pub fn empty_flags_from(&mut self) -> Result<()> {
        self.get_text_mut().remove_all_flag()
    }

    pub fn rejoin_into_surrounding(&mut self) -> Result<()> {
        try_join_component_into_surroundings(self.node_index, &mut self.container.text)
    }
}

impl<'a> std::ops::Deref for FoundWrappedTextRefMut<'a> {
    type Target = TextContainer;

    fn deref(&self) -> &Self::Target {
        &*self.container
    }
}

impl<'a> std::ops::DerefMut for FoundWrappedTextRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.container
    }
}

/// Find and join Component Nodes' of the same type.
///
/// Updates the
fn try_join_component_into_surroundings(
    mut index: usize,
    nodes: &mut Vec<WrappedText>,
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
