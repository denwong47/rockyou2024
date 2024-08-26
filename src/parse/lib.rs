//! Parsing module.
//!
//! This module is responsible for parsing the input data into the indices.

pub mod automatons;
pub mod character;
pub mod cli;
pub mod config;

mod index_key_path;
pub use index_key_path::*;

pub mod models;
pub mod string;

#[cfg(feature = "search")]
pub mod search;

pub mod logger;

mod _templates;
