use rocksdb::{Options, DB as Rocksdb, WriteOptions, WriteBatch, SliceTransform, ReadOptions};
use std::{path::Path};

use crate::{server::value::Value, utils};



pub struct Shared {
    database: Rocksdb,
    options: Options,
}

impl Shared {
    pub fn new(append_file: &str) -> Shared {
        let path = Path::new(append_file);
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let slice_transform = SliceTransform::create_noop();
        opts.set_prefix_extractor(slice_transform);
        let database = match Rocksdb::open(&opts, path) {
            Ok(some) => some,
            Err(err) => panic!("failed to initialize shared database,{}", err),
        };

        Shared {
            database,
            options: opts,
        }
    }

    pub fn scan_for(&self, pattern: Option<&str>, skip: i32, cnt: usize) -> (Vec<Value>, usize) {
        let iterator = self
            .database
            .iterator(rocksdb::IteratorMode::Start)
            .enumerate();
        let mut values = Vec::new();

        let skip = if skip < 0 { 0 } else { skip as usize };

        for (i, (key, _)) in iterator.skip(skip) {
            if let Some(pattern) = pattern {
                if pattern == "*" {
                    values.push(Value::Bytes(key.to_vec()));
                } else {
                    let pattern = pattern.as_bytes();
                    if utils::backtrack_match(&key, pattern) {
                        values.push(Value::Bytes(key.to_vec()));
                    }
                }
            } else {
                values.push(Value::Bytes(key.to_vec()));
            }
            if cnt > 0 && cnt == values.len() {
                return (values, i);
            }
        }
        let len = values.len();
        (values, len)
    }

    pub fn len(&self) -> usize {
        return 0;
    }

    pub fn default_set(&mut self, key: &str, value: Value) -> crate::Result<Option<()>> {
        self.set(key, value, false)
    }


    pub fn set(&mut self, key: &str, value: Value, nx: bool) -> crate::Result<Option<()>> {
        if (nx && self.is_exists(key)) || !nx {
            match self.database.put(key.as_bytes(), value.as_slice()) {
                Ok(_) => Ok(Some(())),
                Err(e) => Err(e.into()),
            }
        } else {
            return Ok(None);
        }
    }


    pub fn get(&self, key: &str) -> Option<Value> {
        match self.database.get(key.as_bytes()) {
            Ok(Some(s)) => Some(Value::from_u8(s.to_vec())),
            Ok(None) => Some(Value::None),
            Err(err) => {
                log::error!(
                    "an error occurred while determining get,key = {} err = {}",
                    key,
                    err
                );
                None
            }
        }
    }

    pub fn del(&mut self, key: &str) -> i8 {
        match self.database.delete(key.as_bytes()) {
            Ok(()) => 1,
            Err(err) => {
                log::error!(
                    "an error occurred while determining delete,key = {} err = {}",
                    key,
                    err
                );
                0
            }
        }
    }

    pub fn flush(&mut self) -> crate::Result<()> {
        Err("invalid operation".into())
    }

    pub fn is_exists(&self, key: &str) -> bool {
        match self.get(key) {
            Some(_) => true,
            None => false,
        }
    }
}

//bound set operations
impl Shared {

    pub fn sets_set(&mut self, key: &str, values: Vec<Value>) -> crate::Result<()> {


        todo!()
    }

    pub fn sets_iterator(&self,key :&str,skip :i32) -> crate::Result<Vec<Value>> {

        todo!()
    }

    pub fn sets_get(&self, key :&str) -> Option<Value> {
        let opt = ReadOptions::default();
        todo!()
    }


}

//bound hash operations {
impl Shared {
    
    pub fn hash_set(&mut self, key: &str, sub_key: &str, value: Value) -> crate::Result<()> {
        todo!()
    }
}

//private method implementation
impl Shared {
    
    fn set_with_sub_key_internal_batch(&mut self,batch :WriteBatch) -> crate::Result<()>{

        match self.database.write_opt(batch, &WriteOptions::default()) {
            Ok(()) => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}
