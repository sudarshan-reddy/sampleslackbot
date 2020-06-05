use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer};
use std::collections::HashMap;
use std::io;

use super::bot::{Action, Jira, PostJiraInput, PostJiraToSlack, Slack};

pub struct ServerConfig {
    jira: Jira,
    slack: Slack,
}

pub struct Server {
    post_jira_to_slack: Box<dyn Action<PostJiraInput>>,
}

impl Server {
    pub fn new(cfg: ServerConfig) -> Self {
        Server {
            post_jira_to_slack: Box::new(PostJiraToSlack::new(cfg.jira, cfg.slack)),
        }
    }

    #[actix_rt::main]
    pub async fn run(&self, addr: String) -> io::Result<()> {
        HttpServer::new(move || App::new().service(web::resource("/").to(call)))
            .bind(addr)?
            .run()
            .await
    }
}

fn call(req: HttpRequest, bot: web::Data<crate::bot::Slack>) -> HttpResponse {
    println!("{:?}", req);

    HttpResponse::Ok().body(format!("ok"))
}
