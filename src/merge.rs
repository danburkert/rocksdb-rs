use super::MergeOperator;

/// The simpler, associative merge operator.
pub trait AssociativeMergeOperator: Sync + Send {
    fn merge(&self, key: &[u8], existing_val: Vec<u8>, operand: &[u8]) -> Option<Vec<u8>>;
}

impl<T: AssociativeMergeOperator> MergeOperator for T {
    fn full_merge(&self,
                  key: &[u8],
                  existing_val: Option<&[u8]>,
                  operands: Vec<&[u8]>)
                  -> Option<Vec<u8>> {
        let mut operands = operands.into_iter();

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
                     operands: Vec<&[u8]>)
                     -> Option<Vec<u8>> {
        let mut operands = operands.into_iter();

        let base: Option<Vec<u8>> = operands.next().map(|val| val.to_vec());

        operands.fold(base, |existing, operand| {
            existing.and_then(|existing| {
                self.merge(key, existing, operand)
            })
        })
    }
}

pub struct ConcatMergeOperator;

impl MergeOperator for ConcatMergeOperator {

    fn full_merge(&self,
                  _key: &[u8],
                  existing_val: Option<&[u8]>,
                  operands: Vec<&[u8]>)
                  -> Option<Vec<u8>> {
        let cap = existing_val.map(|val| val.len()).unwrap_or(0)
                + operands.iter().fold(0, |acc, elem| acc + elem.len());

        let mut vec = Vec::with_capacity(cap);

        for val in existing_val.into_iter() {
            vec.push_all(val);
        }

        for operand in operands.into_iter() {
            vec.push_all(operand);
        }

        Some(vec)
    }

    fn partial_merge(&self,
                     _key: &[u8],
                     operands: Vec<&[u8]>)
                     -> Option<Vec<u8>> {
        let cap = operands.iter().fold(0, |acc, elem| acc + elem.len());
        let mut vec = Vec::with_capacity(cap);
        for operand in operands.into_iter() {
            vec.push_all(operand);
        }
        Some(vec)
    }
}

pub struct AssociativeConcat;

impl AssociativeMergeOperator for AssociativeConcat {

    fn merge(&self, key: &[u8], mut existing_val: Vec<u8>, operand: &[u8]) -> Option<Vec<u8>> {
        existing_val.push_all(operand);
        Some(existing_val)
    }
}
