use super::*;
use std::io::TempDir;

#[test]
fn test_create_database() {
    let dir = TempDir::new("foo").unwrap();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    match Database::create(dir.path(), DatabaseOptions::new(), cfs) {
        Ok(_) => (),
        Err(msg) => fail!(format!("failure!: {}", msg))
    }
}

#[test]
fn test_create_while_open_fails() {
    let dir = TempDir::new("").unwrap();
    let cfs1 = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let cfs2 = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs1).is_ok());
    assert!(Database::create(dir.path(), DatabaseOptions::new(), cfs2).is_err());
}

#[test]
fn test_create_duplicate_fails() {
    let dir = TempDir::new("").unwrap();

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
    let dir = TempDir::new("").unwrap();
    let db_options = DatabaseOptions::new();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let read_options = ReadOptions::new();
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), db_options, cfs).unwrap();
    let cf = db.get_column_family("default").unwrap();
    assert!(cf.put(&write_options, b"key", b"val").is_ok());

    assert!(cf.get(&read_options, b"non-existant").unwrap().is_none());
    assert_eq!(cf.get(&read_options, b"key").unwrap().unwrap().as_slice(), b"val");
}

#[test]
fn test_iterator() {
    let dir = TempDir::new("").unwrap();
    let db_options = DatabaseOptions::new();
    let cfs = vec!(("default".to_string(), ColumnFamilyOptions::new())).into_iter().collect();
    let mut read_options = ReadOptions::new();
    read_options.set_verify_checksums(true);
    read_options.set_fill_cache(false);
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), db_options, cfs).unwrap();
    let cf = db.get_column_family("default").unwrap();

    let kvs = vec!(
                   (b"1", b"1"),
                   (b"2", b"2"),
                   (b"3", b"3"),
                   (b"4", b"4"),
                   (b"5", b"5"),
                   (b"a", b"a"),
                   (b"b", b"b"),
                   (b"c", b"c"),
                   (b"fooz", b"baz"));

    for &(k, v) in kvs.iter() {
        cf.put(&write_options, k, v).unwrap();
    }

    for (kv, tuple) in cf.iter(&read_options).unwrap().zip(kvs.iter()) {
        assert!(kv.key[] == tuple.val0());
        assert!(kv.value[] == tuple.val1());
    }
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
        y.cmp(&x)
    });
}

#[test]
fn test_comparator() {
    let dir = TempDir::new("").unwrap();
    let db_options = DatabaseOptions::new();
    let mut cf_options = ColumnFamilyOptions::new();

    cf_options.set_comparator("foo", |x, y| {
        y.cmp(&x)
    });

    let cfs = vec!(("default".to_string(), cf_options)).into_iter().collect();
    let mut read_options = ReadOptions::new();
    read_options.set_verify_checksums(true);
    read_options.set_fill_cache(false);
    let write_options = WriteOptions::new();

    let db = Database::create(dir.path(), db_options, cfs).unwrap();
    let cf = db.get_column_family("default").unwrap();

    let kvs = vec!(
                   (b"1", b"1"),
                   (b"2", b"2"),
                   (b"3", b"3"),
                   (b"4", b"4"),
                   (b"5", b"5"));

    for &(k, v) in kvs.iter() {
        cf.put(&write_options, k, v).unwrap();
    }

    for kv in cf.iter(&read_options).unwrap() {
        println!("comparing {}", kv);
    }

    for (kv, tuple) in cf.iter(&read_options).unwrap().zip(kvs.iter().rev()) {
        println!("comparing {} to {}", kv, tuple);
        assert!(kv.key[] == tuple.val0());
        assert!(kv.value[] == tuple.val1());
    }
}
