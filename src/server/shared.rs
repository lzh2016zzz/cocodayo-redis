use std::{path::Path};
use rocksdb::{DB as Rocksdb, Options};

use crate::server::value_ref::ValueRef;

pub struct Shared {
    database : Rocksdb,
    append_file : String,
}

impl Shared {
    pub fn new(append_file : &str) -> Shared {
        let path = Path::new(append_file);
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let database = match Rocksdb::open(&opts, path) {
            Ok(some) => some,
            Err(err) => panic!("failed to initialize shared database,{}",err),
        };

        Shared { database,append_file : append_file.to_string() }
    }

    pub fn len(&self) -> usize {
       return 0
    }

    pub fn set_default(&mut self,key: &str,value: ValueRef) -> crate::Result<Option<()>> {
        self.set(key, value, false, false)
    }
    
    pub fn set(
        &mut self,
        key: &str,
        value: ValueRef,
        nx: bool,
        px: bool,
    ) -> crate::Result<Option<()>> {
        let exists = self.is_exists(key);
        if (nx && !exists) || (px && exists) || (!nx && !px) {
            match self.database.put(key.as_bytes(), value.as_slice()) {
                Ok(_) => Ok(Some(())),
                Err(e) => { Err(e.into()) }
            }
        } else {
            return Ok(None);
        }
    }   

    pub fn get(&self, key: &str) -> Option<ValueRef> {

         match self.database.get(key.as_bytes()) {
            Ok(Some(s)) => Some(ValueRef::from_u8(s.to_vec())),
            Ok(None) => Some(ValueRef::None),
            Err(err) => {
                log::error!("an error occurred while determining get,key = {} err = {}",key,err);
                None
            },
        }
    }


    pub fn del(&mut self, key: &str) -> i8 {
        match self.database.delete(key.as_bytes()) {
            Ok(()) => 1,
            Err(err) => {
                log::error!("an error occurred while determining delete,key = {} err = {}",key,err);
                0
            },
        }
    }
    
    pub fn flush(&mut self) -> crate::Result<()> {
        let path = Path::new(&self.append_file);
        let opt = Options::default();
        match Rocksdb::destroy(&opt, path) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn is_exists(& self, key: &str) -> bool {
        match self.get(key) {
            Some(_) => true,
            None => false,
        }
    }

}
