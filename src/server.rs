use actix_web::{
    get, web,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::io;

use super::bot::{Action, Jira, PostJiraInput, PostJiraToSlack, Slack};

pub struct Config {
    pub jira: Jira,
    pub slack: Slack,
}

pub struct Server {
    jira: Jira,
    slack: Slack,
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        Server {
            jira: cfg.jira,
            slack: cfg.slack,
        }
    }

    pub async fn run(&self, addr: String) -> io::Result<()> {
        let post_jira_to_slack = PostJiraToSlack::new(self.jira.clone(), self.slack.clone());
        HttpServer::new(move || {
            App::new()
                .data(post_jira_to_slack.clone())
                .service(web::resource("/invoke").route(web::post().to(call)))
        })
        .bind(addr)?
        .run()
        .await
    }
}

#[derive(Deserialize, Debug)]
struct CallRequest {
    channels: Vec<String>,
}

fn call(body: Json<CallRequest>, bot: Data<Box<dyn Action<PostJiraInput>>>) -> HttpResponse {
    println!("{:?}", body);
    HttpResponse::Ok().body(format!("ok"))
}
