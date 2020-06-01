use reqwest::Client;
use reqwest::RequestBuilder;
use std::result::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Response{
    startAt: i32,
    issues: Vec<Issue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Issue{
    key: String,
    fields: Fields,
}

#[derive(Serialize, Deserialize, Debug)]
struct Fields{
    summary: String,
    labels: Vec<String>,
}

static JQL_MBE_AWAITING_REVIEW: &str  = "project%20%3D%20%22Mobile%20Backend%22%20and%20status%20%3D%20%22Awaiting%20Review%22";

#[tokio::main]
async fn main() {
    let auth = BasicAuth{
        user: "sudarsan.reddy@zalora.com".to_string(),
        api_token: "NyV68THQTeJeNIHyrRiw8CD9".to_string(),
    };
    let b = Jira::new(Box::new(auth));
    b.get_jira_issues(JQL_MBE_AWAITING_REVIEW.to_string()).await.unwrap();
}

trait Authorizer {
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder;
}

struct BasicAuth {
    user: String,
    api_token: String,
}

impl Authorizer for BasicAuth{
    fn authorize_request(&self, req: RequestBuilder) -> RequestBuilder{
        req.basic_auth(
            self.user.to_owned(),
            Some(self.api_token.to_owned()))
    }
}

struct Jira {
    client: Client,
    authorizer: Box<dyn Authorizer>, 
}

impl Jira {
    fn new(authorizer:  Box<dyn Authorizer>) -> Self {
        Jira {
            client: Client::new(),
            authorizer: authorizer,
        }
    }

    async fn get_jira_issues(&self, jql: String) -> Result<(), Error> {
        let url = format!("https://zalora.atlassian.net/rest/api/3/search?jql={}", jql);
        let req = self.client.get(&url);
        let res = self.authorizer.authorize_request(req).send().await?;
        let text = res.text().await?;
        let resp: Response = serde_json::from_str(&text)?;
        println!("{:?}", resp);
        Ok(())
    }

}

type Error = Box<dyn std::error::Error + Send + Sync> ;

