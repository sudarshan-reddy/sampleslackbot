use reqwest::blocking::RequestBuilder;
use std::env;

mod bot;
mod server;

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

pub struct BasicAuth {
    user: String,
    api_token: String,
}

impl bot::Authorizer for BasicAuth {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.basic_auth(self.user.to_owned(), Some(self.api_token.to_owned()))
    }
}

fn main() {
    let user_name = env::var("JIRA_USER_NAME").expect("JIRA_USER_NAME");
    let api_token = env::var("JIRA_API_TOKEN").expect("JIRA_API_TOKEN");
    let slack_token = env::var("SLACK_BOT_TOKEN").expect("SLACK_BOT_TOKEN");
    let slack_channel = env::var("SLACK_CHANNEL").expect("SLACK_CHANNEL");

    let auth = BasicAuth {
        user: user_name,
        api_token: api_token,
    };

    let b = bot::Jira::new(Box::new(auth));
    let mbe_awaiting_review_issues = b
        // TODO: currently only works for MBE AWAITING REVIEW. Add
        // new rules and an ability to switch between.
        .get_jira_issues(JQL_MBE_AWAITING_REVIEW.to_string())
        .unwrap();

    let issues_in_string: String = mbe_awaiting_review_issues.clone().into();
    println!("{}", issues_in_string);

    let bot = bot::Slack::new(slack_token).unwrap();

    //bot.post_message(slack_channel, &mbe_awaiting_review_issues)
    bot.post_message(slack_channel, &mbe_awaiting_review_issues)
        .unwrap();
}
