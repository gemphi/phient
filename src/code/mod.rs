//! Code as a language of relations: Rust read by its own grammar into quest
//! rows (docs/108, docs/124). The compiler's parse is the teacher here and
//! every relation it gives is exact.
//!
//! In Phient's world, a word (w) is a seed of what it relates to, under what
//! conditions (c). In code the conditions are the grammar's.

pub mod bcpl;
pub mod extract;
pub mod files;

pub use extract::{type_name, Reader};
pub use files::{read_dir, read_files};
