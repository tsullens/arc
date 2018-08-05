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

    pub fn insert(&mut self, key: String, val: DatabaseVal) -> () {
        let _ = self.db.insert(key, val);
        ()
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
    val: String,
}

impl StringValType {
    pub fn from(val: &str) -> Self {
        StringValType {
            val: val.to_owned(),
        }
    }
    pub fn to_string(&self) -> String {
        self.val.clone()
    }
}

impl fmt::Display for StringValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[derive(Debug)]
pub struct SetValType {
    val: HashSet<String>,
}

impl SetValType {
    pub fn from(vals: &Vec<&str>) -> Self {
        let mut set: HashSet<String> = HashSet::with_capacity(vals.len());
        let vals_iter = vals.iter();
        for val in vals_iter {
            set.insert(val.to_string());
        }

        SetValType {
            val: set,
        }
    }

    pub fn to_string(&self) -> String {
        let iter_1 = self.val.iter();
        let iter_2 = self.val.iter();
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
        write!(f, "{:?}", self.val)
    }
}