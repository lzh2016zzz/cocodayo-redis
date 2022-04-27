use std::{path::Path};
use crate::server::value_ref::ValueRef;
use db_key::Key;
use leveldb::{database::Database, options::Options};
use leveldb::kv::KV;
use leveldb::options::{WriteOptions, ReadOptions};


pub struct Shared {
    database : Database<DbKey>
}

pub enum DbKey {
    Ok(String),
    Err
}

impl Key for DbKey {
    fn from_u8(key: &[u8]) -> Self {
       match String::from_utf8(key.to_vec()){
            Ok(s) =>  DbKey::Ok(s),
            Err(_) => DbKey::Err,
        }
    }
    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        match self {
            DbKey::Ok(s) => f(s.as_bytes()),
            DbKey::Err => f(b"error"),
        }
    }
}

impl From<String> for DbKey {
    fn from(src: String) -> DbKey {
        DbKey::Ok(src)
    }
}




impl Shared {
    pub fn new(append_file : &str) -> Shared {
        let path = Path::new(append_file);
        let mut options = Options::new();
        options.create_if_missing = true;
        let database = match Database::open(path, options) {
            Ok(db) => { db },
            Err(e) => { panic!("failed to open database: {:?}", e) }
        };
        Shared { database }
    }

    pub fn len(&self) -> usize {
       return 0
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
            let write_opts = WriteOptions::new();
            match self.database.put(write_opts, &key.to_string().into(), value.as_slice()) {
                Ok(_) => Ok(Some(())),
                Err(e) => { Err(e.into()) }
            }
        } else {
            return Ok(None);
        }
    }   

    pub fn get(&self, key: &str) -> Option<ValueRef> {
        let opt = ReadOptions::new();
        match self.database.get(opt,&key.to_string().into()) {
            Ok(Some(s)) => Some(ValueRef::from_u8(s)),
            Ok(None) => None,
            Err(err) => {
                log::error!("an error occurred while determining get,key = {} err = {}",key,err);
                None
            },
        }
    }

    pub fn get_mut(&mut self,key:&str) -> Option<&mut ValueRef> {
       todo!()
    }

    pub fn del(&mut self, key: &str) -> i8 {
        let write_opts = WriteOptions::new();
        match self.database.delete(write_opts, &key.to_string().into()){
            Ok(_) => 1,
            Err(err) => -1,
        }
    }
    
    pub fn flush(&mut self) {
       
    }

    pub fn is_exists(& self, key: &str) -> bool {
        let opt = ReadOptions::new();
        match self.database.get(opt, &key.to_string().into()){
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(err) => {
                log::error!("an error occurred while determining is_exists,key = {} err = {}",key,err);
                false
            },
        }
    }

}
