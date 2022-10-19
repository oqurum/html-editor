use crate::{selection::NodeContainer, ComponentFlag, Result};

use super::Component;

pub struct Highlight;

impl Component for Highlight {
    const FLAG: ComponentFlag = ComponentFlag::HIGHLIGHT;
    const TITLE: &'static str = "H";

    fn on_select(&self, nodes: NodeContainer) -> Result<()> {
        log::debug!("Highlight");

        nodes.toggle_selection(Self::FLAG)?;

        Ok(())
    }
}
