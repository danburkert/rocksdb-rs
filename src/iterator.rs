use std::c_vec::CVec;
use libc;
use ffi::*;

pub struct KeyValues {
    itr: *mut rocksdb_iterator_t
}

pub struct KeyValue {
    pub key: CVec<u8>,
    pub value: CVec<u8>
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

        let mut key_len: u64 = 0;
        let mut val_len: u64 = 0;

        unsafe {
            let key_ptr = rocksdb_iter_key(self.itr(), &mut key_len) as *mut u8;
            let val_ptr = rocksdb_iter_value(self.itr(), &mut val_len) as *mut u8;

            let key = CVec::new_with_dtor(key_ptr,
                                          key_len as uint,
                                          proc() { libc::free(key_ptr as *mut libc::c_void)});

            let val = CVec::new_with_dtor(val_ptr,
                                          val_len as uint,
                                          proc() { libc::free(val_ptr as *mut libc::c_void)});

            rocksdb_iter_next(self.itr_mut());
            Some(KeyValue { key: key, value: val })
        }
    }
}
