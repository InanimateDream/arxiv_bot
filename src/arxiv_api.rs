use super::prelude::*;

pub use index::Index;
pub use metadata::MetaData;

pub use subject::Subject;
// pub type Subject = String; // TODO: Type Rich Enum

mod index;
mod subject;
mod metadata;
pub mod fetch;
