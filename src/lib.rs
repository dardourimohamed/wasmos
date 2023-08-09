pub mod gql;
pub mod sql;

pub use serde::{self, Deserialize, Serialize};
pub use serde_json;
pub use tokio;
pub use riwaq_macro::*;

#[no_mangle]
extern "C" fn str_malloc(capacity: u64) -> *const u8 {
    let s = String::with_capacity(capacity as _);
    let ptr = s.as_ptr();
    std::mem::forget(s);
    ptr
}

extern "C" {
    pub fn riwaq_dbg(ptr: *const u8);
}

#[macro_export]
macro_rules! wdbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        unsafe { riwaq::riwaq_dbg(format!("[{}:{}]\0", file!(), line!()).as_ptr()) }
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                unsafe { riwaq::riwaq_dbg(format!("[{}:{}] {} = {:#?}\0",
                    file!(), line!(), stringify!($val), &tmp).as_ptr()); }
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::riwaqdbg!($val)),+,)
    };
}
