use chrono::{DateTime, Utc};
use reqwest::{blocking::Client, blocking::RequestBuilder, Method};
use serde::{Deserialize, Serialize};
use slack::chat;
use slack_api as slack;
use std::cmp::Ordering;
use std::{fmt, result::Result, vec};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response {
    #[serde(rename = "startAt")]
    start_at: i32,
    issues: Vec<Issue>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Issue {
    key: String,
    fields: Fields,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Fields {
    summary: String,
    updated: DateTime<Utc>,
    labels: Vec<String>,
    priority: Priority,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl std::convert::From<&Response> for std::string::String {
    fn from(r: &Response) -> Self {
        if r.issues.len() == 0 {
            return format!(
                r##"Great job @mbe-devs . No tickets to review.
                Should we all take a day off?"##
            );
        }

        let mut issue_list = String::new();
        let mut issues = vec::Vec::new();
        for issue in &r.issues {
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
        format!(
            r##"@mbe-devs: the following issues need attention.
```{}```
"##,
            issue_list,
        )
    }
}

pub trait Authorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder;
}

pub struct Jira {
    client: Client,
    authorizer: Box<dyn Authorizer>,
}

impl Jira {
    pub fn new(authorizer: Box<dyn Authorizer>) -> Self {
        Jira {
            client: Client::new(),
            authorizer: authorizer,
        }
    }

    pub fn get_jira_issues(&self, jql: String) -> Result<Response, Error> {
        let url = format!("https://zalora.atlassian.net/rest/api/3/search?jql={}", jql);
        let req = self.client.request(Method::GET, &url);
        let res = self.authorizer.authorize_request(req).send()?;
        let text = res.text()?;
        let resp: Response = serde_json::from_str(&text)?;
        Ok(resp)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct Slack {
    token: String,
    client: reqwest::blocking::Client,
}

impl Slack {
    pub fn new(token: String) -> Result<Self, Error> {
        let client = slack::default_client()?;
        let s = Slack {
            token: token,
            client: client,
        };
        Ok(s)
    }

    pub fn post_message<Response: Into<String>>(
        &self,
        channel: String,
        response: Response,
    ) -> Result<(), Error> {
        chat::post_message(
            &self.client,
            &self.token,
            &chat::PostMessageRequest {
                channel: &channel,
                text: &response.into(),
                link_names: Some(true),
                ..chat::PostMessageRequest::default()
            },
        )?;

        Ok(())
    }
}
