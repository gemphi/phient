#![allow(dead_code)]
//! Program synthesis: searches over possible programs to find one
//! matching input-output examples. Uses phase patterns as specification.
//! Implements Ch 14.5's hybrid AI: deep learning + program synthesis.

pub mod program;
pub mod search;
pub mod heuristic;
pub mod library;
