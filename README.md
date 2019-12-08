Vai [![Github](https://github.com/m-lima/rust-vai/workflows/build/badge.svg)](https://github.com/m-lima/rust-vai/actions?workflow=build) [![Appveyor](https://ci.appveyor.com/api/projects/status/sv6wqqb7s6wo1e0x?svg=true)](https://ci.appveyor.com/project/mlima/rust-vai)
========
> A browser query launcher

## Quick start
```bash
$ git clone https://github.com/m-lima/rust-vai
$ cd rust-vai
$ cargo install --path .
$ cat config.json | vai -r
$ # Now you can just use it!
$ vai youtube rust jon gjengset
```

## Configuration

| Field        | Description |
| ------------ | -------------|
| `name`       | Name of the target, referenced when calling `vai` |
| `command`    | URL to use when calling the browser for this target. Query will be appended to it |
| `suggestion` | URL to use for suggestions from the target. Query will be appended to it |
| `parser`     | How to parse the suggestions. One of `GOOGLE`, `DUCK`, `NONE` |

#### Quick setup
You can load a JSON configuration to `vai` directly using the `-r` flag
```bash
$ cd <this_repository>
$ cat config.json | vai -r
```

#### Example
```json
[
  {
    "name": "start",
    "command": "https://www.startpage.com/sp/search?t=blak&lui=english&language=english&cat=web&query=",
    "suggestion": "https://suggestqueries.google.com/complete/search?output=firefox&q=",
    "parser": "GOOGLE"
  },
  {
    "name": "google",
    "command": "https://www.google.com/search?q=",
    "suggestion": "https://suggestqueries.google.com/complete/search?output=firefox&q=",
    "parser": "GOOGLE"
  },
  {
    "name": "duck",
    "command": "https://duckduckgo.com/?q=",
    "suggestion": "https://duckduckgo.com/ac/?q=",
    "parser": "DUCK"
  },
  {
    "name": "youtube",
    "command": "https://youtube.com/results?search_query=",
    "suggestion": "",
    "parser": "NONE"
  },
  {
    "name": "image",
    "command": "https://www.google.com/search?tbm=isch&q=",
    "suggestion": "https://suggestqueries.google.com/complete/search?output=firefox&q=",
    "parser": "GOOGLE"
  }
]
```

## Usage
`$ vai <target> <query>`

#### Example
`$ vai youtube rust jon gjengset`
