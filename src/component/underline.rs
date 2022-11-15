use crate::{selection::NodeContainer, ComponentFlag, Result};

use super::Component;

pub struct Underline;

impl Component for Underline {
    const FLAG: ComponentFlag = ComponentFlag::UNDERLINE;
    const TITLE: &'static str = "U";

    type Data = ();

    fn on_select(&self, nodes: NodeContainer) -> Result<()> {
        log::debug!("Underline");

        nodes.toggle_selection::<Self>()?;

        Ok(())
    }
}
