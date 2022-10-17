mod highlight;

use bitflags::bitflags;
pub use highlight::*;

use crate::{selection::NodeContainer, Result};

pub static STYLING_PREFIX_CLASS: &str = "editor-styling";

pub trait Component {
    fn title() -> &'static str;

    fn on_select(&self, nodes: NodeContainer) -> Result<()>;
}

bitflags! {
    pub struct ComponentFlag: u32 {
        const ANCHOR = 0b00000001;
        const HIGHLIGHT = 0b00000010;
        const UNDERSCORE = 0b00000100;
    }
}

impl ComponentFlag {
    pub fn into_class_names(self) -> String {
        let mut classes = Vec::new();

        if self.contains(Self::ANCHOR) {
            classes.push("anchor");
        }

        if self.contains(Self::HIGHLIGHT) {
            classes.push("highlight");
        }

        if self.contains(Self::UNDERSCORE) {
            classes.push("underscore");
        }

        classes.join(" ")
    }
}
