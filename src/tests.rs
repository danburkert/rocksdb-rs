use super::*;
use super::merge_operators::{AddMergeOperator, ConcatMergeOperator};
use std::io;

#[test]
fn test_create_database() {
    let dir = io::TempDir::new("foo").unwrap();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
}

#[test]
fn test_create_database_multiple_cfs() {
    let dir = io::TempDir::new("foo").unwrap();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new()),
                   ("other".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
}

#[test]
fn test_create_while_open_fails() {
    let dir = io::TempDir::new("").unwrap();
    let cfs1 = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let cfs2 = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs1).is_ok());
    assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs2).is_err());
}

#[test]
fn test_create_duplicate_fails() {
    let dir = io::TempDir::new("").unwrap();

    {
        let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
        assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs).is_ok());
    }
    {
        let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
        assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs).is_err());
    }
}

#[test]
fn test_put_get() {
    let dir = io::TempDir::new("").unwrap();
    let db_options = DatabaseOptions::new();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new()),
                   ("other".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let read_options = ReadOptions::new();
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), db_options, cfs).unwrap();
    let default = db.get_column_family("default").unwrap();
    assert!(default.put(&write_options, b"key", b"val").is_ok());
    assert!(default.get(&read_options, b"non-existant").unwrap().is_none());
    assert_eq!(default.get(&read_options, b"key").unwrap().unwrap().as_slice(), b"val");

    let other = db.get_column_family("other").unwrap();
    assert!(other.put(&write_options, b"key", b"val").is_ok());
    assert!(other.get(&read_options, b"non-existant").unwrap().is_none());
    assert_eq!(other.get(&read_options, b"key").unwrap().unwrap().as_slice(), b"val");
}

#[test]
fn test_iterator() {
    let dir = io::TempDir::new("").unwrap();
    let mut reversed_cf_options = ColumnFamilyOptions::new();

    reversed_cf_options.set_comparator("foo", |x, y| {
        y.cmp(x)
    });

    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new()),
                   ("other".to_string(), reversed_cf_options)).into_iter().collect();
    let read_options = ReadOptions::new();
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
    let default = db.get_column_family("default").unwrap();
    let other = db.get_column_family("other").unwrap();

    let kvs = vec!((b"1", b"1"),
                   (b"2", b"2"),
                   (b"3", b"3"),
                   (b"4", b"4"),
                   (b"5", b"5"),
                   (b"a", b"a"),
                   (b"b", b"b"),
                   (b"c", b"c"),
                   (b"fooz", b"baz"));

    for &(k, v) in kvs.iter() {
        default.put(&write_options, k, v).unwrap();
    }

    for (kv, tuple) in default.iter(&read_options).unwrap().zip(kvs.iter()) {
        assert!(kv.key[] == tuple.val0());
        assert!(kv.value[] == tuple.val1());
    }

    assert_eq!(0, other.iter(&read_options).unwrap().count());
}

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

#[test]
fn test_set_comparator() {
    let mut options = ColumnFamilyOptions::new();
    options.set_comparator("foo", |x, y| {
        y.cmp(x)
    });
}

#[test]
fn test_comparator() {
    let dir = io::TempDir::new("").unwrap();
    let mut reversed_cf_options = ColumnFamilyOptions::new();

    reversed_cf_options.set_comparator("foo", |x, y| {
        y.cmp(x)
    });

    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new()),
                   ("reversed".to_string(), reversed_cf_options)).into_iter().collect();
    let read_options = ReadOptions::new();
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
    let default = db.get_column_family("default").unwrap();
    let reversed = db.get_column_family("reversed").unwrap();

    let kvs = vec!((b"1", b"1"),
                   (b"2", b"2"),
                   (b"3", b"3"),
                   (b"4", b"4"),
                   (b"5", b"5"));

    for &(k, v) in kvs.iter() {
        default.put(&write_options, k, v).unwrap();
        reversed.put(&write_options, k, v).unwrap();
    }

    for (kv, tuple) in default.iter(&read_options).unwrap().zip(kvs.iter()) {
        assert!(kv.key[] == tuple.val0());
        assert!(kv.value[] == tuple.val1());
    }

    for (kv, tuple) in reversed.iter(&read_options).unwrap().zip(kvs.iter().rev()) {
        assert!(kv.key[] == tuple.val0());
        assert!(kv.value[] == tuple.val1());
    }
}

#[test]
fn test_set_merge_operator() {
    let mut options = ColumnFamilyOptions::new();
    options.set_merge_operator("foo", box ConcatMergeOperator);
}

#[test]
fn test_merge() {
    let dir = io::TempDir::new("").unwrap();
    let mut options = ColumnFamilyOptions::new();
    options.set_merge_operator("concat", box ConcatMergeOperator);

    let cfs = vec!(("default".to_string(), options)).into_iter().collect();
    let read_options = &ReadOptions::new();
    let write_options = &WriteOptions::new();

    let db = Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
    let default = db.get_column_family("default").unwrap();

    default.put(write_options, b"key", b"foo").unwrap();
    default.merge(write_options, b"key", b"-").unwrap();
    default.merge(write_options, b"key", b"bar").unwrap();
    default.merge(write_options, b"key", b"-").unwrap();
    default.merge(write_options, b"key", b"baz").unwrap();
    assert_eq!(b"foo-bar-baz", default.get(read_options, b"key").unwrap().unwrap().as_slice());
}

#[test]
fn test_associative_merge() {
    let dir = io::TempDir::new("").unwrap();
    let mut options = ColumnFamilyOptions::new();
    options.set_merge_operator("add", box AddMergeOperator);

    let cfs = vec!(("default".to_string(), options)).into_iter().collect();
    let read_options = &ReadOptions::new();
    let write_options = &WriteOptions::new();

    let db = Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
    let default = db.get_column_family("default").unwrap();

    default.put(write_options, b"key", AddMergeOperator::write_u64(1).unwrap().as_slice()).unwrap();
    default.merge(write_options, b"key", AddMergeOperator::write_u64(1).unwrap().as_slice()).unwrap();
    default.merge(write_options, b"key", AddMergeOperator::write_u64(2).unwrap().as_slice()).unwrap();
    default.merge(write_options, b"key", AddMergeOperator::write_u64(3).unwrap().as_slice()).unwrap();
    default.merge(write_options, b"key", AddMergeOperator::write_u64(5).unwrap().as_slice()).unwrap();
    assert_eq!(AddMergeOperator::write_u64(12).unwrap().as_slice(),
               default.get(read_options, b"key").unwrap().unwrap().as_slice());
}

#[test]
#[should_fail]
fn test_merge_fail() {

    struct FailingMergeOperator;

    impl AssociativeMergeOperator for FailingMergeOperator {
        fn merge(&self,
                 _key: &[u8],
                 _existing_val: Vec<u8>,
                 _operand: &[u8])
                 -> io::IoResult<Vec<u8>> {
            Err(io::standard_error(io::OtherIoError))
        }
    }

    let dir = io::TempDir::new("").unwrap();
    let mut options = ColumnFamilyOptions::new();
    options.set_merge_operator("add", box FailingMergeOperator);

    let cfs = vec!(("default".to_string(), options)).into_iter().collect();
    let read_options = &ReadOptions::new();
    let write_options = &WriteOptions::new();

    let db = Database::create(dir.path(), DatabaseOptions::new(), cfs).unwrap();
    let default = db.get_column_family("default").unwrap();

    default.put(write_options, b"key", b"a").unwrap();
    default.merge(write_options, b"key", b"b").unwrap();
    default.merge(write_options, b"key", b"c").unwrap();
    assert_eq!(b"foo-bar-baz", default.get(read_options, b"key").unwrap().unwrap().as_slice());
}

#[test]
fn test_write_batch() {
    let dir = io::TempDir::new("").unwrap();
    let db_options = DatabaseOptions::new();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new()),
                   ("other".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let read_options = ReadOptions::new();
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), db_options, cfs).unwrap();
    let default = db.get_column_family("default").unwrap();
    let other = db.get_column_family("other").unwrap();

    default.put(&write_options, b"key", b"val1").unwrap();
    other.put(&write_options, b"key", b"val1").unwrap();

    let mut batch = WriteBatch::new();
    batch.put(default, b"key", b"to-be-cleared");
    batch.put(other, b"key", b"to-be-cleared");
    batch.clear();
    batch.put(default, b"key", b"val2");
    batch.delete(other, b"key");

    let _ = db.write(&write_options, batch);

    assert_eq!(default.get(&read_options, b"key").unwrap().unwrap().as_slice(), b"val2");
    assert!(other.get(&read_options, b"key").unwrap().is_none());
}
