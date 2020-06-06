use actix_web::{
    web::{Data, Json},
    HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::bot::{PostJiraInput, PostJiraToSlack};

#[derive(Serialize, Deserialize, Debug)]
pub struct CallRequest {
    channel: String,
}

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

pub async fn call(data: Json<CallRequest>, bot: Data<Arc<Mutex<PostJiraToSlack>>>) -> HttpResponse {
    let guard = bot.lock().unwrap();
    guard
        .do_action(PostJiraInput {
            jql: JQL_MBE_AWAITING_REVIEW.to_string(),
            slack_channel: data.channel.clone(),
        })
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("ok"))
}
