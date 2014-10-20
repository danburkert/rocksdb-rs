#![allow(non_camel_case_types,dead_code)]

use libc::{c_char, c_double, c_int, c_uchar, c_uint, c_void, size_t};

#[repr(C)]
pub struct rocksdb_t;
#[repr(C)]
pub struct rocksdb_cache_t;
#[repr(C)]
pub struct rocksdb_compactionfilter_t;
#[repr(C)]
pub struct rocksdb_compactionfiltercontext_t;
#[repr(C)]
pub struct rocksdb_compactionfilterfactory_t;
#[repr(C)]
pub struct rocksdb_compactionfilterv2_t;
#[repr(C)]
pub struct rocksdb_compactionfilterfactoryv2_t;
#[repr(C)]
pub struct rocksdb_comparator_t;
#[repr(C)]
pub struct rocksdb_env_t;
#[repr(C)]
pub struct rocksdb_fifo_compaction_options_t;
#[repr(C)]
pub struct rocksdb_filelock_t;
#[repr(C)]
pub struct rocksdb_filterpolicy_t;
#[repr(C)]
pub struct rocksdb_flushoptions_t;
#[repr(C)]
pub struct rocksdb_iterator_t;
#[repr(C)]
pub struct rocksdb_logger_t;
#[repr(C)]
pub struct rocksdb_mergeoperator_t;
#[repr(C)]
pub struct rocksdb_options_t;
#[repr(C)]
pub struct rocksdb_block_based_table_options_t;
#[repr(C)]
pub struct rocksdb_randomfile_t;
#[repr(C)]
pub struct rocksdb_readoptions_t;
#[repr(C)]
pub struct rocksdb_seqfile_t;
#[repr(C)]
pub struct rocksdb_slicetransform_t;
#[repr(C)]
pub struct rocksdb_snapshot_t;
#[repr(C)]
pub struct rocksdb_writablefile_t;
#[repr(C)]
pub struct rocksdb_writebatch_t;
#[repr(C)]
pub struct rocksdb_writeoptions_t;
#[repr(C)]
pub struct rocksdb_universal_compaction_options_t;
#[repr(C)]
pub struct rocksdb_livefiles_t;
#[repr(C)]
pub struct rocksdb_column_family_handle_t;

#[link(name = "rocksdb")]
#[link(name = "snappy")]
extern {

    /* DB operations */
    pub fn rocksdb_open(options: *const rocksdb_options_t,
                        name: *const c_char,
                        errptr: *mut *mut c_char)
                        -> *mut rocksdb_t;
    pub fn rocksdb_open_for_read_only(options: *const rocksdb_options_t,
                                      name: *const c_char,
                                      error_if_log_file_exist: *const c_char,
                                      errptr: *mut *mut c_char)
                                      -> *mut rocksdb_t;
    pub fn rocksdb_open_column_families(options: *const rocksdb_options_t,
                                        name: *const c_char,
                                        num_column_families: c_int,
                                        column_family_names: *const *const c_char,
                                        column_family_options: *const *const rocksdb_options_t,
                                        column_family_handles: *mut *mut rocksdb_column_family_handle_t,
                                        errptr: *mut *mut c_char)
                                        -> *mut rocksdb_t;
    pub fn rocksdb_open_for_read_only_column_families(options: *const rocksdb_options_t,
                                                      name: *const c_char,
                                                      num_column_families: c_int,
                                                      column_family_names: *const *const c_char,
                                                      column_family_options: *const *const rocksdb_options_t,
                                                      column_fmamily_handles: *mut *mut rocksdb_column_family_handle_t,
                                                      error_if_log_file_exist: *const c_char,
                                                      errptr: *mut *mut c_char)
                                                      -> *mut rocksdb_t;
    pub fn rocksdb_list_column_families(options: *const rocksdb_options_t,
                                        name: *const c_char,
                                        num_column_families: *mut size_t,
                                        errptr: *mut *mut c_char)
                                        -> *mut *mut c_char;
    pub fn rocksdb_list_column_families_destroy(column_families: *mut *mut c_char,
                                                num_column_families: size_t);
    pub fn rocksdb_create_column_family(database: *mut rocksdb_t,
                                        column_family_options: *const rocksdb_options_t,
                                        column_family_name: *const c_char,
                                        errptr: *mut *mut c_char)
                                        -> *mut rocksdb_column_family_handle_t;
    pub fn rocksdb_drop_column_family(database: *mut rocksdb_t,
                                      column_family_handle: *mut rocksdb_column_family_handle_t,
                                      errptr: *mut *mut c_char);
    pub fn rocksdb_column_family_handle_destroy(column_family_handle: *mut rocksdb_column_family_handle_t);
    pub fn rocksdb_close(database: *mut rocksdb_t);
    pub fn rocksdb_put(database: *mut rocksdb_t,
                       options: *const rocksdb_writeoptions_t,
                       key: *const c_char, key_len: size_t,
                       val: *const c_char, val_len: size_t,
                       errptr: *mut *mut c_char);
    pub fn rocksdb_put_cf(database: *mut rocksdb_t,
                          options: *const rocksdb_writeoptions_t,
                          column_family: *mut rocksdb_column_family_handle_t,
                          key: *const c_char, key_len: size_t,
                          val: *const c_char, val_len: size_t,
                          errptr: *mut *mut c_char);
    pub fn rocksdb_delete(database: *mut rocksdb_t,
                          options: *const rocksdb_writeoptions_t,
                          key: *const c_char, key_len: size_t,
                          errptr: *mut *mut c_char);
    pub fn rocksdb_delete_cf(database: *mut rocksdb_t,
                             options: *const rocksdb_writeoptions_t,
                             column_family: *mut rocksdb_column_family_handle_t,
                             key: *const c_char, key_len: size_t,
                             errptr: *mut *mut c_char);
    pub fn rocksdb_merge(database: *mut rocksdb_t,
                         options: *const rocksdb_writeoptions_t,
                         key: *const c_char, key_len: size_t,
                         val: *const c_char, val_len: size_t,
                         errptr: *mut *mut c_char);
    pub fn rocksdb_merge_cf(database: *mut rocksdb_t,
                            options: *const rocksdb_writeoptions_t,
                            column_family: *mut rocksdb_column_family_handle_t,
                            key: *const c_char, key_len: size_t,
                            val: *const c_char, val_len: size_t,
                            errptr: *mut *mut c_char);
    pub fn rocksdb_write(
        database: *mut rocksdb_t,
        options: *const rocksdb_writeoptions_t,
        batch: *mut rocksdb_writebatch_t,
        errptr: *mut *mut c_char);
    /* Returns NULL if not found.  A malloc()ed array otherwise.
       Stores the length of the array in *vallen. */
    pub fn rocksdb_get(database: *mut rocksdb_t,
                       options: *const rocksdb_readoptions_t,
                       key: *const c_char, key_len: size_t,
                       val_len: *mut size_t,
                       errptr: *mut *mut c_char)
                       -> *mut c_char;
    pub fn rocksdb_get_cf(database: *mut rocksdb_t,
                          options: *const rocksdb_readoptions_t,
                          column_family: *mut rocksdb_column_family_handle_t,
                          key: *const c_char, key_len: size_t,
                          val_len: *mut size_t,
                          errptr: *mut *mut c_char)
                          -> *mut c_char;
    pub fn rocksdb_create_iterator(database: *mut rocksdb_t,
                                   options: *const rocksdb_readoptions_t)
                                   -> *mut rocksdb_iterator_t;
    pub fn rocksdb_create_iterator_cf(database: *mut rocksdb_t,
                                      options: *const rocksdb_readoptions_t,
                                      column_family: *mut rocksdb_column_family_handle_t)
                                      -> *mut rocksdb_iterator_t;
    pub fn rocksdb_create_snapshot(database: *mut rocksdb_t) -> *const rocksdb_snapshot_t;
    pub fn rocksdb_release_snapshot(database: *mut rocksdb_t,
                                    snapshot: *const rocksdb_snapshot_t);
    /* Returns NULL if property name is unknown.
       Else returns a pointer to a malloc()-ed null-terminated value. */
    pub fn rocksdb_property_value(database: *mut rocksdb_t,
                                  property: *const c_char);
    pub fn rocksdb_property_value_cf(database: *mut rocksdb_t,
                                     column_family: *mut rocksdb_column_family_handle_t,
                                     property: *const c_char)
                                     -> *mut char;
    pub fn rocksdb_approximate_sizes(database: *mut rocksdb_t,
                                     num_ranges: c_int,
                                     range_start_key: *const *const c_char, range_start_key_len: *const size_t,
                                     range_limit_key: *const *const c_char, range_limit_key_len: *const size_t,
                                     sizes: *mut u64);
    pub fn rocksdb_approximate_sizes_cf(database: *mut rocksdb_t,
                                        column_family: *mut rocksdb_column_family_handle_t,
                                        num_ranges: c_int,
                                        range_start_key: *const *const c_char, range_start_key_len: *const size_t,
                                        range_limit_key: *const *const c_char, range_limit_key_len: *const size_t,
                                        sizes: *mut u64);
    pub fn rocksdb_compact_range(database: *mut rocksdb_t,
                                 start_key: *const c_char, start_key_len: *const size_t,
                                 limit_key: *const c_char, limit_key_len: *const size_t);
    pub fn rocksdb_compact_range_cf(database: *mut rocksdb_t,
                                    column_family: *mut rocksdb_column_family_handle_t,
                                    start_key: *const c_char, start_key_len: size_t,
                                    limit_key: *const c_char, limit_key_len: size_t);
    pub fn rocksdb_delete_file(database: *mut rocksdb_t,
                               name: *const c_char);
    pub fn rocksdb_livefiles(database: *mut rocksdb_t) -> *mut rocksdb_livefiles_t;
    pub fn rocksdb_flush(database: *mut rocksdb_t,
                         options: *const rocksdb_flushoptions_t,
                         errptr: *mut *mut c_char);
    pub fn rocksdb_disable_file_deletions(database: *mut rocksdb_t,
                                          errptr: *mut *mut c_char);
    pub fn rocksdb_enable_file_deletions(database: *mut rocksdb_t,
                                         force: c_uchar,
                                         errptr: *mut *mut c_char);

    /* Management operations */
    pub fn rocksdb_destroy_db(options: *const rocksdb_options_t,
                              name: *const c_char,
                              errptr: &mut *const c_char);
    pub fn rocksdb_repair_db(options: *const rocksdb_options_t,
                             name: *const c_char,
                             errptr: &mut *const c_char);

    /* Iterator */
    pub fn rocksdb_iter_destroy(itr: *mut rocksdb_iterator_t);
    pub fn rocksdb_iter_valid(itr: *const rocksdb_iterator_t) -> c_uchar;
    pub fn rocksdb_iter_seek_to_first(itr: *mut rocksdb_iterator_t);
    pub fn rocksdb_iter_seek_to_last(itr: *mut rocksdb_iterator_t);
    pub fn rocksdb_iter_seek(itr: *mut rocksdb_iterator_t,
                             key: *const c_char,
                             key_len: size_t);
    pub fn rocksdb_iter_next(itr: *mut rocksdb_iterator_t);
    pub fn rocksdb_iter_prev(itr: *mut rocksdb_iterator_t);
    pub fn rocksdb_iter_key(itr: *const rocksdb_iterator_t,
                            len: *mut size_t)
                            -> *const c_char;
    pub fn rocksdb_iter_value(itr: *const rocksdb_iterator_t,
                              len: *mut size_t)
                              -> *const c_char;
    pub fn rocksdb_iter_get_error(itr: *const rocksdb_iterator_t,
                                  errptr: &mut *const c_char);

    /* Write batch */
    pub fn rocksdb_writebatch_create() -> *mut rocksdb_writebatch_t;
    pub fn rocksdb_writebatch_create_from(rep: *const c_char,
                                          size: size_t)
                                          -> *mut rocksdb_writebatch_t;
    pub fn rocksdb_writebatch_destroy(batch: *mut rocksdb_writebatch_t);
    pub fn rocksdb_writebatch_clear(batch: *mut rocksdb_writebatch_t);
    pub fn rocksdb_writebatch_count(batch: *mut rocksdb_writebatch_t) -> c_int;
    pub fn rocksdb_writebatch_put(batch: *mut rocksdb_writebatch_t,
                                  key: *const c_char, key_len: size_t,
                                  val: *const c_char, val_len: size_t);
    pub fn rocksdb_writebatch_put_cf(batch: *mut rocksdb_writebatch_t,
                                     column_family: *mut rocksdb_column_family_handle_t,
                                     key: *const c_char, key_len: size_t,
                                     val: *const c_char, val_len: size_t);
    pub fn rocksdb_writebatch_merge(batch: *mut rocksdb_writebatch_t,
                                    key: *const c_char, key_len: size_t,
                                    val: *const c_char, val_len: size_t);
    pub fn rocksdb_writebatch_merge_cf(batch: *mut rocksdb_writebatch_t,
                                       column_family: *mut rocksdb_column_family_handle_t,
                                       key: *const c_char, key_len: size_t,
                                       val: *const c_char, val_len: size_t);
    pub fn rocksdb_writebatch_delete(batch: *mut rocksdb_writebatch_t,
                                     key: *const char, key_len: size_t);
    pub fn rocksdb_writebatch_delete_cf(batch: *mut rocksdb_writebatch_t,
                                        column_family: *mut rocksdb_column_family_handle_t,
                                        key: *const c_char, key_len: size_t);
    //pub fn rocksdb_writebatch_iterate(batch: *mut rocksdb_writebatch_t,
                                      //void* state,
                                      //void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
                                      //void (*deleted)(void*, const char* k, size_t klen));
    pub fn rocksdb_writebatch_data(batch: *mut rocksdb_writebatch_t,
                                   size: size_t)
                                   -> *const c_char;

    /* Block based table options */
    pub fn rocksdb_block_based_options_create() -> *mut rocksdb_block_based_table_options_t;
    pub fn rocksdb_block_based_options_destroy(options: *mut rocksdb_block_based_table_options_t);
    pub fn rocksdb_block_based_options_set_block_size(options: *mut rocksdb_block_based_table_options_t,
                                                      block_size: size_t);
    pub fn rocksdb_block_based_options_set_block_size_deviation(options: *mut rocksdb_block_based_table_options_t,
                                                                block_size_deviation: c_int);
    pub fn rocksdb_block_based_options_set_block_restart_interval(options: *mut rocksdb_block_based_table_options_t,
                                                                  block_restart_interval: c_int);
    pub fn rocksdb_block_based_options_set_filter_policy(options: *mut rocksdb_block_based_table_options_t,
                                                         filter_policy: *mut rocksdb_filterpolicy_t);
    pub fn rocksdb_block_based_options_set_no_block_cache(options: *mut rocksdb_block_based_table_options_t,
                                                          no_block_cache: c_uchar);
    pub fn rocksdb_block_based_options_set_block_cache(options: *mut rocksdb_block_based_table_options_t,
                                                       block_cache: *mut rocksdb_cache_t);
    pub fn rocksdb_block_based_options_set_block_cache_compressed(options: *mut rocksdb_block_based_table_options_t,
                                                                  block_cache_compressed: *mut rocksdb_cache_t);
    pub fn rocksdb_block_based_options_set_whole_key_filtering(options: *mut rocksdb_block_based_table_options_t,
                                                               whole_key_filtering: c_uchar);
    pub fn rocksdb_options_set_block_based_table_factory(options: *mut rocksdb_options_t,
                                                         table_options: *mut rocksdb_block_based_table_options_t);

    /* Options */
    pub fn rocksdb_options_create() -> *mut rocksdb_options_t;
    pub fn rocksdb_options_destroy(options: *mut rocksdb_options_t);
    pub fn rocksdb_options_increase_parallelism(options: *mut rocksdb_options_t,
                                                total_threads: c_int);
    pub fn rocksdb_options_optimize_for_point_lookup(options: *mut rocksdb_options_t,
                                                     block_cache_size_mb: u64);
    pub fn rocksdb_options_optimize_level_style_compaction(options: *mut rocksdb_options_t,
                                                           memtable_memory_budget: u64);
    pub fn rocksdb_options_optimize_universal_style_compaction(options: *mut rocksdb_options_t,
                                                               memtable_memory_budget: u64);
    pub fn rocksdb_options_set_compaction_filter(options: *mut rocksdb_options_t,
                                                 compaction_filter: *mut rocksdb_compactionfilter_t);
    pub fn rocksdb_options_set_compaction_filter_factory(options: *mut rocksdb_options_t,
                                                         compaction_filter_factory: *mut rocksdb_compactionfilterfactory_t);
    pub fn rocksdb_options_set_compaction_filter_factory_v2(options: *mut rocksdb_options_t,
                                                            compaction_filter_factory_v2: *mut rocksdb_compactionfilterfactoryv2_t);
    pub fn rocksdb_options_set_comparator(options: *mut rocksdb_options_t,
                                          comparator: *mut rocksdb_comparator_t);
    pub fn rocksdb_options_set_merge_operator(options: *mut rocksdb_options_t,
                                              merge_operator: *mut rocksdb_mergeoperator_t);
    pub fn rocksdb_options_set_compression_per_level(options: *mut rocksdb_options_t,
                                                     level_values: c_int,
                                                     num_levels: size_t);
    pub fn rocksdb_options_set_create_if_missing(options: *mut rocksdb_options_t,
                                                 create_if_missing: c_uchar);
    pub fn rocksdb_options_set_create_missing_column_families(options: *mut rocksdb_options_t,
                                                              create_missing_column_families: c_uchar);
    pub fn rocksdb_options_set_error_if_exists(options: *mut rocksdb_options_t,
                                               error_if_exists: c_uchar);
    pub fn rocksdb_options_set_paranoid_checks(options: *mut rocksdb_options_t,
                                               paranoid_checks: c_uchar);
    pub fn rocksdb_options_set_env(options: *mut rocksdb_options_t,
                                   env: *mut rocksdb_env_t);
    pub fn rocksdb_options_set_info_log(options: *mut rocksdb_options_t,
                                        logger: *mut rocksdb_logger_t);
    pub fn rocksdb_options_set_info_log_level(options: *mut rocksdb_options_t,
                                              info_log_level: c_int);
    pub fn rocksdb_options_set_write_buffer_size(options: *mut rocksdb_options_t,
                                                 write_buffer_size: size_t);
    pub fn rocksdb_options_set_max_open_files(options: *mut rocksdb_options_t,
                                              max_open_files: c_int);
    // TODO: figure out what these options are
    pub fn rocksdb_options_set_compression_options(options: *mut rocksdb_options_t,
                                                   val1: c_int,
                                                   val2: c_int,
                                                   val3: c_int);
    pub fn rocksdb_options_set_prefix_extractor(options: *mut rocksdb_options_t,
                                                slice_transform: *mut rocksdb_slicetransform_t);
    pub fn rocksdb_options_set_num_levels(options: *mut rocksdb_options_t,
                                          num_levels: c_int);
    pub fn rocksdb_options_set_level0_file_num_compaction_trigger(options: *mut rocksdb_options_t,
                                                                  level0_file_num_compaction_trigger: c_int);
    pub fn rocksdb_options_set_level0_slowdown_writes_trigger(options: *mut rocksdb_options_t,
                                                              level0_slowdown_writes_trigger: c_int);
    pub fn rocksdb_options_set_level0_stop_writes_trigger(options: *mut rocksdb_options_t,
                                                          level0_stop_writes_trigger: c_int);
    pub fn rocksdb_options_set_max_mem_compaction_level(options: *mut rocksdb_options_t,
                                                        max_mem_compaction_level: c_int);
    pub fn rocksdb_options_set_target_file_size_base(options: *mut rocksdb_options_t,
                                                     file_size_base: u64);
    pub fn rocksdb_options_set_target_file_size_multiplier(options: *mut rocksdb_options_t,
                                                           file_size_multiplier: u64);
    pub fn rocksdb_options_set_max_bytes_for_level_base(options: *mut rocksdb_options_t,
                                                        max_bytes_for_level_base: u64);
    pub fn rocksdb_options_set_max_bytes_for_level_multiplier(options: *mut rocksdb_options_t,
                                                              max_bytes_for_level_multiplier: c_int);
    pub fn rocksdb_options_set_expanded_compaction_factor(options: *mut rocksdb_options_t,
                                                          expanded_compaction_factor: c_int);
    pub fn rocksdb_options_set_max_grandparent_overlap_factor(options: *mut rocksdb_options_t,
                                                              max_grandparent_overlap_factor: c_int);
    pub fn rocksdb_options_set_max_bytes_for_level_multiplier_additional(options: *mut rocksdb_options_t,
                                                                         level_values: *mut c_int,
                                                                         num_levels: size_t);
    pub fn rocksdb_options_enable_statistics(options: *mut rocksdb_options_t);
    pub fn rocksdb_options_set_max_write_buffer_number(options: *mut rocksdb_options_t,
                                                       max_write_buffer_number: c_int);
    pub fn rocksdb_options_set_min_write_buffer_number_to_merge(options: *mut rocksdb_options_t,
                                                                min_write_buffer_number_to_merge: c_int);
    pub fn rocksdb_options_set_max_background_compactions(options: *mut rocksdb_options_t,
                                                          max_background_compactions: c_int);
    pub fn rocksdb_options_set_max_background_flushes(options: *mut rocksdb_options_t,
                                                      max_background_flushes: c_int);
    pub fn rocksdb_options_set_max_log_file_size(options: *mut rocksdb_options_t,
                                                 max_log_file_size: size_t);
    pub fn rocksdb_options_set_log_file_time_to_roll(options: *mut rocksdb_options_t,
                                                     log_file_time_to_roll: size_t);
    pub fn rocksdb_options_set_keep_log_file_num(options: *mut rocksdb_options_t,
                                                 keep_log_file_num: size_t);
    pub fn rocksdb_options_set_soft_rate_limit(options: *mut rocksdb_options_t,
                                               soft_rate_limit: c_double);
    pub fn rocksdb_options_set_hard_rate_limit(options: *mut rocksdb_options_t,
                                               hard_rate_limit: c_double);
    pub fn rocksdb_options_set_rate_limit_delay_max_milliseconds(options: *mut rocksdb_options_t,
                                                                 rate_limit_delay_max_ms: c_uint);
    pub fn rocksdb_options_set_max_manifest_file_size(options: *mut rocksdb_options_t,
                                                      max_manifest_file_size: size_t);
    pub fn rocksdb_options_set_no_block_cache(options: *mut rocksdb_options_t,
                                              no_block_cache: c_uchar);
    pub fn rocksdb_options_set_table_cache_numshardbits(options: *mut rocksdb_options_t,
                                                        table_cache_numshardbits: c_int);
    pub fn rocksdb_options_set_table_cache_remove_scan_count_limit(options: *mut rocksdb_options_t,
                                                                   table_cache_remove_scan_count_limit: c_int);
    pub fn rocksdb_options_set_arena_block_size(options: *mut rocksdb_options_t,
                                                arena_block_size: size_t);
    pub fn rocksdb_options_set_use_fsync(options: *mut rocksdb_options_t,
                                         use_fsync: c_int);
    pub fn rocksdb_options_set_db_log_dir(options: *mut rocksdb_options_t,
                                          sb_log_dir: *const c_char);
    pub fn rocksdb_options_set_wal_dir(options: *mut rocksdb_options_t,
                                       wal_dir: *const c_char);
    pub fn rocksdb_options_set_WAL_ttl_seconds(options: *mut rocksdb_options_t,
                                               WAL_ttl_seconds: u64);
    pub fn rocksdb_options_set_WAL_size_limit_MB(options: *mut rocksdb_options_t,
                                                 WAL_size_limit_MB: u64);
    pub fn rocksdb_options_set_manifest_preallocation_size(options: *mut rocksdb_options_t,
                                                           manifest_preallocation_size: u64);
    pub fn rocksdb_options_set_purge_redundant_kvs_while_flush(options: *mut rocksdb_options_t,
                                                               purge_redundant_kvs_while_flush: c_uchar);
    pub fn rocksdb_options_set_allow_os_buffer(options: *mut rocksdb_options_t,
                                               allow_os_buffer: c_uchar);
    pub fn rocksdb_options_set_allow_mmap_reads(options: *mut rocksdb_options_t,
                                                allow_mmap_reads: c_uchar);
    pub fn rocksdb_options_set_allow_mmap_writes(options: *mut rocksdb_options_t,
                                                 allow_mmap_writes: c_uchar);
    pub fn rocksdb_options_set_is_fd_close_on_exec(options: *mut rocksdb_options_t,
                                                   is_fd_close_on_exec: c_uchar);
    pub fn rocksdb_options_set_skip_log_error_on_recovery(options: *mut rocksdb_options_t,
                                                          skip_log_error_on_recovery: c_uchar);
    pub fn rocksdb_options_set_stats_dump_period_sec(options: *mut rocksdb_options_t,
                                                     stats_dump_period_sec: c_uint);
    pub fn rocksdb_options_set_block_size_deviation(options: *mut rocksdb_options_t,
                                                    block_size_deviation: c_int);
    pub fn rocksdb_options_set_advise_random_on_open(options: *mut rocksdb_options_t,
                                                     advise_random_on_open: c_uchar);
    pub fn rocksdb_options_set_access_hint_on_compaction_start(options: *mut rocksdb_options_t,
                                                               access_hint_on_compaction_start: c_int);
    pub fn rocksdb_options_set_use_adaptive_mutex(options: *mut rocksdb_options_t,
                                                  use_adaptive_mutex: c_uchar);
    pub fn rocksdb_options_set_bytes_per_sync(options: *mut rocksdb_options_t,
                                              bytes_per_sync: u64);
    pub fn rocksdb_options_set_verify_checksums_in_compaction(options: *mut rocksdb_options_t,
                                                              verify_checksums_in_compaction: c_uchar);
    pub fn rocksdb_options_set_filter_deletes(options: *mut rocksdb_options_t,
                                              filter_deletes: c_uchar);
    pub fn rocksdb_options_set_max_sequential_skip_in_iterations(options: *mut rocksdb_options_t,
                                                                 max_sequential_skip_in_iterations: u64);
    pub fn rocksdb_options_set_disable_data_sync(options: *mut rocksdb_options_t,
                                                 disable_data_sync: c_int);
    pub fn rocksdb_options_set_disable_auto_compactions(options: *mut rocksdb_options_t,
                                                        disable_auto_compactions: c_int);
    pub fn rocksdb_options_set_delete_obsolete_files_period_micros(options: *mut rocksdb_options_t,
                                                                   delete_obsolete_files_period_micros: u64);
    pub fn rocksdb_options_set_source_compaction_factor(options: *mut rocksdb_options_t,
                                                        set_source_compaction_factor: c_int);
    pub fn rocksdb_options_prepare_for_bulk_load(options: *mut rocksdb_options_t);
    pub fn rocksdb_options_set_memtable_vector_rep(options: *mut rocksdb_options_t);
    // TODO: figure out what these options are
    pub fn rocksdb_options_set_hash_skip_list_rep(options: *mut rocksdb_options_t,
                                                  val1: size_t,
                                                  val2: c_int,
                                                  val3: c_int );
    pub fn rocksdb_options_set_hash_link_list_rep(options: *mut rocksdb_options_t,
                                                  val: size_t);
    pub fn rocksdb_options_set_plain_table_factory(options: *mut rocksdb_options_t,
                                                   val1: c_uint,
                                                   val2: c_int,
                                                   val3: c_double,
                                                   val4: size_t);
    pub fn rocksdb_options_set_min_level_to_compress(options: *mut rocksdb_options_t,
                                                     min_level_to_compress: c_int);
    pub fn rocksdb_options_set_memtable_prefix_bloom_bits(options: *mut rocksdb_options_t,
                                                          memtable_prefix_bloom_bits: c_uint);
    pub fn rocksdb_options_set_memtable_prefix_bloom_probes(options: *mut rocksdb_options_t,
                                                            memtable_prefix_bloom_probes: c_uint);
    pub fn rocksdb_options_set_max_successive_merges(options: *mut rocksdb_options_t,
                                                     max_successive_merges: size_t);
    pub fn rocksdb_options_set_min_partial_merge_operands(options: *mut rocksdb_options_t,
                                                          min_partial_merge_operands: c_uint);
    pub fn rocksdb_options_set_bloom_locality(options: *mut rocksdb_options_t,
                                              bloom_locality: c_uint);
    pub fn rocksdb_options_set_allow_thread_local(options: *mut rocksdb_options_t,
                                                  allow_thread_local: c_uchar);
    pub fn rocksdb_options_set_inplace_update_support(options: *mut rocksdb_options_t,
                                                      inplace_update_support: c_uchar);
    pub fn rocksdb_options_set_inplace_update_num_locks(options: *mut rocksdb_options_t,
                                                        inplace_update_num_locks: size_t);

    /* Comparator */
    pub fn rocksdb_comparator_create(state: *mut c_void,
                                     destructor: extern fn(*mut c_void),
                                     comparator: extern fn(*mut c_void,
                                                           *const c_char, size_t,
                                                           *const c_char, size_t)
                                                           -> c_int,
                                     name: extern fn(*mut c_void) -> *const c_char)
                                     -> *mut rocksdb_comparator_t;
    pub fn rocksdb_comparator_destroy(comparator: *mut rocksdb_comparator_t);

    /* Merge Operator */
    pub fn rocksdb_mergeoperator_create(state: *mut c_void,
                                        destructor: extern fn(*mut c_void),
                                        full_merge: extern fn(*mut c_void,                         // state
                                                              *const c_char, size_t,               // key
                                                              *const c_char, size_t,               // existing value
                                                              *const *const c_char, *const size_t, // operand list
                                                              c_int,                               // number of operands
                                                              *mut c_uchar, *mut size_t)           // success, result length
                                                              -> *mut c_char,                      // result
                                        partial_merge: extern fn(*mut c_void,                         // state
                                                                 *const c_char, size_t,               // key
                                                                 *const *const c_char, *const size_t, // operand list
                                                                 c_int,                               // number of operands
                                                                 *mut c_uchar, *mut size_t)           // success, result length
                                                                 -> *mut c_char,                      // result
                                        delete_value: extern fn(*mut c_void,
                                                                *const c_char, size_t),
                                        name: extern fn(*mut c_void) -> *const c_char)
                                        -> *mut rocksdb_mergeoperator_t;
    pub fn rocksdb_mergeoperator_destroy(operator: *mut rocksdb_mergeoperator_t);

    /* Read options */
    pub fn rocksdb_readoptions_create() -> *mut rocksdb_readoptions_t;
    pub fn rocksdb_readoptions_destroy(options: *mut rocksdb_readoptions_t);
    pub fn rocksdb_readoptions_set_verify_checksums(options: *mut rocksdb_readoptions_t,
                                                    verify_checksums: c_uchar);
    pub fn rocksdb_readoptions_set_fill_cache(options: *mut rocksdb_readoptions_t,
                                              fill_cache: c_uchar);
    pub fn rocksdb_readoptions_set_snapshot(options: *mut rocksdb_readoptions_t,
                                            snapshot: *const rocksdb_snapshot_t);
    pub fn rocksdb_readoptions_set_read_tier(options: *mut rocksdb_readoptions_t,
                                             read_tier: c_int);
    pub fn rocksdb_readoptions_set_tailing(options: *mut rocksdb_readoptions_t,
                                           tailing: c_uchar);

    /* Write options */
    pub fn rocksdb_writeoptions_create() -> *mut rocksdb_writeoptions_t;
    pub fn rocksdb_writeoptions_destroy(options: *mut rocksdb_writeoptions_t);
    pub fn rocksdb_writeoptions_set_sync(options: *mut rocksdb_writeoptions_t,
                                         sync: c_uchar);
    pub fn rocksdb_writeoptions_disable_WAL(options: *mut rocksdb_writeoptions_t,
                                            disable: c_int);

    /* Flush options */
    pub fn rocksdb_flushoptions_create() -> *mut rocksdb_flushoptions_t;
    pub fn rocksdb_flushoptions_destroy(options: *mut rocksdb_flushoptions_t);
    pub fn rocksdb_flushoptions_set_wait(options: *mut rocksdb_flushoptions_t,
                                         wait: c_uchar);

    /* Cache */
    pub fn rocksdb_cache_create_lru(capacity: size_t) -> *mut rocksdb_cache_t;
    pub fn rocksdb_cache_destroy(cache: *mut rocksdb_cache_t);

    /* SliceTransform */

    pub fn rocksdb_slicetranform_create(state: *mut c_void,
                                        destructor: extern fn(*mut c_void),
                                        transform: extern fn(*mut c_void,           // state
                                                             *const c_char, size_t, // key
                                                             *mut size_t)           // prefix length
                                                             -> *mut c_char,        // result
                                        in_domain: extern fn(*mut c_void,           // state
                                                             *const c_char, size_t) // key
                                                             -> c_uchar,           // result
                                        in_range: extern fn(*mut c_void,           // state
                                                            *const c_char, size_t) // key
                                                            -> c_uchar,            // result
                                        name: extern fn(*mut c_void) -> *const c_char)
                                        -> *mut rocksdb_slicetransform_t;
}
