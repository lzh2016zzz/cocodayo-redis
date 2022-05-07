use rocksdb::{Options, DB as Rocksdb};
use std::{path::Path, cell::RefCell};

use crate::{server::value_ref::ValueRef, utils};

pub struct Shared {
    database: RefCell<Rocksdb>,
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
            database:RefCell::new(database),
            options: opts,
        }
    }

    pub fn scan_for(&self, pattern: Option<&str>, skip: i32, cnt: usize) -> (Vec<ValueRef>, usize) {

        let database = self.database.borrow();
        let iterator = database.iterator(rocksdb::IteratorMode::Start);
        let mut values = Vec::new();
        let mut skip = skip;
        for (i, (key, _)) in iterator.enumerate() {
            if skip > 0 {
                skip -= 1;
            } else {
                if let Some(pattern) = pattern {
                    if pattern == "*" {
                        values.push(ValueRef::Bytes(key.to_vec()));
                    } else {
                        let pattern = pattern.as_bytes();
                        if utils::backtrack_match(&key, pattern) {
                            values.push(ValueRef::Bytes(key.to_vec()));
                        }
                    }
                } else {
                    values.push(ValueRef::Bytes(key.to_vec()));
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

    pub fn set_default(&mut self, key: &str, value: ValueRef) -> crate::Result<Option<()>> {
        self.set(key, value, false)
    }

    pub fn set(&mut self, key: &str, value: ValueRef, nx: bool) -> crate::Result<Option<()>> {
        if (nx && self.is_exists(key)) || !nx {

            let mut database = self.database.borrow_mut();
            match database.put(key.as_bytes(), value.as_slice()) {
                Ok(_) => Ok(Some(())),
                Err(e) => Err(e.into()),
            }
        } else {
            return Ok(None);
        }
    }

    pub fn get(&self, key: &str) -> Option<ValueRef> {

        let database = self.database.borrow();
        match database.get(key.as_bytes()) {
            Ok(Some(s)) => Some(ValueRef::from_u8(s.to_vec())),
            Ok(None) => Some(ValueRef::None),
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

        let mut database = self.database.borrow_mut();
        match database.delete(key.as_bytes()) {
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

        let database = self.database.borrow();
        let column_family = {
            match database.cf_handle(key) {
                Some(col) => Some(col),
                None => None,
            }
        };
        let mut database = self.database.borrow_mut();
        let column_family = match column_family {
            Some(col) => col,
            None => {
                match database.create_cf(key, &self.options) {
                    Ok(_) => match database.cf_handle(key) {
                        Some(value) => value,
                        None => return Err("Some errors occurred in get column family".into()),
                    },
                    Err(err) => return Err(err.into()),
                }
            },
        };

        match database.put_cf(column_family, sub_key, value) {
            Ok(()) => Ok(Some(())),
            Err(err) => return Err(err.into()),
        }
    }

}
