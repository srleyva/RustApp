// extern crate recommendation_service;

// use recommendation_service::recommendation::{
//     User
// };

// use env_logger::init;
// use log::{
//     info
// };

// use std::fs;
// use serde_json::{
//     from_str,
//     Value
// };

// #[tokio::main]
// async fn main() {
//     init();
//     info!("loading users");
//     let users = fs::read_to_string("./seed/seed.txt").unwrap();
//     let users: Value = from_str(users.as_str()).unwrap();

//     let users: Vec<User> = users
//             .as_array()
//             .unwrap()
//             .iter()
//             .map(|h| {
//                 User {
//                     first_name: h["first_name"].to_string(),
//                     last_name: h["last_name"].to_string(),
//                     uid: h["uid"].to_string(),
//                     age: h["age"]
//                 }
//             })
//             .collect();

// }