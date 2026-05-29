//! Abstract syntax tree for a `.cv` document.
//!
//! The renderer walks this tree with `match`, so the shape here is deliberately
//! flat and explicit: a [`Document`] is a handful of optional header pieces plus
//! an ordered list of [`Section`]s.

/// A `key "value"` pair, used for `contact` and `sidebar` lines where the set of
/// keys is open-ended (email, github, linkedin, location, languages, skills, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub value: String,
}

/// A single resume entry (one job, project, degree, ...).
///
/// Every field except `bullets` is optional so partially-specified entries still
/// render. `#[derive(Default)]` lets the parser build one up field by field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    pub role: Option<String>,
    pub org: Option<String>,
    pub when: Option<String>,
    pub location: Option<String>,
    pub link: Option<String>,
    pub stack: Option<String>,
    pub bullets: Vec<String>,
}

/// The body of a [`Section`]: either a list of entries, or a flat skills tag list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionBody {
    Entries(Vec<Entry>),
    Tags(String),
}

/// A titled section (`section "Experience": ...`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub title: String,
    pub body: SectionBody,
}

/// A complete parsed CV document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub name: Option<String>,
    pub contact: Vec<Field>,
    pub summary: Vec<String>,
    pub sidebar: Vec<Field>,
    pub sections: Vec<Section>,
}
