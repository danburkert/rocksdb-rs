use ffi::*;
use std::c_str::CString;
use std::collections::HashMap;
use std::path::posix::Path;
use std::ptr;
use std::string;
use std::vec;

use Table;
use columnfamily::{ColumnFamily};
use options::{DatabaseOptions, ColumnFamilyOptions, WriteOptions, ReadOptions};

static DEFAULT_COLUMN_FAMILY: &'static str = "default";

pub struct Database {
    database: *mut rocksdb_t,
    column_families: HashMap<String, ColumnFamily>
}

impl Database {

    /// Create a RocksDB database at the provided path with the default column family and configured
    /// with the provided options.
    pub fn create(path: &Path,
                  db_options: DatabaseOptions,
                  cf_options: ColumnFamilyOptions)
                  -> Result<Database, String> {
        let mut column_families = HashMap::new();
        column_families.insert(DEFAULT_COLUMN_FAMILY.to_string(), cf_options);
        Database::create_with_column_families(path, db_options, column_families)
    }

    /// Create a RocksDB database at the provided path with the specified column families.
    pub fn create_with_column_families(path: &Path,
                                       db_options: DatabaseOptions,
                                       cf_options: HashMap<String, ColumnFamilyOptions>)
                                       -> Result<Database, String> {
        unsafe {
            let raw_db_opts = db_options.options();
            rocksdb_options_set_error_if_exists(raw_db_opts, 1);
            rocksdb_options_set_create_if_missing(raw_db_opts, 1);
        }
        Database::create_or_open_with_column_families(path, db_options, cf_options)
    }

    /// Open a RocksDB database at the provided path with the default column family and configured
    /// with the provided options.
    pub fn open(path: &Path,
                db_options: DatabaseOptions,
                cf_options: ColumnFamilyOptions)
                -> Result<Database, String> {
        let mut column_families = HashMap::new();
        column_families.insert(DEFAULT_COLUMN_FAMILY.to_string(), cf_options);
        Database::open_with_column_families(path, db_options, column_families)
    }

    /// Create a RocksDB database at the provided path with the specified column families.
    pub fn open_with_column_families(path: &Path,
                                     db_options: DatabaseOptions,
                                     cf_options: HashMap<String, ColumnFamilyOptions>)
                                     -> Result<Database, String> {
        unsafe {
            let raw_db_opts = db_options.options();
            rocksdb_options_set_error_if_exists(raw_db_opts, 0);
            rocksdb_options_set_create_if_missing(raw_db_opts, 0);
        }
        Database::create_or_open_with_column_families(path, db_options, cf_options)
    }

    /// Create or open a RocksDB database at the provided path with the specified column families.
    fn create_or_open_with_column_families(path: &Path,
                                           db_options: DatabaseOptions,
                                           cf_options: HashMap<String, ColumnFamilyOptions>)
                                           -> Result<Database, String> {
        let num_cfs = cf_options.len();
        let (cf_names, cf_options) = vec::unzip(cf_options.into_iter());

        // Translate the column family names to a vec of c string pointers.
        let cf_c_names: Vec<CString> = cf_names.iter()
                                               .map(|cf_name| cf_name.to_c_str())
                                               .collect();
        let cf_c_name_ptrs: Vec<*const i8> = cf_c_names.iter()
                                                       .map(|cf_c_name| cf_c_name.as_ptr())
                                                       .collect();
        let cf_option_ptrs: Vec<*const rocksdb_options_t> =
            cf_options.iter()
                      .map(|option| option.options() as *const rocksdb_options_t)
                      .collect();
        let c_path = path.to_c_str();
        let cf_ptrs: *mut *mut rocksdb_column_family_handle_t = &mut ptr::null_mut();
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            let database = rocksdb_open_column_families(db_options.options() as *const rocksdb_options_t,
                                                        c_path.as_ptr(),
                                                        num_cfs as i32,
                                                        cf_c_name_ptrs.as_ptr(),
                                                        cf_option_ptrs.as_ptr(),
                                                        cf_ptrs,
                                                        &mut error);
            drop(c_path); // Ensure c-string path isn't dropped before the pointer is used
            drop(cf_c_names); // Ensure cf names are not dropped before the pointers are used
            drop(cf_options); // Ensure that options are not dropped before pointers are used
            if error == ptr::null_mut() {
                let column_families: HashMap<String, ColumnFamily> =
                    cf_names.into_iter()
                            .enumerate()
                            .map(|(i, cf_name)|
                                 (cf_name,
                                  ColumnFamily::create(database, (*cf_ptrs).offset(i as int))))
                            .collect();
                Ok(Database { database: database, column_families: column_families })
            } else {
                Err(string::raw::from_buf(error as *const u8))
            }
        }
    }
}

impl Table for Database {

    fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        match self.column_families.find_equiv(&DEFAULT_COLUMN_FAMILY) {
            Some(column_family) => column_family.get(options, key),
            None => Err("No default column family for database.".to_string())
        }
    }

    fn put(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<(), String> {
        match self.column_families.find_equiv(&DEFAULT_COLUMN_FAMILY) {
            Some(column_family) => column_family.put(options, key, val),
            None => Err("No default column family for database.".to_string())
        }
    }

    fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), String> {
        match self.column_families.find_equiv(&DEFAULT_COLUMN_FAMILY) {
            Some(column_family) => column_family.delete(options, key),
            None => Err("No default column family for database.".to_string())
        }
    }
}

impl Drop for Database {
  fn drop(&mut self) {
    unsafe {
        for (_, column_family) in self.column_families.iter_mut() {
            column_family.destroy();
        }
        rocksdb_close(self.database);
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use options::{DatabaseOptions, ColumnFamilyOptions, ReadOptions, WriteOptions};
    use std::io::TempDir;
    use Table;

    #[test]
    fn test_create_database() {
        let dir = TempDir::new("foo").unwrap();
        match Database::create(dir.path(), DatabaseOptions::new(), ColumnFamilyOptions::new()) {
            Ok(_) => (),
            Err(msg) => fail!(format!("failure!: {}", msg))
        }
    }

    #[test]
    fn test_create_while_open_fails() {
        let dir = TempDir::new("").unwrap();
        assert!(Database::create(dir.path(), DatabaseOptions::new(), ColumnFamilyOptions::new()).is_ok());
        assert!(Database::create(dir.path(), DatabaseOptions::new(), ColumnFamilyOptions::new()).is_err());
    }

    #[test]
    fn test_create_duplicate_fails() {
        let dir = TempDir::new("").unwrap();

        {
            assert!(Database::create(dir.path(), DatabaseOptions::new(), ColumnFamilyOptions::new()).is_ok());
        }
        {
            assert!(Database::create(dir.path(), DatabaseOptions::new(), ColumnFamilyOptions::new()).is_err());
        }
    }

    #[test]
    fn test_put_get() {
        let dir = TempDir::new("").unwrap();
        let db_options = DatabaseOptions::new();
        let cf_options = ColumnFamilyOptions::new();
        let read_options = ReadOptions::new();
        let write_options = WriteOptions::new();

        let table = Database::create(dir.path(), db_options, cf_options).unwrap();
        assert!(table.put(&write_options, b"key", b"val").is_ok());

        assert!(table.get(&read_options, b"non-existant").unwrap().is_none());
        assert_eq!(table.get(&read_options, b"key").unwrap().unwrap(), b"val".to_vec());
    }

    #[test]
    fn test_comparator() {
        let dir = TempDir::new("").unwrap();
        let db_options = DatabaseOptions::new();
        let cf_options = ColumnFamilyOptions::new();
        let read_options = ReadOptions::new();
        let write_options = WriteOptions::new();

        let table = Database::create(dir.path(), db_options, cf_options).unwrap();
        assert!(table.put(&write_options, b"key", b"val").is_ok());

        assert!(table.get(&read_options, b"non-existant").unwrap().is_none());
        assert_eq!(table.get(&read_options, b"key").unwrap().unwrap(), b"val".to_vec());

    }

}
