use bitflags::bitflags;
use serde::{Deserialize, Serialize};

mod highlight;
mod italicize;
mod underline;

pub use highlight::*;
pub use italicize::*;
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
    #[derive(Serialize, Deserialize)]
    pub struct ComponentFlag: u32 {
        const ITALICIZE = 0b0000_0001;
        const HIGHLIGHT = 0b0000_0010;
        const UNDERLINE = 0b0000_0100;
    }
}

impl ComponentFlag {
    pub fn into_class_names(self) -> String {
        let mut classes = Vec::new();

        if self.contains(Self::ITALICIZE) {
            classes.push("italicize");
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
