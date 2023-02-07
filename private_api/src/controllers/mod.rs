use constata_lib::{models::Site, error::{Result as MyResult}};
use rocket::{ get, post, serde::json::Json, State, };
use rocket::serde::{Serialize, Deserialize};

pub mod sessions;
pub mod private_graphql;
pub mod react_admin;

pub type JsonResult<T> = MyResult<Json<T>>;
