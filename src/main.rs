use reqwest::blocking::RequestBuilder;
use std::env;

mod bot;
mod server;

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

static addr: &str = "127.0.0.1:8001";

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
async fn main() {
    let user_name = env::var("JIRA_USER_NAME").expect("JIRA_USER_NAME");
    let api_token = env::var("JIRA_API_TOKEN").expect("JIRA_API_TOKEN");
    let slack_token = env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN");

    let auth = BasicAuth {
        user: user_name,
        api_token: api_token,
    };

    let jira = bot::Jira::new(Box::new(auth));
    let slack = bot::Slack::new(slack_token).unwrap();

    let cfg = server::Config {
        jira: jira,
        slack: slack,
    };

    let s = server::Server::new(cfg);
    s.run(addr.to_string()).await.unwrap();
}
