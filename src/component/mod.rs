use std::{borrow::Cow, rc::Rc, cell::RefCell};

use bitflags::bitflags;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod highlight;
mod italicize;
mod note;
mod underline;

pub use highlight::*;
pub use italicize::*;
pub use note::*;
pub use underline::*;

use crate::{selection::NodeContainer, Result};

pub static STYLING_PREFIX_CLASS: &str = "editor-styling";

pub trait Component {
    const TITLE: &'static str;
    const FLAG: ComponentFlag;

    type Data: ComponentData;

    fn on_click_button(&self, ctx: &Context) -> Result<()>;

    fn on_held(&self) {}

    fn does_selected_contain_self(nodes: &NodeContainer) -> bool {
        nodes.does_selected_contain_any(&FlagsWithData::new_flag(Self::FLAG))
    }

    fn get_default_data_id() -> u32 {
        <Self::Data as ComponentData>::default().map(|v| v.id()).unwrap_or_default()
    }
}

pub trait ComponentData where Self: Sized {
    fn get_css(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn id(&self) -> u32;
    fn from_id(value: u32) -> Self;

    fn default() -> Option<Self> {
        None
    }
}

impl ComponentData for () {
    fn id(&self) -> u32 {
        0
    }

    fn from_id(_value: u32) -> Self {}
}


/// `ComponentFlag` is used to determine the type of component the Store is for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDataStore(pub(crate) ComponentFlag, pub(crate) String);

impl ComponentDataStore {
    pub fn new<S: Serialize>(flag: ComponentFlag, value: &S) -> Self {
        Self(flag, serde_json::to_string(value).unwrap())
    }

    pub fn update<S: Serialize>(&mut self, value: &S) {
        self.1 = serde_json::to_string(value).unwrap();
    }

    pub fn parse<D: DeserializeOwned>(&self) -> D {
        serde_json::from_str(&self.1).unwrap()
    }
}


#[derive(Clone)]
pub struct Context {
    pub nodes: Rc<RefCell<NodeContainer>>,
}

impl Context {
    pub fn store_data<D: Component, S: Serialize>(&self, value: &S) -> u32 {
        self.nodes.borrow().data.borrow_mut().store_data(D::FLAG, value)
    }

    pub fn remove_data<D: Component>(&self, value: u32) {
        self.nodes.borrow().data.borrow_mut().remove_data(D::FLAG, value)
    }

    pub fn get_selection_data_ids(&self) -> Vec<(ComponentFlag, u32)> {
        self.nodes.borrow().get_selected_data_ids()
    }

    pub fn insert_selection<D: Component>(&self, data: Option<u32>) -> Result<()> {
        self.nodes.borrow_mut().insert_selection::<D>(data)
    }

    pub fn remove_selection<D: Component>(&self, data: Option<u32>) -> Result<()> {
        self.nodes.borrow_mut().insert_selection::<D>(data)
    }

    pub fn save(&self) {
        let id = self.nodes.borrow().data.borrow().listener_id;

        id.try_get().unwrap().borrow().on_event.borrow()(id);
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct ComponentFlag: u32 {
        const ITALICIZE = 0b0000_0001;
        const HIGHLIGHT = 0b0000_0010;
        const UNDERLINE = 0b0000_0100;
        const NOTE      = 0b0000_1000;
    }
}

static COMPONENT_FLAGS: [ComponentFlag; 4] = [ ComponentFlag::ITALICIZE, ComponentFlag::HIGHLIGHT, ComponentFlag::UNDERLINE, ComponentFlag::NOTE ];

impl ComponentFlag {
    // TODO: Replace with get_data() but we're using Self: Sized for the trait
    pub fn get_data_class(self, value: u32) -> Option<Cow<'static, str>> {
        match self {
            Self::HIGHLIGHT => {
                HighlightTypes::try_from_primitive(value as u8).unwrap().get_css()
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

        if self.contains(Self::NOTE) {
            classes.push("note");
        }

        classes.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlagsWithData {
    pub flag: ComponentFlag,
    /// Singular Flag with data.
    pub data: Vec<(ComponentFlag, u32)>,
}

impl FlagsWithData {
    pub fn new_flag(flag: ComponentFlag) -> Self {
        Self {
            flag,
            data: Vec::new(),
        }
    }

    pub fn new_with_data(flag: ComponentFlag, data: u32) -> Self {
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


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SingleFlagWithData(u64);
// TODO: Replace u64 w/ enum for Component, Data (array)

impl SingleFlagWithData {
    pub fn new(flag: ComponentFlag, data: u32) -> Self {
        Self((flag.bits() as u64) << 32 | (data as u64))
    }

    pub fn empty() -> Self {
        Self(0)
    }


    pub fn flag(self) -> ComponentFlag {
        ComponentFlag::from_bits_truncate((self.0 >> 32) as u32)
    }

    pub fn data(self) -> u32 {
        (self.0 & 0xFF) as u32
    }
}

impl std::fmt::Debug for SingleFlagWithData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SingleFlagWithData")
            .field(&self.flag())
            .field(&self.data())
            .finish()
    }
}