pub mod alloc;
pub mod attr;
pub mod codable;
pub mod context;
pub mod decoder;
pub mod env;
pub mod file;
pub mod filter;
pub mod fixed;
pub mod layer;
pub mod metadata;
pub mod package;
pub mod reader;
pub mod result;
pub mod slice;
pub mod token;
pub mod variant;
pub mod writer;

mod string;
mod vec;

use alloc::SharedAllocator;

#[global_allocator]
static ALLOC: SharedAllocator = SharedAllocator;
