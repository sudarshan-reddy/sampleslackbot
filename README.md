# DontSlack

Is a slack bot that tells mbe devs (for now) what issues are awaiting review

It sorts them by their priority and number of days since the last update.

# To build

This is builty using Rust. You need cargo to build it.

```bash
cargo build
```

# To run

The following environment variables are required

```bash
EXPORT JIRA_USER_NAME="your zalora email id"
EXPORT JIRA_API_TOKEN="your jira api token"
EXPORT SLACK_BOT_TOKEN="your slackbot/app's api token"
EXPORT SLACK_CHANNEL="channel the message should go to"
```

```bash
cargo run
```



