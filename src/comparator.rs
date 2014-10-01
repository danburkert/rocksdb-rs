use libc::{c_void, size_t};
use std::c_str::CString;
use std::mem;
use std::slice;

use ffi::{rocksdb_comparator_create, rocksdb_comparator_t};

struct Comparator<'a> {
    name: CString,
    compare: |&[u8], &[u8]|: Sync + 'a -> Ordering
}

pub fn create<'a>(name: &str,
                  compare: |&[u8], &[u8]|: Sync + Send -> Ordering)
                  -> *mut rocksdb_comparator_t {
    let comparator = box Comparator { name: name.to_c_str(), compare: compare };
    unsafe {
        rocksdb_comparator_create(mem::transmute(comparator),
                                  _destructor,
                                  _compare,
                                  _name)
    }
}

/// Callback that rocksdb will execute in order to get the name of the comparator.
extern "C" fn _name(state: *mut c_void) -> *const i8 {
     let x: &Comparator = unsafe { &*(state as *mut Comparator) };
     x.name.as_ptr()
}

/// Callback that rocksdb will execute to compare keys.
extern "C" fn _compare(state: *mut c_void,
                       a: *const i8, a_len: size_t,
                       b: *const i8, b_len: size_t) -> i32 {
    unsafe {
        slice::raw::buf_as_slice(a as *const u8, a_len as uint, |a_slice| {
            slice::raw::buf_as_slice(b as *const u8, b_len as uint, |b_slice| {
                let x: &mut Comparator = &mut *(state as *mut Comparator);
                match (x.compare)(a_slice, b_slice) {
                    Less => -1,
                    Equal => 0,
                    Greater => 1
                }
            })
        })
    }
}

/// Callback that rocksdb will execute to destroy the comparator.
extern "C" fn _destructor(state: *mut c_void) {
    // Convert back to a box and let destructor reclaim
    let _: Box<Comparator> = unsafe {mem::transmute(state)};
}
