use bitflags::bitflags;

mod highlight;
mod underline;

pub use highlight::*;
pub use underline::*;

use crate::{selection::NodeContainer, Result};

pub static STYLING_PREFIX_CLASS: &str = "editor-styling";

pub trait Component {
    const TITLE: &'static str;
    const FLAG: ComponentFlag;

    fn on_select(&self, nodes: NodeContainer) -> Result<()>;

    fn does_selected_contain_self(nodes: &NodeContainer) -> bool {
        nodes.does_selected_contain_any(Self::FLAG)
    }
}

bitflags! {
    pub struct ComponentFlag: u32 {
        const ANCHOR = 0b00000001;
        const HIGHLIGHT = 0b00000010;
        const UNDERLINE = 0b00000100;
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

        if self.contains(Self::UNDERLINE) {
            classes.push("underline");
        }

        classes.join(" ")
    }
}
