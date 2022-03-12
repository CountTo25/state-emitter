use std::sync::RwLock;
use chrono::Utc;
use crate::types::Servable;
use serde::ser::{Serialize, Serializer,SerializeStruct};
use serde::Deserialize;
use crate::types::Incrementer;
use std::any::Any;

pub struct StateContainer<T> where T: Servable {
    pub state: RwLock<T>,
    modified: i64, //maybe to string? TODO: think on it
}


impl StateContainer<Incrementer> { //todo: thats boring, but i need more experience before going into generics here
    pub fn increment(&self) -> Result<i64, String> {
        let lock = self.state.write();  
        match lock {
            Ok(val) => self.updateValue(val),
            Err(e) => Err(format!("Failed unlocking RwLock @ StateContainer<Incrementer>: {}", e)), //duh
        } 
    }

    fn updateValue(&self, mut unlocked: std::sync::RwLockWriteGuard<Incrementer>) -> Result<i64, String> {
        unlocked.increment();
        //self.modified = Utc::now().timestamp();
        return Ok(unlocked.val);
    }
}

pub fn create_state_container<T: Servable>(item: T) -> StateContainer<T> {
    return StateContainer {
        state: RwLock::new(item),
        modified: Utc::now().timestamp()
    }
}

impl Serialize for StateContainer<Incrementer> {
    fn serialize<S: Serializer>(&self, serializer: S)-> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Person", 3)?;
        let val = &*self.state.read().unwrap().toString();

        s.serialize_field("value", &val)?;
        s.serialize_field("modified_at", &self.modified)?;
        s.end()
    }
}