use serde_derive::{Serialize, Deserialize};

#[derive(Serialize,Deserialize)]
pub struct User {
    pub id: Option<i32>,
    pub name: String,
    pub email: String,
}