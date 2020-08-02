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
pub mod rsim;

pub use processor::Processor;
