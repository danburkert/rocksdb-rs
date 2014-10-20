#![feature(phase, globs, unsafe_destructor)]
#[phase(plugin, link)] extern crate log;
extern crate libc;

use libc::c_void;
use std::c_str::CString;
use std::c_vec::CVec;
use std::collections::HashMap;
use std::kinds::marker;
use std::path::posix::Path;
use std::{io, mem, ptr, raw, slice, vec};

use ffi::*;

#[cfg(test)]
mod tests;
mod ffi;
pub mod merge_operators;

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Database
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Database {
    database: *mut rocksdb_t,
    column_families: HashMap<String, ColumnFamily>,
    _column_family_options: Vec<ColumnFamilyOptions>
}

impl Drop for Database {
    fn drop(&mut self) {
        self.column_families.clear();
        debug!("Database::drop");
        unsafe { rocksdb_close(self.database); }
    }
}

impl Database {

    /// Create a RocksDB database at the provided path with the specified column families.
    pub fn create(path: &Path,
                  db_options: DatabaseOptions,
                  column_families: HashMap<String, ColumnFamilyOptions>)
                  -> Result<Database, String> {
        unsafe {
            let raw_db_opts = db_options.options_mut();
            rocksdb_options_set_error_if_exists(raw_db_opts, 1);
            rocksdb_options_set_create_if_missing(raw_db_opts, 1);
            rocksdb_options_set_create_missing_column_families(raw_db_opts, 1);
        }
        Database::create_or_open(path, db_options, column_families)
    }

    /// Create a RocksDB database at the provided path with the specified column families.
    pub fn open(path: &Path,
                db_options: DatabaseOptions,
                cf_options: HashMap<String, ColumnFamilyOptions>)
                -> Result<Database, String> {
        unsafe {
            let raw_db_opts = db_options.options_mut();
            rocksdb_options_set_error_if_exists(raw_db_opts, 0);
            rocksdb_options_set_create_if_missing(raw_db_opts, 0);
        }
        Database::create_or_open(path, db_options, cf_options)
    }

    /// Create or open a RocksDB database at the provided path with the specified column families.
    fn create_or_open(path: &Path,
                      db_options: DatabaseOptions,
                      column_family_options: HashMap<String, ColumnFamilyOptions>)
                      -> Result<Database, String> {
        let num_cfs = column_family_options.len();
        let (cf_names, cf_options) = vec::unzip(column_family_options.into_iter());

        // Translate the column family names to a vec of c string pointers.
        let cf_c_names: Vec<CString> = cf_names.iter()
                                               .map(|cf_name| cf_name.to_c_str())
                                               .collect();
        let cf_c_name_ptrs = cf_c_names.iter()
                                       .map(|cf_c_name| cf_c_name.as_ptr())
                                       .collect::<Vec<_>>();
        let cf_option_ptrs = cf_options.iter()
                                       .map(|option| option.options())
                                       .collect::<Vec<_>>();
        let cf_ptrs: *mut *mut rocksdb_column_family_handle_t = &mut ptr::null_mut();
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            let database = rocksdb_open_column_families(db_options.options(),
                                                        path.to_c_str().as_ptr(),
                                                        num_cfs as i32,
                                                        cf_c_name_ptrs.as_ptr(),
                                                        cf_option_ptrs.as_ptr(),
                                                        cf_ptrs,
                                                        &mut error);
            if error == ptr::null_mut() {
                let column_families: HashMap<String, ColumnFamily> =
                    cf_names.into_iter()
                            .enumerate()
                            .map(|(i, cf_name)|
                                 (cf_name,
                                  ColumnFamily { database: database,
                                                 column_family: *cf_ptrs.offset(i as int) }))
                            .collect();
                Ok(Database { database: database,
                              column_families: column_families,
                              _column_family_options: cf_options })
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }
    }

    pub fn get_column_family<'a>(&'a self, column_family: &str) -> Option<&'a ColumnFamily> {
        self.column_families.find_equiv(column_family)
    }

    pub fn get_column_families(&self) -> &HashMap<String, ColumnFamily> {
        &self.column_families
    }

    pub fn write(&self, options: &WriteOptions, write_batch: WriteBatch) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_write(self.database,
                          options.options(),
                          write_batch.write_batch,
                          (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }

    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Column Family
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ColumnFamily {
    database: *mut rocksdb_t,
    column_family: *mut rocksdb_column_family_handle_t,
}

impl Drop for ColumnFamily {
    fn drop(&mut self) {
        debug!("ColumnFamily::drop");
        unsafe { rocksdb_column_family_handle_destroy(self.column_family) }
    }
}

impl ColumnFamily {

    pub fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<CVec<u8>>, String> {
        let mut error: *mut i8 = ptr::null_mut();
        let mut val_len: u64 = 0;
        unsafe {
            let val = rocksdb_get_cf(self.database,
                                     options.options(),
                                     self.column_family,
                                     key.as_ptr() as *const i8, key.len() as u64,
                                     (&mut val_len) as *mut u64,
                                     (&mut error) as *mut *mut i8);

            if error == ptr::null_mut() {
                if val == ptr::null_mut() {
                    Ok(None)
                } else {
                    let vec = CVec::new_with_dtor(val as *mut u8,
                                                  val_len as uint,
                                                  proc() { libc::free(val as *mut libc::c_void) });
                    Ok(Some(vec))
                }
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }
    }

    pub fn iter(&self, options: &ReadOptions) -> Result<KeyValues, String> {
        let itr = unsafe {
            rocksdb_create_iterator_cf(self.database, options.options(), self.column_family)
        };
        Ok(KeyValues::new(itr))
    }

    pub fn put(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_put_cf(self.database,
                           options.options(),
                           self.column_family,
                           key.as_ptr() as *const i8, key.len() as u64,
                           val.as_ptr() as *const i8, val.len() as u64,
                           (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }
    }

    pub fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_delete_cf(self.database,
                              options.options(),
                              self.column_family,
                              key.as_ptr() as *const i8, key.len() as u64,
                              (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }
    }

    pub fn merge(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_merge_cf(self.database,
                             options.options(),
                             self.column_family,
                             key.as_ptr() as *const i8, key.len() as u64,
                             val.as_ptr() as *const i8, val.len() as u64,
                             (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(CString::new(error as *const i8, true).to_string())
            }
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Comparator
///////////////////////////////////////////////////////////////////////////////////////////////////

struct ComparatorState {
    name: CString,
    compare: |&[u8], &[u8]|: Sync + Send -> Ordering
}

struct Comparator {
    comparator: *mut rocksdb_comparator_t
}

impl Drop for Comparator {
    fn drop(&mut self) {
        debug!("Comparator::drop");
        unsafe { rocksdb_comparator_destroy(self.comparator) }
    }
}

impl Comparator {
    fn new(name: &str, compare: |&[u8], &[u8]|: Sync + Send -> Ordering) -> Comparator {
        let state = box ComparatorState { name: name.to_c_str(), compare: compare };
        let comparator = unsafe {
            rocksdb_comparator_create(mem::transmute(state),
                                      comparator_destructor_callback,
                                      compare_callback,
                                      comparator_name_callback)
        };
        Comparator { comparator: comparator }
    }
}

/// Callback that rocksdb will execute in order to get the name of the comparator.
extern fn comparator_name_callback(state: *mut c_void) -> *const i8 {
     let state: &ComparatorState = unsafe { &*(state as *mut ComparatorState) };
     state.name.as_ptr()
}

/// Callback that rocksdb will execute to compare keys.
extern fn compare_callback(state: *mut c_void,
                           a: *const i8, a_len: u64,
                           b: *const i8, b_len: u64) -> i32 {
    unsafe {
        slice::raw::buf_as_slice(a as *const u8, a_len as uint, |a_slice| {
            slice::raw::buf_as_slice(b as *const u8, b_len as uint, |b_slice| {
                let x: &mut ComparatorState = &mut *(state as *mut ComparatorState);
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
extern fn comparator_destructor_callback(state: *mut c_void) {
    // Convert back to a box and let destructor reclaim
    let _: Box<ComparatorState> = unsafe {mem::transmute(state)};
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Merge Operator
///////////////////////////////////////////////////////////////////////////////////////////////////

/// The Merge Operator
///
/// Essentially, a MergeOperator specifies the semantics of a merge, which only client knows.
/// It could be numeric addition, list append, string concatenation, edit data structure, ...,
/// anything.  The library, on the other hand, is concerned with the exercise of this interface, at
/// the right time (during get, iteration, compaction...).
pub trait MergeOperator : Sync + Send {

    /// Gives the client a way to express single-key read -> modify -> write semantics.
    ///
    /// * key: The key that's associated with this merge operation. Client could multiplex the merge
    /// operator based on it if the key space is partitioned and different subspaces refer to
    /// different types of data which have different merge operation semantics.
    /// * existing_val: The value existing at the key prior to executing this merge.
    /// * operands: The sequence of merge operations to apply, front first.
    ///
    /// All values passed in will be client-specific values. So if this method returns false, it is
    /// because client specified bad data or there was internal corruption. This will be treated as
    /// an error by the library.
    fn full_merge(&self,
                  key: &[u8],
                  existing_val: Option<&[u8]>,
                  operands: Operands)
                  -> io::IoResult<Vec<u8>>;

    /// This function performs merge when all the operands are themselves merge operation types that
    /// you would have passed to a ColumnFamily::merge call in the same order (front first).
    /// (i.e. `ColumnFamily::merge(key, operands[0])`, followed by
    /// `ColumnFamily::merge(key, operands[1])`, `...`)
    ///
    /// `partial_merge` should combine the operands into a single merge operation. The returned
    /// operand should be constructed such that a call to `ColumnFamily::Merge(key, new_operand)`
    /// would yield the same result as individual calls to `ColumnFamily::Merge(key, operand)` for
    /// each operand in `operands` from front to back.
    ///
    /// `partial_merge` will be called only when the list of operands are long enough. The minimum
    /// number of operands that will be passed to the function is specified by the
    /// `ColumnFamilyOptions::min_partial_merge_operands` option.
    fn partial_merge(&self,
                     key: &[u8],
                     operands: Operands)
                     -> io::IoResult<Vec<u8>>;
}

/// The simpler, associative merge operator.
pub trait AssociativeMergeOperator: Sync + Send {
    fn merge(&self, key: &[u8], existing_val: Vec<u8>, operand: &[u8]) -> io::IoResult<Vec<u8>>;
}

impl<T: AssociativeMergeOperator> MergeOperator for T {
    fn full_merge(&self,
                  key: &[u8],
                  existing_val: Option<&[u8]>,
                  mut operands: Operands)
                  -> io::IoResult<Vec<u8>> {
        // base should never be Err, since operands always contains at least 1 element
        let base: io::IoResult<Vec<u8>> =
            existing_val.map(|val| val.to_vec())
                        .or_else(|| operands.next().map(|val| val.to_vec()))
                        .ok_or(io::standard_error(io::OtherIoError));

        operands.fold(base, |existing, operand| {
            existing.and_then(|existing| {
                self.merge(key, existing, operand)
            })
        })
    }

    fn partial_merge(&self,
                     key: &[u8],
                     mut operands: Operands)
                     -> io::IoResult<Vec<u8>> {
        // base should never be Err, since operands always contains at least 1 element
        let base: io::IoResult<Vec<u8>> = operands.next()
                                                  .map(|val| val.to_vec())
                                                  .ok_or(io::standard_error(io::OtherIoError));
        operands.fold(base, |existing, operand| {
            existing.and_then(|existing| {
                self.merge(key, existing, operand)
            })
        })
    }
}

pub struct Operands<'a> {
    operands: slice::Items<'a, *const u8>,
    lens: slice::Items<'a, u64>,
    marker: marker::ContravariantLifetime<'a>
}

impl<'a> Operands<'a> {

    fn new(operands: *const *const u8,
               operand_lens: *const u64,
               num_operands: uint)
               -> Operands<'a> {
        unsafe {
            slice::raw::buf_as_slice(operands, num_operands, |operands| {
                slice::raw::buf_as_slice(operand_lens, num_operands, |operand_lens| {
                    // Transumutes are necessary for lifetime params
                    Operands { operands: mem::transmute(operands.iter()),
                               lens: mem::transmute(operand_lens.iter()),
                               marker: marker::ContravariantLifetime::<'a> }
                })
            })
        }
    }
}

impl<'a> Iterator<&'a [u8]> for Operands<'a> {

    fn next(&mut self) -> Option<&'a [u8]> {
        match (self.operands.next(), self.lens.next()) {
            (Some(operand), Some(len)) =>
                unsafe { Some(mem::transmute(raw::Slice { data: *operand, len: *len as uint })) },
            _ => None
        }
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.operands.size_hint()
    }
}

impl<'a> DoubleEndedIterator<&'a [u8]> for Operands<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a [u8]> {
        match (self.operands.next(), self.lens.next()) {
            (Some(operand), Some(len)) =>
                unsafe { Some(mem::transmute(raw::Slice { data: *operand, len: *len as uint })) },
            _ => None
        }
    }
}

impl<'a> Clone for Operands<'a> {
    fn clone(&self) -> Operands<'a> { *self }
}

impl<'a> RandomAccessIterator<&'a [u8]> for Operands<'a> {
    fn indexable(&self) -> uint {
        self.operands.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a [u8]> {
        match (self.operands.idx(index), self.lens.idx(index)) {
            (Some(operand), Some(len)) =>
                unsafe { Some(mem::transmute(raw::Slice { data: *operand, len: *len as uint })) },
            _ => None
        }
    }
}

struct MergeOperatorState {
    name: CString,
    merge_operator: Box<MergeOperator>
}

impl MergeOperatorState {
    fn new(name: &str, merge_operator: Box<MergeOperator>) -> *mut rocksdb_mergeoperator_t {
        let state = box MergeOperatorState { name: name.to_c_str(),
                                             merge_operator: merge_operator };
        unsafe {
            rocksdb_mergeoperator_create(mem::transmute(state),
                                         merge_operator_destructor_callback,
                                         full_merge_callback,
                                         partial_merge_callback,
                                         merge_operator_delete_callback,
                                         merge_operator_name_callback)
        }
    }
}

/// Callback that rocksdb will execute in order to get the name of the merge operator.
extern fn merge_operator_name_callback(state: *mut c_void) -> *const i8 {
     let x: &MergeOperatorState = unsafe { &*(state as *mut MergeOperatorState) };
     x.name.as_ptr()
}

/// Callback that rocksdb will execute to perform a full merge.
extern fn full_merge_callback(state: *mut c_void,
                              key: *const i8, key_len: u64,
                              existing_val: *const i8, existing_val_len: u64,
                              operands: *const *const i8, operand_lens: *const u64,
                              num_operands: i32,
                              success: *mut u8, len: *mut u64)
                              -> *mut i8 {
    unsafe {
        slice::raw::buf_as_slice(key as *const u8, key_len as uint, |key| {
            buf_as_optional_slice(existing_val as *const u8, existing_val_len as uint, |existing_val| {
                let operands = Operands::new(operands as *const *const u8, operand_lens, num_operands as uint);
                let state: &mut MergeOperatorState = &mut *(state as *mut MergeOperatorState);

                // The RocksDB C API does not correctly handle merge operator failures, so it is better
                // to catch them here and explicitly unwind the stack. If the failure is propogated
                // through the C API, a segfault occurs.
                let mut val = (*state.merge_operator).full_merge(key, existing_val, operands).unwrap();
                val.shrink_to_fit();
                let ptr = val.as_mut_ptr();
                *len = val.len() as u64;
                *success = 1;
                mem::forget(val);
                ptr as *mut i8
            })
        })
    }
}

/// Callback that rocksdb will execute to perform a partial merge.
extern fn partial_merge_callback(state: *mut c_void,
                                 key: *const i8, key_len: u64,
                                 operands: *const *const i8, operand_lens: *const u64,
                                 num_operands: i32,
                                 success: *mut u8, len: *mut u64)
                                 -> *mut i8 {
    unsafe {
        slice::raw::buf_as_slice(key as *const u8, key_len as uint, |key| {
            let operands = Operands::new(operands as *const *const u8, operand_lens, num_operands as uint);
            let state: &mut MergeOperatorState = &mut *(state as *mut MergeOperatorState);

            // The RocksDB C API does not correctly handle merge operator failures, so it is better
            // to catch them here and explicitly unwind the stack. If the failure is propogated
            // through the C API, a segfault occurs.
            let mut val = (*state.merge_operator).partial_merge(key, operands).unwrap();
            val.shrink_to_fit();
            let ptr = val.as_mut_ptr();
            *len = val.len() as u64;
            *success = 1;
            mem::forget(val);
            ptr as *mut i8
        })
    }
}

/// Callback that rocksdb will execute to  the result of a merge.
extern fn merge_operator_delete_callback(_state: *mut c_void,
                                         val: *const i8, val_len: u64) {
    let _ = unsafe { Vec::from_raw_parts(val as *mut u8, val_len as uint, val_len as uint) };
}

/// Callback that rocksdb will execute to destroy the merge operator.
extern fn merge_operator_destructor_callback(state: *mut c_void) {
    // Convert back to a box and let destructor reclaim
    let _: Box<MergeOperatorState> = unsafe {mem::transmute(state)};
}

/**
 * Form a slice from a pointer and length (as a number of units,
 * not bytes).
 */
#[inline]
pub unsafe fn buf_as_optional_slice<T,U>(p: *const T,
                                         len: uint,
                                         f: |v: Option<&[T]>| -> U) -> U {
    if p == ptr::null() {
        f(None)
    } else {
        let slice = raw::Slice { data: p, len: len };
        f(Some(mem::transmute(slice)))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Iterator
///////////////////////////////////////////////////////////////////////////////////////////////////

#[deriving(Show, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValue {
    pub key: Vec<u8>,
    pub value: Vec<u8>
}

pub struct KeyValues {
    itr: *mut rocksdb_iterator_t
}

impl Drop for KeyValues {
    fn drop(&mut self) {
        debug!("KeyValues::drop");
        unsafe { rocksdb_iter_destroy(self.itr) }
    }
}

impl KeyValues {
    pub fn new(itr: *mut rocksdb_iterator_t) -> KeyValues {
        unsafe { rocksdb_iter_seek_to_first(itr) };
        KeyValues { itr: itr }
    }

    fn itr(&self) -> *const rocksdb_iterator_t {
        self.itr as *const rocksdb_iterator_t
    }

    fn itr_mut(&mut self) -> *mut rocksdb_iterator_t {
        self.itr
    }
}

impl Iterator<KeyValue> for KeyValues {
    fn next(&mut self) -> Option<KeyValue> {
        if unsafe { rocksdb_iter_valid(self.itr()) } == 0 {
            return None;
        }

        let mut len: u64 = 0;

        unsafe {
            let key_ptr = rocksdb_iter_key(self.itr(), &mut len) as *mut u8;
            let key = CVec::new(key_ptr, len as uint).as_slice().to_vec();

            let val_ptr = rocksdb_iter_value(self.itr(), &mut len) as *mut u8;
            let val = CVec::new(val_ptr, len as uint).as_slice().to_vec();

            rocksdb_iter_next(self.itr_mut());
            Some(KeyValue { key: key, value: val })
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////// Write Batch
///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct WriteBatch {
    write_batch: *mut rocksdb_writebatch_t
}

impl Drop for WriteBatch {
    fn drop(&mut self) {
        debug!("WriteBatch::drop");
        unsafe { rocksdb_writebatch_destroy(self.write_batch) }
    }
}

impl WriteBatch {

    pub fn new() -> WriteBatch {
        WriteBatch { write_batch: unsafe { rocksdb_writebatch_create() } }
    }

    pub fn put(&mut self, column_family: &ColumnFamily, key: &[u8], val: &[u8]) {
        unsafe {
            rocksdb_writebatch_put_cf(self.write_batch,
                                      column_family.column_family,
                                      key.as_ptr() as *const i8, key.len() as u64,
                                      val.as_ptr() as *const i8, val.len() as u64)
        }
    }

    pub fn delete(&mut self, column_family: &ColumnFamily, key: &[u8]) {
        unsafe {
            rocksdb_writebatch_delete_cf(self.write_batch,
                                         column_family.column_family,
                                         key.as_ptr() as *const i8, key.len() as u64)
        }
    }

    pub fn merge(&mut self, column_family: &ColumnFamily, key: &[u8], val: &[u8]) {
        unsafe {
            rocksdb_writebatch_merge_cf(self.write_batch,
                                        column_family.column_family,
                                        key.as_ptr() as *const i8, key.len() as u64,
                                        val.as_ptr() as *const i8, val.len() as u64)
        }
    }

    pub fn count(&self) -> i32 {
        unsafe { rocksdb_writebatch_count(self.write_batch) }
    }

    pub fn clear(&mut self) {
        unsafe { rocksdb_writebatch_clear(self.write_batch) }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
////// Slice Transform
///////////////////////////////////////////////////////////////////////////////////////////////////



///////////////////////////////////////////////////////////////////////////////////////////////////
////// Options
///////////////////////////////////////////////////////////////////////////////////////////////////

/// Options for opening or creating a RocksDB database.
pub struct DatabaseOptions {
  options: *mut rocksdb_options_t
}

impl Drop for DatabaseOptions {
  fn drop(&mut self) {
    debug!("DatabaseOptions::drop");
    unsafe { rocksdb_options_destroy(self.options) }
  }
}

impl DatabaseOptions {

    /// Create a new database options struct for specifying configuration to use when opening or
    /// creating a database.
    pub fn new() -> DatabaseOptions {
        let options = unsafe { rocksdb_options_create() };
        DatabaseOptions { options: options }
    }

    /// By default, RocksDB uses a single background thread for flush and compaction. Calling this
    /// function will set it up such that `total_threads` number of threads is used. A good value
    /// for `total_threads` is the number of cores. You should call this function if your system is
    /// bottlenecked by RocksDB.
    pub fn increase_parallelism(&mut self, total_threads: i32) -> &mut DatabaseOptions {
        unsafe { rocksdb_options_increase_parallelism(self.options, total_threads) };
        self
    }

    /// Configure the database for bulk-writing. All data will be flushed to level 0 without
    /// automatic compaction. It is recommended to manually compact the database before resuming
    /// reads.
    pub fn prepare_for_bulkload(&mut self) -> &mut DatabaseOptions {
        unsafe { rocksdb_options_prepare_for_bulk_load(self.options) };
        self
    }

    /// Get the raw `rocksdb_options_t` struct.
    fn options(&self) -> *const rocksdb_options_t {
        self.options as *const rocksdb_options_t
    }

    /// Mutably get the raw `rocksdb_options_t` struct.
    fn options_mut(&self) -> *mut rocksdb_options_t {
        self.options
    }
}

/// Options for opening or creating a column family in a RocksDB database.
pub struct ColumnFamilyOptions {
    options: *mut rocksdb_options_t,
    comparator: Option<Comparator>
}

impl Drop for ColumnFamilyOptions {
    fn drop(&mut self) {
        debug!("ColumnFamilyOptions::drop");
        unsafe { rocksdb_options_destroy(self.options) }
    }
}

impl ColumnFamilyOptions {

    /// Create a new column family options struct for specifying configuration to use when opening
    /// or creating a column family.
    pub fn new() -> ColumnFamilyOptions {
        let options = unsafe { rocksdb_options_create() };
        ColumnFamilyOptions { options: options, comparator: None }
    }

    /// Configure the column family to use level-style compaction with a memtable of size
    /// `memtable_size`.
    pub fn optimize_level_style_compaction(&mut self,
                                           memtable_size: u64)
                                           -> &mut ColumnFamilyOptions {
        unsafe { rocksdb_options_optimize_level_style_compaction(self.options, memtable_size) };
        self
    }

    /// Configure the column family to use universal-style compaction with a memtable of size
    /// `memtable_size`.
    pub fn optimize_universal_style_compaction(&mut self,
                                               memtable_size: u64)
                                               -> &mut ColumnFamilyOptions {
        unsafe { rocksdb_options_optimize_universal_style_compaction(self.options, memtable_size) };
        self
    }

    /// Comparator used to define the order of keys in the table.
    /// Default: a comparator that uses lexicographic byte-wise ordering
    ///
    /// REQUIRES: The client must ensure that the comparator supplied
    /// here has the same name and orders keys *exactly* the same as the
    /// comparator provided to previous open calls on the same DB.
    pub fn set_comparator(&mut self,
                          name: &str,
                          compare: |&[u8], &[u8]|: Sync + Send -> Ordering)
                          -> &mut ColumnFamilyOptions {
        let comparator = Comparator::new(name, compare);
        unsafe { rocksdb_options_set_comparator(self.options, comparator.comparator) };
        self.comparator = Some(comparator);
        self
    }

    /// REQUIRES: The client must provide a merge operator if Merge operation
    /// needs to be accessed. Calling Merge on a DB without a merge operator
    /// would result in Status::NotSupported. The client must ensure that the
    /// merge operator supplied here has the same name and *exactly* the same
    /// semantics as the merge operator provided to previous open calls on
    /// the same DB. The only exception is reserved for upgrade, where a DB
    /// previously without a merge operator is introduced to Merge operation
    /// for the first time. It's necessary to specify a merge operator when
    /// openning the DB in this case.
    /// Default: nullptr
    pub fn set_merge_operator(&mut self,
                              name: &str,
                              merge_operator: Box<MergeOperator>)
                              -> &mut ColumnFamilyOptions {
        let merge_operator = MergeOperatorState::new(name, merge_operator);
        unsafe { rocksdb_options_set_merge_operator(self.options, merge_operator) };
        self
    }

    /// Get the raw `rocksdb_options_t` struct.
    fn options(&self) -> *const rocksdb_options_t {
        self.options as *const rocksdb_options_t
    }
}

/// Options for writing to a RocksDB database.
pub struct WriteOptions {
  options: *mut rocksdb_writeoptions_t
}

impl Drop for WriteOptions {
    fn drop(&mut self) {
        unsafe { rocksdb_writeoptions_destroy(self.options); }
    }
}

impl WriteOptions {

    /// Create a new write options struct for specifying configuration to use when writing to a
    /// RocksDB database.
    pub fn new() -> WriteOptions {
        unsafe {
            let options = rocksdb_writeoptions_create();
            WriteOptions { options: options }
        }
    }

    /// If true, the write will be flushed from the operating system buffer cache (by calling
    /// `WritableFile::Sync`) before the write is considered complete. If this flag is true, writes
    /// will be slower.
    ///
    /// If this flag is false, and the machine crashes, some recent writes may be lost. Note that if
    /// it is just the process that crashes (i.e., the machine does not reboot), no writes will be
    /// lost even if `sync==false`. In other words, a DB write with `sync==false` has similar crash
    /// semantics as the `write` system call.  A DB write with `sync==true` has similar crash
    /// semantics to a `write` system call followed by `fdatasync`.
    pub fn set_sync(&mut self, sync: bool) -> &mut WriteOptions {
        unsafe {
            rocksdb_writeoptions_set_sync(self.options,
                                          if sync { 1 } else { 0 });
        }
        self
    }

    /// If false, writes will not first go to the write ahead log, and the write may got lost after
    /// a crash.
    ///
    /// Default: `true`
    pub fn set_write_to_wal(&mut self, write_to_wal: bool) -> &mut WriteOptions {
        unsafe {
            rocksdb_writeoptions_disable_WAL(self.options,
                                             if write_to_wal { 0 } else { 1 });
        }
        self
    }

    /// Get the raw `rocksdb_writeoptions_t` struct.
    fn options(&self) -> *const rocksdb_writeoptions_t {
        self.options as *const rocksdb_writeoptions_t
    }
}

/// Options for reading from a RocksDB database.
pub struct ReadOptions {
  options: *mut rocksdb_readoptions_t
}

impl Drop for ReadOptions {
    fn drop(&mut self) {
        unsafe { rocksdb_readoptions_destroy(self.options); }
    }
}

impl ReadOptions {

    /// Create a new read options struct for specifying configuration to use when reading from a
    /// RocksDB database.
    pub fn new() -> ReadOptions {
        unsafe {
            let options = rocksdb_readoptions_create();
            ReadOptions { options: options }
        }
    }

    /// If true, all data read from underlying storage will be verified against corresponding
    /// checksums.
    ///
    /// Default: true
    pub fn set_verify_checksums(&mut self, verify_checksums: bool) -> &mut ReadOptions {
        unsafe {
            rocksdb_readoptions_set_verify_checksums(self.options,
                                                     if verify_checksums { 1 } else { 0 });
        }
        self
    }

    /// Should the "data block"/"index block"/"filter block" read for this iteration be cached in
    /// memory? Callers may wish to set this field to false for bulk scans.
    ///
    /// Default: true
    pub fn set_fill_cache(&mut self, fill_cache: bool) -> &mut ReadOptions {
        unsafe { rocksdb_readoptions_set_fill_cache(self.options, if fill_cache { 1 } else { 0 }); }
        self
    }

    /// Get the raw `rocksdb_readoptions_t` struct.
    fn options(&self) -> *const rocksdb_readoptions_t {
        self.options as *const rocksdb_readoptions_t
    }
}
