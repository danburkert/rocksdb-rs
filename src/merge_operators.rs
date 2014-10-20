use std::io;

use super::{AssociativeMergeOperator, MergeOperator, Operands};

pub struct ConcatMergeOperator;

impl MergeOperator for ConcatMergeOperator {

    fn full_merge(&self,
                  _key: &[u8],
                  existing_val: Option<&[u8]>,
                  mut operands: Operands)
                  -> io::IoResult<Vec<u8>> {
        let cap = existing_val.map(|val| val.len()).unwrap_or(0)
                + operands.clone().fold(0, |acc, elem| acc + elem.len());

        let mut vec = Vec::with_capacity(cap);

        for val in existing_val.into_iter() {
            vec.push_all(val);
        }

        for operand in operands {
            vec.push_all(operand);
        }

        Ok(vec)
    }

    fn partial_merge(&self,
                     _key: &[u8],
                     mut operands: Operands)
                     -> io::IoResult<Vec<u8>> {
        let cap = operands.clone().fold(0, |acc, elem| acc + elem.len());
        let mut vec = Vec::with_capacity(cap);
        for operand in operands {
            vec.push_all(operand);
        }
        Ok(vec)
    }
}

pub struct AddMergeOperator;

impl AddMergeOperator {

    pub fn read_u64(bytes: &[u8]) -> io::IoResult<u64> {
        io::BufReader::new(bytes).read_be_u64()
    }

    pub fn write_u64(value: u64) -> io::IoResult<Vec<u8>> {
        let mut writer = io::MemWriter::with_capacity(8);
        try!(writer.write_be_u64(value));
        Ok(writer.unwrap())
    }
}

impl AssociativeMergeOperator for AddMergeOperator {
    fn merge(&self,
             _key: &[u8],
             existing_val: Vec<u8>,
             operand: &[u8])
             -> io::IoResult<Vec<u8>> {
        let existing = try!(AddMergeOperator::read_u64(existing_val.as_slice()));
        let operand = try!(AddMergeOperator::read_u64(operand));
        AddMergeOperator::write_u64(existing + operand)
    }
}
