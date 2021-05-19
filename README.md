# indexa-http

[![build](https://github.com/mosmeh/indexa-http/workflows/build/badge.svg)](https://github.com/mosmeh/indexa-http/actions)

HTTP server and web interface for [indexa](https://github.com/mosmeh/indexa)

## Installation

Clone this repository and run:

```sh
cargo install --path .
```

## Usage

Launch the server with:

```sh
indexa-http
```

It will locate and load indexa's database and config.

The web interface is at <http://127.0.0.1:8080> by default.

## API

### `GET /info`: Get database information

```bash
curl 'http://127.0.0.1:8080/info'
```

#### Example response

```json
{
    "numEntries": 1287895,
    "rootDirs": ["/"],
    "indexed": ["basename", "path", "extension", "size", "modified"],
    "fastSortable": ["basename", "modified"]
}
```

### `GET /search`: Search

```bash
curl 'http://127.0.0.1:8080/search?query=foo&statuses=basename,size,path&sortBy=modified&sortOrder=desc'
```

#### Parameters

-   query
-   limit
-   statuses
-   matchPath
-   caseSensitivity
-   regex
-   sortBy
-   sortOrder
-   sortDirsBeforeFiles

#### Example response

```json
{
    "query": "foo",
    "numHits": 573,
    "hits": [
        {
            "isDir": false,
            "basename": "foo.rs",
            "path": "/path/to/file/foo.rs",
            "size": 42,
            "highlighted": {
                "basename": "<em>foo</em>.rs",
                "path": "/path/to/file/<em>foo</em>.rs"
            }
        },
        ...
    ]
}
```

## Command-line options

```
USAGE:
    indexa-http [OPTIONS]

OPTIONS:
    -a, --addr <addr>          Address to listen on [default: 127.0.0.1:8080]
    -t, --threads <threads>    Number of threads to use
    -C, --config <config>      Location of the config file
```
