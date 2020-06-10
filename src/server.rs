use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use super::bot::{PostJiraInput, PostJiraToSlack};

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

#[derive(Serialize, Deserialize, Debug)]
pub struct CallRequest {
    channel: String,
    jql: String,
    at: String,
}

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

pub async fn call(data: Json<CallRequest>, bot: Data<Arc<Mutex<PostJiraToSlack>>>) -> HttpResponse {
    let encoded_jql = utf8_percent_encode(&data.jql, FRAGMENT).to_string();
    println!("{}", encoded_jql);
    let guard = bot.lock().unwrap();
    guard
        .do_action(PostJiraInput {
            jql: encoded_jql.replace("=", "%3D"),
            slack_channel: data.channel.clone(),
            message: data.at.clone(),
        })
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(format!("ok"))
}
