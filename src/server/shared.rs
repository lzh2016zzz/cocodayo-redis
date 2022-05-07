use rocksdb::{BoundColumnFamily, ColumnFamily, Options, DB as Rocksdb};
use std::{path::Path, sync::Arc};

use crate::{server::value_ref::Value, utils};

pub struct Shared {
    database: Rocksdb,
    options: Options,
}

impl Shared {
    pub fn new(append_file: &str) -> Shared {
        let path = Path::new(append_file);
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
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
        let iterator = self.database.iterator(rocksdb::IteratorMode::Start);
        let mut values = Vec::new();
        let mut skip = skip;
        for (i, (key, _)) in iterator.enumerate() {
            if skip > 0 {
                skip -= 1;
            } else {
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


    pub fn sets_set(&mut self, key: &str, value: Value) -> crate::Result<()> {
        let value = value.as_slice();
        self.set_with_sub_key_internal(key, value, b"")?;
        Ok(())
    }

    pub fn hash_set(&mut self,key : &str, sub_key: &str,value: Value) -> crate::Result<()> {
        let value = value.as_slice();
        self.set_with_sub_key_internal(key, sub_key.as_bytes(), value)?;
        Ok(())
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

//private method implementation
impl Shared {
    fn set_with_sub_key_internal(
        &mut self,
        key: &str,
        sub_key: &[u8],
        value: &[u8],
    ) -> crate::Result<Option<()>> {
        let column_family = self.get_column_family(key)?;
        match self.database.put_cf(&column_family, sub_key, value) {
            Ok(()) => Ok(Some(())),
            Err(err) => return Err(err.into()),
        }
    }

    fn get_column_family(&self, key: &str) -> crate::Result<Arc<BoundColumnFamily>> {
        match self.database.cf_handle(key) {
            Some(col) => Ok(col),
            None => match self.database.create_cf(key, &self.options) {
                Ok(_) => match self.database.cf_handle(key) {
                    Some(value) => Ok(value),
                    None => return Err("Some errors occurred in get column family".into()),
                },
                Err(err) => return Err(err.into()),
            },
        }
    }
}
