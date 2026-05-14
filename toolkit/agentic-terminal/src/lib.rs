#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

pub mod agent;
pub mod app;
pub mod journal;
pub mod manifest;
pub mod navigator;
pub mod runtime;
pub mod state;
pub mod term;
pub mod types;
pub mod view;
