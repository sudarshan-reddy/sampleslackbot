use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::bot::PostJiraToSlack;

#[derive(Serialize, Deserialize, Debug)]
pub struct CallRequest {
    channel: Vec<String>,
}

//pub fn call(data: Json<CallRequest>, _bot: Data<Arc<Mutex<PostJiraToSlack>>>) -> HttpResponse {

pub async fn call(data: Json<CallRequest>) -> HttpResponse {
    println!("called");
    println!("{:?}", data);
    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("ok"))
}
