#![feature(globs, unboxed_closure_sugar)]
extern crate libc;

pub use options::{ReadOptions, WriteOptions};

mod ffi;
mod comparator;
pub mod database;
pub mod columnfamily;
pub mod options;

pub trait Table {

    fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<Vec<u8>>, String>;

    fn put(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<(), String>;

    fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), String>;
}
