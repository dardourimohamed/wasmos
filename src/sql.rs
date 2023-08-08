use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::Value;

extern "C" {
    fn ext_sql_exec(ptr: *const u8) -> *const u8;
    fn ext_sql_query(ptr: *const u8) -> *const u8;
}

#[derive(Serialize)]
pub struct DBConn {
    pub url: String,
    pub db_name: String,
}

pub trait Col {
    fn sql_name(&self) -> String;
}

#[derive(Deserialize)]
struct SqlRes {
    ok: bool,
    msg: Option<String>,
    data: Option<Value>,
}

pub async fn sql_exec(cmd: Value) -> Result<Value, String> {
    tokio::task::spawn(async move {
        let req_ptr = format!("{}\0", serde_json::to_string(&cmd).unwrap()).as_ptr();
        let res_ptr = unsafe { ext_sql_exec(req_ptr) };
        let res_str = unsafe {
            std::ffi::CString::from_raw(res_ptr as _)
                .into_string()
                .unwrap()
        };
        let res = serde_json::from_str::<SqlRes>(&res_str).unwrap();
        if res.ok {
            Ok(res.data.unwrap())
        } else {
            Err(res.msg.unwrap())
        }
    })
    .await
    .unwrap()
}
pub async fn sql_query(cmd: Value) -> Result<Value, String> {
    tokio::task::spawn(async move {
        let req_ptr = format!("{}\0", serde_json::to_string(&cmd).unwrap()).as_ptr();
        let res_ptr = unsafe { ext_sql_query(req_ptr) };
        let res_str = unsafe {
            std::ffi::CString::from_raw(res_ptr as _)
                .into_string()
                .unwrap()
        };
        let res = serde_json::from_str::<SqlRes>(&res_str).unwrap();
        if res.ok {
            Ok(res.data.unwrap())
        } else {
            Err(res.msg.unwrap())
        }
    })
    .await
    .unwrap()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FilterItem {
    Eq {
        col: String,
        value: Value,
    },
    Ne {
        col: String,
        value: Value,
    },
    In {
        col: String,
        values: Vec<Value>,
    },
    Nin {
        col: String,
        values: Vec<Value>,
    },
    Gt {
        col: String,
        value: Value,
    },
    Gte {
        col: String,
        value: Value,
    },
    Lt {
        col: String,
        value: Value,
    },
    Lte {
        col: String,
        value: Value,
    },
    Between {
        col: String,
        start: Value,
        end: Value,
    },
    Like {
        col: String,
        expr: String,
    },
    IsNull {
        col: String,
    },
    IsNotNull {
        col: String,
    },
}
impl Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterItem::Eq { col, value } => {
                f.write_str(&format!("{} == {}", col, sql_render_value(value)))
            }
            FilterItem::Ne { col, value } => {
                f.write_str(&format!("{} <> {}", col, sql_render_value(value)))
            }
            FilterItem::In { col, values } => f.write_str(&format!(
                "{} in ({})",
                col,
                values
                    .iter()
                    .map(sql_render_value)
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
            FilterItem::Nin { col, values } => f.write_str(&format!(
                "{} in ({})",
                col,
                values
                    .iter()
                    .map(sql_render_value)
                    .collect::<Vec<String>>()
                    .join(", ")
            )),
            FilterItem::Gt { col, value } => {
                f.write_str(&format!("{} > {}", col, sql_render_value(value)))
            }
            FilterItem::Gte { col, value } => {
                f.write_str(&format!("{} >= {}", col, sql_render_value(value)))
            }
            FilterItem::Lt { col, value } => {
                f.write_str(&format!("{} < {}", col, sql_render_value(value)))
            }
            FilterItem::Lte { col, value } => {
                f.write_str(&format!("{} <= {}", col, sql_render_value(value)))
            }
            FilterItem::Between { col, start, end } => f.write_str(&format!(
                "{} between ({}, {})",
                col,
                sql_render_value(start),
                sql_render_value(end)
            )),
            FilterItem::Like { col, expr } => f.write_str(&format!("{} like {}", col, expr)),
            FilterItem::IsNull { col } => f.write_str(&format!("{} IS NULL", col)),
            FilterItem::IsNotNull { col } => f.write_str(&format!("{} IS NOT NULL", col)),
        }
    }
}

pub fn sql_render_value(value: &Value) -> String {
    match value {
        Value::Null => "NULL".into(),
        Value::Bool(b) => format!("{}", b),
        Value::Number(n) => format!("{}", n),
        Value::String(s) => format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'")),
        Value::Array(a) => format!(
            "( {} )",
            a.into_iter()
                .map(sql_render_value)
                .collect::<Vec<String>>()
                .join(", ")
        ),
        Value::Object(obj) => format!(
            "( {} )",
            obj.into_iter()
                .map(|o| format!("{} AS {}", sql_render_value(o.1), o.0))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    }
}

pub trait SQLFilterTrait {
    fn get_filter(&self) -> FilterItem;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FilterStmt<T>
where
    T: SQLFilterTrait,
{
    And(Vec<FilterStmt<T>>),
    Or(Vec<FilterStmt<T>>),
    Filter(T),
}

impl<T> Display for FilterStmt<T>
where
    T: SQLFilterTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterStmt::And(filter) => f.write_str(&format!(
                "({})",
                filter
                    .iter()
                    .map(|item| format!("{}", item))
                    .collect::<Vec<String>>()
                    .join(" AND ")
            )),
            FilterStmt::Or(filter) => f.write_str(&format!(
                "({})",
                filter
                    .iter()
                    .map(|item| format!("{}", item))
                    .collect::<Vec<String>>()
                    .join(" OR ")
            )),
            FilterStmt::Filter(filter) => f.write_str(&format!("{}", filter.get_filter())),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Select<T>
where
    T: SQLFilterTrait,
{
    pub op: String,
    pub tbl: String,
    pub cols: Vec<String>,
    pub filter: Option<FilterStmt<T>>,
}
impl<T> Select<T>
where
    T: SQLFilterTrait,
{
    pub fn and(self, filter: T) -> Self {
        let mut m = self;
        m.filter = match m.filter {
            Some(FilterStmt::And(mut f)) => {
                f.push(FilterStmt::Filter(filter));
                Some(FilterStmt::And(f))
            }
            Some(FilterStmt::Or(f)) => Some(FilterStmt::And(vec![
                FilterStmt::Or(f),
                FilterStmt::Filter(filter),
            ])),
            Some(FilterStmt::Filter(f)) => Some(FilterStmt::And(vec![
                FilterStmt::Filter(f),
                FilterStmt::Filter(filter),
            ])),
            None => Some(FilterStmt::Filter(filter)),
        };
        m
    }
    pub fn and_all<VEC>(self, filter: VEC) -> Self
    where
        VEC: IntoIterator<Item = T>,
    {
        let mut m = self;
        m.filter = match m.filter {
            Some(FilterStmt::And(mut f)) => {
                f.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::And(f))
            }
            Some(FilterStmt::Or(f)) => {
                let mut res = vec![FilterStmt::Or(f)];
                res.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::And(res))
            }
            Some(FilterStmt::Filter(f)) => {
                let mut res = vec![FilterStmt::Filter(f)];
                res.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::And(res))
            }
            None => Some(FilterStmt::And(
                filter
                    .into_iter()
                    .map(|item| FilterStmt::Filter(item))
                    .collect(),
            )),
        };
        m
    }
    pub fn or(self, filter: T) -> Self {
        let mut m = self;
        m.filter = match m.filter {
            Some(FilterStmt::Or(mut f)) => {
                f.push(FilterStmt::Filter(filter));
                Some(FilterStmt::Or(f))
            }
            Some(FilterStmt::And(f)) => Some(FilterStmt::Or(vec![
                FilterStmt::And(f),
                FilterStmt::Filter(filter),
            ])),
            Some(FilterStmt::Filter(f)) => Some(FilterStmt::Or(vec![
                FilterStmt::Filter(f),
                FilterStmt::Filter(filter),
            ])),
            None => Some(FilterStmt::Filter(filter)),
        };
        m
    }

    pub fn or_any<VEC>(self, filter: VEC) -> Self
    where
        VEC: IntoIterator<Item = T>,
    {
        let mut m = self;
        m.filter = match m.filter {
            Some(FilterStmt::Or(mut f)) => {
                f.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::Or(f))
            }
            Some(FilterStmt::And(f)) => {
                let mut res = vec![FilterStmt::And(f)];
                res.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::Or(res))
            }
            Some(FilterStmt::Filter(f)) => {
                let mut res = vec![FilterStmt::Filter(f)];
                res.extend(filter.into_iter().map(|item| FilterStmt::Filter(item)));
                Some(FilterStmt::Or(res))
            }
            None => Some(FilterStmt::Or(
                filter
                    .into_iter()
                    .map(|item| FilterStmt::Filter(item))
                    .collect(),
            )),
        };
        m
    }

    pub async fn exec(self) {}
}

impl<T> Display for Select<T>
where
    T: SQLFilterTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tbl = self.tbl.clone();
        let cols = self.cols.join(", ");
        let filter = self
            .filter
            .as_ref()
            .map(|filter| format!("WHERE {}", filter))
            .unwrap_or("".to_string());

        f.write_str(&format!("SELECT {cols} FROM {tbl} {filter};"))
    }
}
