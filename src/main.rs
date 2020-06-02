use chrono::{DateTime, Utc};
use reqwest::{blocking::Client, blocking::RequestBuilder, Method};
use serde::{Deserialize, Serialize};
use slack::chat;
use slack_api as slack;
use std::cmp::Ordering;
use std::{env, fmt, result::Result, vec};

fn main() {
    let user_name = env::var("JIRA_USER_NAME").unwrap();
    let api_token = env::var("JIRA_API_TOKEN").unwrap();
    let slack_token = env::var("SLACK_BOT_TOKEN").unwrap();
    let slack_channel = env::var("SLACK_CHANNEL").unwrap();

    let auth = BasicAuth {
        user: user_name,
        api_token: api_token,
    };
    let b = Jira::new(Box::new(auth));
    let issues = b
        // TODO: currently only works for MBE AWAITING REVIEW. Add
        // new rules and an ability to switch between.
        .get_jira_issues(JQL_MBE_AWAITING_REVIEW.to_string())
        .unwrap();

    let bot = Slack::new(slack_token).unwrap();

    bot.post_message(slack_channel, &issues).unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    #[serde(rename = "startAt")]
    start_at: i32,
    issues: Vec<Issue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Issue {
    key: String,
    fields: Fields,
}

#[derive(Serialize, Deserialize, Debug)]
struct Fields {
    summary: String,
    updated: DateTime<Utc>,
    labels: Vec<String>,
    priority: Priority,
}

#[derive(Serialize, Deserialize, Debug)]
struct Priority {
    id: String,
}

#[derive(Default)]
struct IssueReport {
    issue_link: String,
    summary: String,
    easy_label: String,
    days: i64,
    priority: i64,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut issue_list = String::new();

        let mut issues = vec::Vec::new();
        for issue in &self.issues {
            issues.push(IssueReport {
                issue_link: format!("https://zalora.atlassian.net/browse/{}", issue.key),
                summary: issue.fields.summary.clone(),
                priority: issue.fields.priority.id.parse::<i64>().unwrap(),
                easy_label: "".to_string(),
                days: Utc::now()
                    .signed_duration_since(issue.fields.updated)
                    .num_days(),
            });
        }

        issues.sort_by(|a, b| match a.priority.cmp(&b.priority) {
            Ordering::Equal => b.days.cmp(&a.days),
            other => other,
        });

        for issue in issues {
            issue_list.push_str(&format!(
                "{}: {} ({}days)\n",
                issue.issue_link, issue.summary, issue.days,
            ))
        }
        write!(
            f,
            r##"@mbe-devs: the following issues need attention.
```{}```
"##,
            issue_list,
        )
    }
}

static JQL_MBE_AWAITING_REVIEW: &str =
    "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

trait Authorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder;
}

struct BasicAuth {
    user: String,
    api_token: String,
}

impl Authorizer for BasicAuth {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.basic_auth(self.user.to_owned(), Some(self.api_token.to_owned()))
    }
}

struct Jira {
    client: Client,
    authorizer: Box<dyn Authorizer>,
}

impl Jira {
    fn new(authorizer: Box<dyn Authorizer>) -> Self {
        Jira {
            client: Client::new(),
            authorizer: authorizer,
        }
    }

    fn get_jira_issues(&self, jql: String) -> Result<Response, Error> {
        let url = format!("https://zalora.atlassian.net/rest/api/3/search?jql={}", jql);
        let req = self.client.request(Method::GET, &url);
        let res = self.authorizer.authorize_request(req).send()?;
        let text = res.text()?;
        let resp: Response = serde_json::from_str(&text)?;
        Ok(resp)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

struct Slack {
    token: String,
    client: reqwest::blocking::Client,
}

impl Slack {
    fn new(token: String) -> Result<Self, Error> {
        let client = slack::default_client()?;
        let s = Slack {
            token: token,
            client: client,
        };
        Ok(s)
    }

    fn post_message(&self, channel: String, response: &Response) -> Result<(), Error> {
        chat::post_message(
            &self.client,
            &self.token,
            &chat::PostMessageRequest {
                channel: &channel,
                text: &response.to_string(),
                link_names: Some(true),
                ..chat::PostMessageRequest::default()
            },
        )?;
        Ok(())
    }
}
