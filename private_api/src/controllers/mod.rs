use constata_lib::prelude::*;
use rocket::{ get, post, serde::json::Json, State };

pub mod sessions;
pub mod api;
pub mod react_admin;

pub type JsonResult<T> = ConstataResult<Json<T>>;
