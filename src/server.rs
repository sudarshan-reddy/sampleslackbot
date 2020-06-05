use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use serde::Deserialize;

use super::bot::{Action, PostJiraInput};

#[derive(Deserialize, Debug)]
pub struct CallRequest {
    channels: Vec<String>,
}

pub fn call(body: Json<CallRequest>, bot: Data<Box<dyn Action<PostJiraInput>>>) -> HttpResponse {
    println!("{:?}", body);
    HttpResponse::Ok().body(format!("ok"))
}
