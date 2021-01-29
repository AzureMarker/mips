//! A MIPS R2000 simulator
//!
//! Currently supports executing RSIM modules

#[macro_use]
extern crate log;

pub mod config;
mod constants;
mod instruction;
mod math;
mod memory;
mod operations;
mod processor;
mod registers;

pub use processor::Processor;
