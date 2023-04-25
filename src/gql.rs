pub use serde::{self, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request<T> {
    // pub method: String,
    pub body: T,
}

pub trait ObjectMeta {
    fn metastruct() -> String;
}
