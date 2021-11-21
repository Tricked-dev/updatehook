# updatehook

A simple program to run a command when a repository gets mentioned in the json body made for github!

### Setup

```toml
# ~/.hook/config.toml
port = 3000
path = "/updates"
[[project]]
# Repo to be checking for case doesn't matter
repo = "tricked-dev/diplo"
# Command to run
command = "ls"
```

Set up github webhooks to the url and it will work
