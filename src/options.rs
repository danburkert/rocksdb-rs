//! Options for using a RocksDB database.
//!
//! Option structs return themselves when properties are set in order to allow chained method
//! calls.

use ffi::*;
use comparator;

/// Options to use when opening or creating a RocksDB database.
pub struct DatabaseOptions {
  options: *mut rocksdb_options_t
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
    pub fn options(&self) -> *mut rocksdb_options_t {
        self.options
    }
}

impl Drop for DatabaseOptions {
  fn drop(&mut self) {
    unsafe { rocksdb_options_destroy(self.options); }
  }
}

/// Options for opening or creating a column family in a RocksDB database.
pub struct ColumnFamilyOptions {
    options: *mut rocksdb_options_t
}

impl ColumnFamilyOptions {

    /// Create a new column family options struct for specifying configuration to use when opening
    /// or creating a column family.
    pub fn new() -> ColumnFamilyOptions {
        let options = unsafe { rocksdb_options_create() };
        ColumnFamilyOptions { options: options }
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
                          compare: fn(&[u8], &[u8]) -> Ordering)
                          -> &mut ColumnFamilyOptions {


        let comparator = comparator::create(name, compare);
        unsafe { rocksdb_options_set_comparator(self.options, comparator) };
        self
    }

    /// Get the raw `rocksdb_options_t` struct.
    pub fn options(&self) -> *mut rocksdb_options_t {
        self.options
    }
}

impl Drop for ColumnFamilyOptions {
  fn drop(&mut self) {
    unsafe { rocksdb_options_destroy(self.options); }
  }
}

/// Options for writing to a RocksDB database.
pub struct WriteOptions {
  options: *mut rocksdb_writeoptions_t
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
    pub fn options(&self) -> *mut rocksdb_writeoptions_t {
        self.options
    }
}

impl Drop for WriteOptions {
    fn drop(&mut self) {
        unsafe { rocksdb_writeoptions_destroy(self.options); }
    }
}

/// Options for reading from a RocksDB database.
pub struct ReadOptions {
  options: *mut rocksdb_readoptions_t
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
    pub fn options(&self) -> *mut rocksdb_readoptions_t {
        self.options
    }
}

impl Drop for ReadOptions {
    fn drop(&mut self) {
        unsafe { rocksdb_readoptions_destroy(self.options); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_db_options() {
        let mut options = DatabaseOptions::new();
        options.increase_parallelism(16);
        options.prepare_for_bulkload();
    }

    #[test]
    fn test_create_cf_options() {
        let mut options = ColumnFamilyOptions::new();
        options.optimize_level_style_compaction(2 << 26);
        options.optimize_universal_style_compaction(2 << 26);
    }

    #[test]
    fn test_create_write_options() {
        let mut options = WriteOptions::new();
        options.set_sync(true);
        options.set_write_to_wal(false);
    }

    #[test]
    fn test_create_read_options() {
        let mut options = ReadOptions::new();
        options.set_verify_checksums(false);
        options.set_fill_cache(true);
    }
}
