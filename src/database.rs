use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt;

#[derive(Debug)]
pub struct Database {
     db: HashMap<String, DatabaseVal>,
}

impl Database {
    pub fn init() -> Self {
        let map = HashMap::with_capacity(10000);
        Database { db: map }
    }

    /*
     * This is a `generic` function as is that will insert _any_
     * DatabaseVal type into our Database but lacks any logic 
     * for updating complex types like sets.
     */
    pub fn update(&mut self, key: &str, val: DatabaseVal) -> () {
        let _option = self.db.insert(key.to_owned(), val);
    }

    pub fn update_string(&mut self, key: &str, val: &str) -> () {
        let v = DatabaseVal::StringVal(StringValType::from(val));
        self.update(key, v)
    }

    /*
     * This is split into two functions, one being it's private companion
     * `try_update_set` below, because we cannot hold a mut ref (db.get_mut(key))
     * and also write to the HashMap in the same function - though we only write they 
     * val if it doesn't exist. 
     * Thus, we have a helper that can try to update a key if it exists, and if not
     * we will update it here.
     */
    pub fn update_or_insert_set(&mut self, key: &str, vals: &Vec<&str>) -> () {
        match self.try_update_set(key, vals) {
            Ok(()) => (),
            Err(()) => {
                let v = DatabaseVal::SetVal(SetValType::from(vals));
                self.update(key, v)
            }
        }
    }
    fn try_update_set(&mut self, key: &str, vals: &Vec<&str>) -> Result<(), ()> {
        match self.db.get_mut(key) {
            Some(DatabaseVal::SetVal(v)) => {
                v.add(vals);
                Ok(())
            },
            _ => Err(()),
        }
    }

    pub fn get(&self, key: &str) -> Option<&DatabaseVal> {
        self.db.get(key)
    }

    /*
     * Should this just pass the Option back up to del_command?
     */
    pub fn delete(&mut self, key: &str) -> Result<(), DatabaseError> {
        match self.db.remove(key) {
            Some(_) => Ok(()),
            None => Err(DatabaseError("key not found".to_string()))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseError(String);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for DatabaseError {
    fn description(&self) -> &str {
        return "Database error"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug)]
pub enum DatabaseVal {
    StringVal(StringValType),
    SetVal(SetValType),
}

impl DatabaseVal {
    pub fn to_string(&self) -> String {
        match self {
            DatabaseVal::StringVal(t) => t.to_string(),
            DatabaseVal::SetVal(t) => t.to_string(),
        }
    }
}

impl fmt::Display for DatabaseVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let stringy = match self {
            DatabaseVal::StringVal(t) => t.to_string(),
            DatabaseVal::SetVal(t) => t.to_string(),
        };
        write!(f, "{}", &stringy)
    }
}

#[derive(Debug)]
pub struct StringValType {
    inner: String,
}

impl StringValType {
    pub fn from(val: &str) -> Self {
        StringValType {
            inner: val.to_owned(),
        }
    }
    pub fn to_string(&self) -> String {
        self.inner.clone()
    }
}

impl fmt::Display for StringValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug)]
pub struct SetValType {
    inner: HashSet<String>,
}

impl SetValType {
    pub fn from(vals: &Vec<&str>) -> Self {
        let mut set: HashSet<String> = HashSet::with_capacity(vals.len());
        for val in vals.iter() {
            set.insert(val.to_string());
        }

        SetValType {
            inner: set,
        }
    }

    pub fn add(&mut self, vals: &Vec<&str>) -> () {
        for v in vals.iter() {
            let _bool = self.inner.insert(v.to_string());
        }
    }

    pub fn to_string(&self) -> String {
        let iter_1 = self.inner.iter();
        let iter_2 = self.inner.iter();
        let t_size = iter_1.fold(0, |acc, s| acc + s.len());
        let ret_string = iter_2.fold(
            String::with_capacity(t_size), 
                |mut acc, s| {
                    acc.push_str(&" ");
                    acc.push_str(&s);
                    acc
                }
            );
        ret_string
    }
}

impl fmt::Display for SetValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}