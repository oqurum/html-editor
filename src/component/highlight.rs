use std::borrow::Cow;

use num_enum::{TryFromPrimitive, IntoPrimitive};

use crate::{ComponentFlag, Result};

use super::{Component, ComponentData, Context};

pub struct Highlight;

impl Component for Highlight {
    const FLAG: ComponentFlag = ComponentFlag::HIGHLIGHT;
    const TITLE: &'static str = "Highlight";

    type Data = HighlightTypes;

    fn on_click_button(&self, ctx: &Context) -> Result<()> {
        log::debug!("Highlight");

        ctx.nodes.borrow_mut().toggle_selection::<Self>()?;

        Ok(())
    }
}

#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum HighlightTypes {
    Yellow,
    Orange,
    Blue,
    Purple,
}

impl HighlightTypes {
    pub fn css(self) -> &'static str {
        match self {
            Self::Yellow => "yellow",
            Self::Orange => "orange",
            Self::Blue => "blue",
            Self::Purple => "purple",
        }
    }
}

impl ComponentData for HighlightTypes {
    fn get_css(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(self.css()))
    }

    fn default() -> Option<Self> {
        Some(Self::Yellow)
    }

    fn id(&self) -> u32 {
        *self as u32
    }

    fn from_id(value: u32) -> Self {
        HighlightTypes::try_from_primitive(value as u8).unwrap()
    }
}