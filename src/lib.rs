use std::collections::HashMap;

pub use serde::{self, Deserialize, Serialize};
pub use wasmos_macro::*;
pub use tokio;
pub use serde_json;

#[no_mangle]
pub extern "C" fn str_malloc(capacity: u64) -> *const u8 {
    let s = String::with_capacity(capacity as _);
    let ptr = s.as_ptr();
    std::mem::forget(s);
    ptr
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<T> {
    // pub method: String,
    pub body: T
}

#[derive(Serialize, Deserialize)]
pub enum ObjectFieldType {
    String,
    Number
}

#[derive(Serialize, Deserialize)]
pub struct ObjectMetadata {
    fields: HashMap<String, ObjectFieldType>
}

pub trait ObjectMeta {
    fn metastruct() -> String;
}
