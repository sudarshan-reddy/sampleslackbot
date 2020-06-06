use bot::PostJiraToSlack;
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::env;
use std::sync::{Arc, Mutex};

use actix_web::{web, web::Data, App, HttpRequest, HttpResponse, HttpServer, Responder};

mod bot;
mod server;

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

static ADDR: &str = "127.0.0.1:8001";

#[derive(Clone)]
pub struct BasicAuth {
    user: String,
    api_token: String,
}

impl bot::Authorizer for BasicAuth {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.basic_auth(self.user.to_owned(), Some(self.api_token.to_owned()))
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let user_name = env::var("JIRA_USER_NAME").expect("JIRA_USER_NAME");
    let api_token = env::var("JIRA_API_TOKEN").expect("JIRA_API_TOKEN");
    let slack_token = env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN");

    let auth = BasicAuth {
        user: user_name,
        api_token: api_token,
    };

    let jira = bot::Jira::new(Box::new(auth));
    let slack = bot::Slack::new(&slack_token).unwrap();

    let post_jira_to_slack = PostJiraToSlack::new(jira.clone(), slack.clone());
    let data = Arc::new(Mutex::new(post_jira_to_slack));
    HttpServer::new(move || {
        App::new()
            //.data(data.clone())
            .service(web::resource("/invoke").route(web::post().to(server::call)))
    })
    .bind(&ADDR)?
    .run()
    .await
}
