use crate::{ComponentFlag, Result};

use super::{Component, Context};

pub struct Italicize;

impl Component for Italicize {
    const FLAG: ComponentFlag = ComponentFlag::ITALICIZE;
    const TITLE: &'static str = "Italicize";

    type Data = ();

    fn on_click_button(&self, ctx: &Context) -> Result<()> {
        log::debug!("Italicize");

        ctx.nodes.borrow_mut().toggle_selection::<Self>()?;

        Ok(())
    }
}
