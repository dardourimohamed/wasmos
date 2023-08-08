use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TableDDLOp {
    Keep,
    Drop,
    DropAll,
    Undrop,
    Rename(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DDLOp {
    Keep,
    Drop,
    Rename(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldDDL {
    pub name: String,
    pub opt: bool,
    pub ty: String,
    pub op: DDLOp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableDDL {
    pub name: String,
    pub cols: Vec<FieldDDL>,
    pub op: TableDDLOp,
}
