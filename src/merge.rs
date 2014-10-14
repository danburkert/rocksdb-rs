use std::{mem, raw, slice};
use std::io::{BufReader, MemWriter};
use std::kinds::marker;

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
                  -> Option<Vec<u8>>;

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
                     -> Option<Vec<u8>>;
}


/// The simpler, associative merge operator.
pub trait AssociativeMergeOperator: Sync + Send {
    fn merge(&self, key: &[u8], existing_val: Vec<u8>, operand: &[u8]) -> Option<Vec<u8>>;
}

impl<T: AssociativeMergeOperator> MergeOperator for T {
    fn full_merge(&self,
                  key: &[u8],
                  existing_val: Option<&[u8]>,
                  mut operands: Operands)
                  -> Option<Vec<u8>> {
        let base: Option<Vec<u8>> = existing_val.map(|val| val.to_vec())
                                                .or_else(|| operands.next().map(|val| val.to_vec()));
        operands.fold(base, |existing, operand| {
            existing.and_then(|existing| {
                self.merge(key, existing, operand)
            })
        })
    }

    fn partial_merge(&self,
                     key: &[u8],
                     mut operands: Operands)
                     -> Option<Vec<u8>> {
        let base: Option<Vec<u8>> = operands.next().map(|val| val.to_vec());
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

    #[doc(hidden)]
    pub fn new(operands: *const *const u8,
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

impl<'a> ExactSize<&'a [u8]> for Operands<'a> {}

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

pub struct ConcatMergeOperator;

impl MergeOperator for ConcatMergeOperator {

    fn full_merge(&self,
                  _key: &[u8],
                  existing_val: Option<&[u8]>,
                  mut operands: Operands)
                  -> Option<Vec<u8>> {
        let cap = existing_val.map(|val| val.len()).unwrap_or(0)
                + operands.clone().fold(0, |acc, elem| acc + elem.len());

        let mut vec = Vec::with_capacity(cap);

        for val in existing_val.into_iter() {
            vec.push_all(val);
        }

        for operand in operands {
            vec.push_all(operand);
        }

        Some(vec)
    }

    fn partial_merge(&self,
                     _key: &[u8],
                     mut operands: Operands)
                     -> Option<Vec<u8>> {
        let cap = operands.clone().fold(0, |acc, elem| acc + elem.len());
        let mut vec = Vec::with_capacity(cap);
        for operand in operands {
            vec.push_all(operand);
        }
        Some(vec)
    }
}

pub struct AddMergeOperator;

impl AddMergeOperator {

    pub fn read_u64(bytes: &[u8]) -> Option<u64> {
        let mut reader = BufReader::new(bytes);
        match reader.read_be_u64() {
            Ok(val) => {
                if reader.eof() {
                    Some(val)
                } else {
                    error!("More than 8 bytes provided to `read_u64`: {}.", bytes);
                    None
                }
            },
            Err(error) => {
                error!("Encountered error {} when reading existing value {}.", error, bytes);
                None
            }
        }
    }

    pub fn write_u64(value: u64) -> Option<Vec<u8>> {
        let mut writer = MemWriter::with_capacity(8);
        match writer.write_be_u64(value) {
            Ok(_) => Some(writer.unwrap()),
            Err(error) => {
                error!("Encountered error {} when writing value {}.", error, value);
                None
            }
        }
    }
}

impl AssociativeMergeOperator for AddMergeOperator {
    fn merge(&self,
             _key: &[u8],
             existing_val: Vec<u8>,
             operand: &[u8])
             -> Option<Vec<u8>> {
        match (AddMergeOperator::read_u64(existing_val.as_slice()),
               AddMergeOperator::read_u64(operand)) {
            (Some(existing), Some(operand)) => AddMergeOperator::write_u64(existing + operand),
            _ => None
        }
    }
}
