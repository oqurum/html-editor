use std::borrow::Cow;

use bitflags::bitflags;
use num_enum::TryFromPrimitive;
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

    type Data: ComponentData;

    fn on_select(&self, nodes: NodeContainer) -> Result<()>;

    fn on_held(&self) {}

    fn does_selected_contain_self(nodes: &NodeContainer) -> bool {
        nodes.does_selected_contain_any(&FlagsWithData::new_flag(Self::FLAG))
    }

    fn get_default_data_id() -> u8 {
        <Self::Data as ComponentData>::default().map(|v| v.id()).unwrap_or_default()
    }
}

pub trait ComponentData where Self: Sized {
    fn get_css(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn id(&self) -> u8;
    fn from_id(value: u8) -> Self;

    fn default() -> Option<Self> {
        None
    }
}

impl ComponentData for () {
    fn id(&self) -> u8 {
        0
    }

    fn from_id(_value: u8) -> Self {}
}


bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ComponentFlag: u32 {
        const ITALICIZE = 0b0000_0001;
        const HIGHLIGHT = 0b0000_0010;
        const UNDERLINE = 0b0000_0100;
    }
}

static COMPONENT_FLAGS: [ComponentFlag; 3] = [ ComponentFlag::ITALICIZE, ComponentFlag::HIGHLIGHT, ComponentFlag::UNDERLINE ];

impl ComponentFlag {
    // TODO: Replace with get_data() but we're using Self: Sized for the trait
    pub fn get_data_class(self, value: u8) -> Option<Cow<'static, str>> {
        match self {
            Self::HIGHLIGHT => {
                HighlightTypes::try_from_primitive(value).unwrap().get_css()
            }

            _ => None,
        }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagsWithData {
    pub flag: ComponentFlag,
    /// Singular Flag with data.
    pub data: Vec<(ComponentFlag, u8)>,
}

impl FlagsWithData {
    pub fn new_flag(flag: ComponentFlag) -> Self {
        Self {
            flag,
            data: Vec::new(),
        }
    }

    pub fn new_with_data(flag: ComponentFlag, data: u8) -> Self {
        Self {
            flag,
            data: vec![ (flag, data) ],
        }
    }

    pub fn empty() -> Self {
        Self {
            flag: ComponentFlag::empty(),
            data: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.flag.is_empty()
    }

    // TODO: Rename to Contains Flag
    pub fn contains(&self, value: &Self) -> bool {
        if self.flag.contains(value.flag) {
            for a in &value.data {
                if !self.data.iter().any(|b| b == a) {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn insert(&mut self, value: Self) {
        self.flag.insert(value.flag);

        for &(flag, new_data) in &value.data {
            if let Some(data) = self.data.iter_mut().find(|v| v.0 == flag) {
                data.1 = new_data;
            } else {
                self.data.push((flag, new_data));
            }
        }

        self.data.sort_unstable();
    }

    pub fn remove(&mut self, value: &Self) {
        self.flag.remove(value.flag);

        for &(flag, _) in &value.data {
            if let Some(idx) = self.data.iter().position(|v| v.0 == flag) {
                self.data.remove(idx);
            }
        }

        self.data.sort_unstable();
    }

    pub fn generate_class_name(&self) -> String {
        let mut classes = self.flag.into_class_names();

        for (flag, data) in &self.data {
            if let Some(css) = flag.get_data_class(*data) {
                classes += " ";
                classes += &css;
            }
        }

        classes
    }

    pub fn into_singles_vec(&self) -> Vec<SingleFlagWithData> {
        COMPONENT_FLAGS.into_iter()
        .filter_map(|v| {
            if self.flag.contains(v) {
                let data = self.data.iter()
                    .find_map(|(flag, data)| {
                        if &v == flag {
                            Some(*data)
                        } else {
                            None
                        }
                    }).unwrap_or_default();

                Some(SingleFlagWithData::new(v, data))
            } else {
                None
            }
        })
        .collect()
    }

    pub fn from_singles(value: &[SingleFlagWithData]) -> Self {
        let mut this = Self::empty();

        for item in value {
            this.flag.insert(item.flag());
            this.data.push((item.flag(), item.data()));
        }

        this.data.sort_unstable();

        this
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SingleFlagWithData(u64);

impl SingleFlagWithData {
    pub fn new(flag: ComponentFlag, data: u8) -> Self {
        Self((flag.bits() as u64) << 32 | (data as u64))
    }

    pub fn empty() -> Self {
        Self(0)
    }


    pub fn flag(self) -> ComponentFlag {
        ComponentFlag::from_bits_truncate((self.0 >> 32) as u32)
    }

    pub fn data(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}