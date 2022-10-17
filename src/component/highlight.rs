use crate::{selection::NodeContainer, ComponentFlag, Result};

use super::Component;

pub struct Highlight;

impl Component for Highlight {
    fn title() -> &'static str {
        "H"
    }

    fn on_select(&self, nodes: NodeContainer) -> Result<()> {
        log::info!("Highlight");

        nodes.toggle_selection(ComponentFlag::HIGHLIGHT)?;

        Ok(())
    }
}
