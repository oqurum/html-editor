use crate::{ComponentFlag, Result};

use super::{Component, Context};

pub struct Underline;

impl Component for Underline {
    const FLAG: ComponentFlag = ComponentFlag::UNDERLINE;
    const TITLE: &'static str = "U";

    type Data = ();

    fn on_select(&self, ctx: &Context) -> Result<()> {
        log::debug!("Underline");

        ctx.nodes.borrow_mut().toggle_selection::<Self>()?;

        Ok(())
    }
}
