use crate::{ComponentFlag, Result};

use super::{Component, Context};

pub struct Underline;

impl Component for Underline {
    const FLAG: ComponentFlag = ComponentFlag::UNDERLINE;
    const TITLE: &'static str = "Underline";

    type Data = ();

    fn on_click_button(&self, ctx: &Context<Self>) -> Result<()> {
        debug!("Underline");

        ctx.nodes.borrow_mut().toggle_selection::<Self>()?;

        Ok(())
    }
}
