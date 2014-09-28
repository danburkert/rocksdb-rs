use ffi::*;
use options::ReadOptions;

struct Iterator<'a> {
    iterator: *mut rocksdb_iterator_t
}

impl Iterator<'a> {
    pub fn <'db, 'cf: 'db> new(
        database: &'db Database,
        column_family: &'cf ColumnFamily
        a: ReadOptions) -> Iterator {
        let iterator = unsafe { rocksdb_create_iterator_cf(

    }

}
