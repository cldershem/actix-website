use crate::schema::users;
use diesel::{AsChangeset, Insertable, Queryable};
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Deserialize, Serialize, Queryable, AsChangeset)]
pub struct User {
    #[serde(deserialize_with = "from_str")]
    pub id: i32,
    pub name: String,
}

#[derive(Insertable, Deserialize)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
