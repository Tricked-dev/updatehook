# updatehook

A simple program to run a command when a repository gets mentioned in the json body made for github!

```toml
# ~/.hook/config.toml
port = 3000
path = "/updates"
[[project]]
repo = "tricked-dev/diplo"
command = "ls"
```
