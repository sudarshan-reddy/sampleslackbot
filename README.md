# SimpleSlackBot 

Just a POC for a simple server based application in Rust.


# To build

This is builty using Rust. You need cargo to build it.

```bash
cargo build
```

# To run

The following environment variables are required

```bash
EXPORT JIRA_USER_NAME="your jira login id"
EXPORT JIRA_API_TOKEN="your jira api token"
EXPORT SLACK_BOT_TOKEN="your slackbot/app's api token"
EXPORT SLACK_CHANNEL="channel the message should go to"
```

```bash
cargo run
```



