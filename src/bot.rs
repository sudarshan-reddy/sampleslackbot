use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::{result::Result, vec};

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

pub trait Authorizer: CloneAuthorizer + Send + Sync {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder;
}

impl Clone for Box<dyn Authorizer> {
    fn clone(&self) -> Self {
        self.clone_authorizer()
    }
}

trait CloneAuthorizer {
    fn clone_authorizer(&self) -> Box<dyn Authorizer>;
}

impl<T> CloneAuthorizer for T
where
    T: Authorizer + Clone + 'static,
{
    fn clone_authorizer(&self) -> Box<dyn Authorizer> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct BearerTokenAuthorizer {
    token_key: String,
    token_value: String,
}

impl Authorizer for BearerTokenAuthorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder {
        req.header(reqwest::header::AUTHORIZATION, self.token_value.clone())
    }
}

impl BearerTokenAuthorizer {
    fn new(key: &str, value: String) -> Self {
        let value = format!("Bearer {}", value);
        BearerTokenAuthorizer {
            token_key: key.to_string(),
            token_value: value,
        }
    }
}

#[derive(Clone)]
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

    pub async fn get_jira_issues(&self, jql: String) -> Result<Response, Error> {
        let url = format!("https://zalora.atlassian.net/rest/api/3/search?jql={}", jql);
        let req = self.client.request(Method::GET, &url);
        let res = self.authorizer.authorize_request(req).send().await?;
        let text = res.text().await?;
        let resp: Response = serde_json::from_str(&text)?;
        Ok(resp)
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct Slack {
    token: String,
    client: Client,
    authorizer: Box<dyn Authorizer>,
}

impl Slack {
    pub fn new(token: &str) -> Result<Self, Error> {
        let s = Slack {
            token: token.to_string(),
            client: Client::new(),
            authorizer: Box::new(BearerTokenAuthorizer::new("token", token.to_string())),
        };
        Ok(s)
    }

    pub async fn post_message<Message: Into<String>>(
        &self,
        channel: String,
        msg: Message,
    ) -> Result<(), Error> {
        let url = format!("https://slack.com/api/chat.postMessage");
        let mut req = self.client.request(Method::POST, &url);
        req = self.authorizer.authorize_request(req);

        let mut m = HashMap::new();
        m.insert("channel", channel);
        m.insert("text", msg.into());
        m.insert("link_names", "true".to_string());
        req.json(&m).send().await?;

        Ok(())
    }
}

#[async_trait]
pub trait Action<T>: Send + Sync {
    async fn do_action(&self, input: T) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct PostJiraToSlack {
    jira: Jira,
    slack: Slack,
}

impl PostJiraToSlack {
    pub fn new(jira: Jira, slack: Slack) -> Self {
        return {
            PostJiraToSlack {
                jira: jira,
                slack: slack,
            }
        };
    }
}

#[derive(Clone)]
pub struct PostJiraInput {
    jql: String,
    slack_channel: String,
}

impl PostJiraToSlack {
    pub async fn post_message(&self, input: PostJiraInput) -> Result<(), Error> {
        let mbe_awaiting_review_issues = self.jira.get_jira_issues(input.jql.to_string()).await?;

        self.slack
            .post_message(input.slack_channel, &mbe_awaiting_review_issues)
            .await?;

        Ok(())
    }
}
