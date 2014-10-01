use std::{ptr, string, vec};
use ffi::*;
use iterator::KeyValues;

use Table;
use options::{ReadOptions, WriteOptions};

pub struct ColumnFamily {
    database: *mut rocksdb_t,
    column_family: *mut rocksdb_column_family_handle_t
}

impl ColumnFamily {

    /// Create a new `ColumnFamily` with the provided database and column family pointers.
    /// The caller is responsible for calling `destroy` on the returned `ColumnFamily`
    /// before closing the provided database pointer.
    pub fn create(database: *mut rocksdb_t,
                  column_family: *mut rocksdb_column_family_handle_t)
                  -> ColumnFamily {
        ColumnFamily { database: database, column_family: column_family }
    }

    /// An alternative to implementing drop. This allows the owning `Database` instance to destroy
    /// the column family handle owned by this `ColumnFamily` during drop. This method should only
    /// be called by the owning `Database`.
    pub fn destroy(&mut self) {
        unsafe {
            rocksdb_column_family_handle_destroy(self.column_family);
        }
    }
}

impl Table for ColumnFamily {

    fn get(&self, options: &ReadOptions, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        let mut error: *mut i8 = ptr::null_mut();
        let mut val_len: u64 = 0;
        unsafe {
            let val = rocksdb_get_cf(self.database,
                                     options.options() as *const rocksdb_readoptions_t,
                                     self.column_family,
                                     key.as_ptr() as *const i8, key.len() as u64,
                                     (&mut val_len) as *mut u64,
                                     (&mut error) as *mut *mut i8);

            if error == ptr::null_mut() {
                if val == ptr::null_mut() {
                    Ok(None)
                } else {
                    let vec = vec::raw::from_buf(val as *const u8, val_len as uint);
                    Ok(Some(vec))
                }
            } else {
                Err(string::raw::from_buf(error as *const u8))
            }
        }
    }

    fn iter(&self, options: &ReadOptions) -> KeyValues {
        let read_options = options.options() as *const rocksdb_readoptions_t;
        let itr = unsafe {
            rocksdb_create_iterator_cf(self.database, read_options, self.column_family)
        };
        KeyValues::new(itr)
    }

    fn put(&self, options: &WriteOptions, key: &[u8], val: &[u8]) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_put_cf(self.database,
                           options.options() as *const rocksdb_writeoptions_t,
                           self.column_family,
                           key.as_ptr() as *const i8, key.len() as u64,
                           val.as_ptr() as *const i8, val.len() as u64,
                           (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(string::raw::from_buf(error as *const u8))
            }
        }
    }

    fn delete(&self, options: &WriteOptions, key: &[u8]) -> Result<(), String> {
        let mut error: *mut i8 = ptr::null_mut();
        unsafe {
            rocksdb_delete_cf(self.database,
                              options.options() as *const rocksdb_writeoptions_t,
                              self.column_family,
                              key.as_ptr() as *const i8, key.len() as u64,
                              (&mut error) as *mut *mut i8);
            if error == ptr::null_mut() {
                Ok(())
            } else {
                Err(string::raw::from_buf(error as *const u8))
            }
        }
    }

}
