# Qwest

## What is it ?

An http client based on CLI

## How does it works ?

-> Create a new project: `qwest new my_project`

- it opens you default editor (nvim for me)
- it opens a file my_project.toml located in ~/.local/share/.qwest/adventures/my_project.toml
- with an already completed project draft

```toml
[api]
name = "test"
base_url = ""

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"
```

- you can edit this file to change the base_url, the project name, add some requests

## how can I pass variables ?

anywhere the toml will be parsed and replace every placeholders ${my_variable} by the my_variable value.
variables can be set in different places:

- in a .env file that is passed during the execution of the script `qwest run --env-file .env my_project my_route`
- in sqlite file that is located in "~/.local/share/.qwest/qwest.sqlite" this db is passed to the project at anytime
  - they are two levels of variables in this DB: the project_variables column project sets to my_project and the global_variables column variables sets to null
- directly in command line `qwest run my_project my_route -e token=1234`

## what is the schemas of the database ?

as single table: variables: label: str, value: str, project: str

## How can I set variables in the db ?

`qwest set variable my_variable my_value --project my_project`
project is optional

## What are the other commands ?

Run a specific request of a project `qwest run my_project my_route`
Edit a project `qwest edit my_project`
Delete a project `qwest delete my_project`

## Can I run scripts at run time ?

Yes you can set a script instruction in the toml and it will be executed, before or after the request:

```toml
[api]
name = "test"
base_url = "https://my_url.com"

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"
  [[scripts]]
  before = true
  script = "rhai command"
  description = "an optional description shown before the execution of the script"
  [[scripts]]
  before = false
  script = "rhai command"
```

If you return values in the scripts it will be set as variables in the database (usefull for storing tokens as example)

## Can I run multiple requests ?

Yes two ways:
in the api section of the toml you can specify a list of requests to run:

```toml
[api]
name = "test"
base_url = "https://my_url.com"
  [[schenarii]]
  first = ["docs", "login"]
  second = ["login", "docs"]

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"

[[requests]]
name   = "login"
method = "POST"
path   = "/login"
body = """
{"email": "enzo@tantar.ai", "password": "test1234"}
"""
```

second whay, you can set the need option in the requests

```toml
[api]
name = "test"
base_url = "https://my_url.com"
  [[schenarii]]
  first = ["docs", "login"]
  second = ["login", "docs"]

[[requests]]
name   = "docs"
method = "GET"
path   = "/docs"

[[requests]]
name   = "login"
method = "POST"
path   = "/login"
body = """
{"email": "enzo@tantar.ai", "password": "test1234"}
"""

[[requests]]
name   = "user"
method = "GET"
path   = "/user"
headers = """
{}
"""
```

## What are the fields in the toml ?

first level: [api]
name: The name of your project
base_url: The base url of your api (will be concatenated to the whole requests path)
requests: A list of requests

second level: [[requests]]
name: A name given to the endpoint
method: The http method called
path: The path of the endpoint concatenated after the base_url
headers: The headers pass in json
body: The body pass in json
scripts: A list of scripts

third level: [[scripts]]
before: is the script executed before or after the requests
script: the actual content of the script in rhai
description: a short description of the script

## What is the output of the request ?

The content formatted (options to qwest run: --format json/html/...)
The headers of the response
# Qwest
# Qwest
[?1049h[?1h=[H[J[?2004h[?2026$p[?2027$p[?2031$p[?2048$p[?u[c[34h[?25h[?25l[m[H                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                [?1004h[34h[?25h[?25l[2 q[2 q[?1002h[?1006h[m[H                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [m
                                                                                [H# Qwest
