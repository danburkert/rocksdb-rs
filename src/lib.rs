#![feature(phase, globs, unsafe_destructor, if_let)]
#[phase(plugin, link)] extern crate log;
extern crate libc;

use libc::c_void;
use std::c_str::CString;
use std::c_vec::CVec;
use std::collections::HashMap;
use std::path::posix::Path;
use std::{mem, ptr, raw, slice, vec};

use ffi::*;

#[cfg(test)]
mod tests;
mod ffi;


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
        debug!("num of column families: {}", num_cfs);
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
        self.column_families.find_equiv(&column_family)
    }

    pub fn get_column_families(&self) -> &HashMap<String, ColumnFamily> {
        &self.column_families
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
extern "C" fn comparator_name_callback(state: *mut c_void) -> *const i8 {
     let state: &ComparatorState = unsafe { &*(state as *mut ComparatorState) };
     state.name.as_ptr()
}

/// Callback that rocksdb will execute to compare keys.
extern "C" fn compare_callback(state: *mut c_void,
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
extern "C" fn comparator_destructor_callback(state: *mut c_void) {
    // Convert back to a box and let destructor reclaim
    let _: Box<ComparatorState> = unsafe {mem::transmute(state)};
}

///////////////////////////////////////////////////////////////////////////////////////////////////
//// Merge Operator
///////////////////////////////////////////////////////////////////////////////////////////////////

struct MergeOperatorState {
    name: CString,
    full_merge: |&[u8], Option<&[u8]>, Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>,
    partial_merge: |&[u8], Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>
}

struct MergeOperator {
    merge_operator: *mut rocksdb_mergeoperator_t
}

impl Drop for MergeOperator {
    fn drop(&mut self) {
        debug!("MergeOperator::drop");
        // See https://github.com/facebook/rocksdb/issues/343
        // unsafe { rocksdb_mergeoperator_destroy(self.merge_operator) }
    }
}

impl MergeOperator {
    fn new(name: &str,
           full_merge: |&[u8], Option<&[u8]>, Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>,
           partial_merge: |&[u8], Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>)
           -> MergeOperator {
        let state = box MergeOperatorState { name: name.to_c_str(),
                                             full_merge: full_merge,
                                             partial_merge: partial_merge };
        let merge_operator = unsafe {
            rocksdb_mergeoperator_create(mem::transmute(state),
                                         merge_operator_destructor_callback,
                                         full_merge_callback,
                                         partial_merge_callback,
                                         merge_operator_delete_callback,
                                         merge_operator_name_callback)
        };
        MergeOperator { merge_operator: merge_operator }
    }
}

/// Callback that rocksdb will execute in order to get the name of the merge operator.
extern "C" fn merge_operator_name_callback(state: *mut c_void) -> *const i8 {
     let x: &MergeOperatorState = unsafe { &*(state as *mut MergeOperatorState) };
     x.name.as_ptr()
}

/// Callback that rocksdb will execute to perform a full merge.
extern "C" fn full_merge_callback(state: *mut c_void,
                                  key: *const i8, key_len: u64,
                                  existing_val: *const i8, existing_val_len: u64,
                                  operands: *const *const i8, operand_lens: *const u64,
                                  num_operands: i32,
                                  success: *mut u8, len: *mut u64)
                                  -> *mut i8 {
    unsafe {
        slice::raw::buf_as_slice(key as *const u8, key_len as uint, |key| {
            buf_as_optional_slice(existing_val as *const u8, existing_val_len as uint, |existing_val| {
                bufs_as_slices(operands as *const *const u8, operand_lens, num_operands as uint, |operands| {
                    let state: &mut MergeOperatorState = &mut *(state as *mut MergeOperatorState);
                    match (state.full_merge)(key, existing_val, operands) {
                        Some(mut val) => {
                            let ptr = val.as_mut_ptr();
                            *len = val.len() as u64;
                            *success = 1;
                            mem::forget(val);
                            ptr as *mut i8
                        }
                        None => {
                            *success = 0;
                            ptr::null_mut()
                        }
                    }
                })
            })
        })
    }
}

/// Callback that rocksdb will execute to perform a partial merge.
extern "C" fn partial_merge_callback(state: *mut c_void,
                                     key: *const i8, key_len: u64,
                                     operands: *const *const i8, operand_lens: *const u64,
                                     num_operands: i32,
                                     success: *mut u8, len: *mut u64)
                                     -> *mut i8 {
    unsafe {
        slice::raw::buf_as_slice(key as *const u8, key_len as uint, |key| {
            bufs_as_slices(operands as *const *const u8, operand_lens, num_operands as uint, |operands| {
                let state: &mut MergeOperatorState = &mut *(state as *mut MergeOperatorState);
                match (state.partial_merge)(key, operands) {
                    Some(mut val) => {
                        val.shrink_to_fit();
                        let ptr = val.as_mut_ptr();
                        *len = val.len() as u64;
                        *success = 1;
                        mem::forget(val);
                        ptr as *mut i8
                    }
                    None => {
                        *len = 0;
                        *success = 0;
                        ptr::null_mut()
                    }
                }
            })
        })
    }
}

/// Callback that rocksdb will execute to  the result of a merge.
extern "C" fn merge_operator_delete_callback(_state: *mut c_void,
                                             val: *const i8, val_len: u64) {
    let _ = unsafe { Vec::from_raw_parts(val_len as uint, val_len as uint, val as *mut u8) };
}

/// Callback that rocksdb will execute to destroy the merge operator.
extern "C" fn merge_operator_destructor_callback(state: *mut c_void) {
    // Convert back to a box and let destructor reclaim
    let _: Box<MergeOperatorState> = unsafe {mem::transmute(state)};
}

unsafe fn bufs_as_slices<T, U>(ptrs: *const *const T,
                               lens: *const u64,
                               num: uint,
                               f: |Vec<&[T]>| -> U)
                               -> U {
    let mut bufs = Vec::with_capacity(num);
    for i in range(0, num) {
        let slice = raw::Slice { data: *ptrs.offset(i as int), len: *lens.offset(i as int) as uint };
        bufs.push(mem::transmute(slice))
    }

    f(bufs)
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

#[unsafe_destructor]
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
    comparator: Option<Comparator>,
    merge_operator: Option<MergeOperator>
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
        ColumnFamilyOptions { options: options, comparator: None, merge_operator: None }
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
                              full_merge: |&[u8], Option<&[u8]>, Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>,
                              partial_merge: |&[u8], Vec<&[u8]>|: Sync + Send -> Option<Vec<u8>>)
                              -> &mut ColumnFamilyOptions {
        let merge_operator = MergeOperator::new(name, full_merge, partial_merge);
        unsafe { rocksdb_options_set_merge_operator(self.options, merge_operator.merge_operator) };
        self.merge_operator = Some(merge_operator);
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

    /// Mutably get the raw `rocksdb_writeoptions_t` struct.
    fn options_mut(&self) -> *mut rocksdb_writeoptions_t {
        self.options
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

    /// Mutably get the raw `rocksdb_readoptions_t` struct.
    fn options_mut(&self) -> *mut rocksdb_readoptions_t {
        self.options
    }
}
