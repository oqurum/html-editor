use crate::{selection::NodeContainer, ComponentFlag, Result};

use super::Component;

pub struct Italicize;

impl Component for Italicize {
    const FLAG: ComponentFlag = ComponentFlag::ITALICIZE;
    const TITLE: &'static str = "I";

    type Data = ();

    fn on_select(&self, nodes: NodeContainer) -> Result<()> {
        log::debug!("Italicize");

        nodes.toggle_selection::<Self>()?;

        Ok(())
    }
}
