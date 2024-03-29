# [Akasio (Rust)](https://github.com/k4yt3x/akasio-rust)

## Description

Akasio is a simple HTTP server that redirects traffic based on a JSON redirect table.

If you own a domain and wish to self-host a URL-shortening service, then this is the right tool for you.

Originally, Akasio is a backend server for website [akas.io](https://akas.io) (akas stands for "also known as") written in Python and Flask. There is a previous attempt to port this program [to Golang](https://github.com/k4yt3x/akasio-go), but I didn't really like how Go works so I decided to rewrite it in Rust. This program requires only one JSON file as its configuration file, and can be deployed with a minimal container image as small as 8.5 MiB (on-disk size).

## Why Akasio

> What can this be used for?

Personally, I find sending long URLs like `https://gist.githubusercontent.com/k4yt3x/3b41a1a65f5d3087133e449793eb8858/raw` to people pretty annoying, since you'll either have to copy and paste the whole URL or type the whole URL out. URL shorteners like Akasio solve this issue. All that's needed to be done to send such a long URL is just to create a new mapping in the redirect table (akas.io/z).

> What are Akasio's benefits compared to services like [bit.ly](https://bit.ly)?

Akasio is self-hosted, and the redirect table is just a JSON file. This gives the users lots of flexibilities. The JSON file on the server can be symbolic-linked from a local workstation, updated by a front-end webpage, generated from a program, and so on.

> Are there even lighter alternatives if I don't want to set a server up?

Yes. You can use cloud functions like [AWS Lambda](https://aws.amazon.com/lambda/) to run a similar redirection service. Cloud function scripts can be written in Python, JavaScript, and several other languages. You can seek alternative solutions that are written for cloud functions.

## Usages

This section covers Akasio's fundamental concepts, basic usages and setup guide.

### Redirect Table

Akasio redirects incoming requests based on what's called a "redirect table". This table is essentially a JSON file with a simple source-to-target mapping. You can find an example `akasio.json` under the `configs` directory. By default, Akasio reads the redirect table from `/etc/akasio.json`.

```json
{
  "/": "http://k4yt3x.com/akasio-rust",
  "/g": "https://github.com/k4yt3x",
  "/k4yt3x": "https://k4yt3x.com"
}
```

This example redirect table does the following mappings:

- `/` to http://k4yt3x.com/akasio-rust/
- `/g` to https://github.com/k4yt3x
- `/k4yt3x` to https://k4yt3x.com

Taking the `/g` mapping for example, when a user visits `https://yourwebsite.com/g`, the user will be redirected to https://github.com/k4yt3x via a HTTP 301 (moved permanently) response.

### URL Segments

When Akasio receives a request, it chops the request's path into segments, looks up the first segment against the redirect table, and returns the target URL + the rest of the segments joined with `/`.

For example, if the request URI is `/segment1/segment2/segment3`, the URI will be split into a string array with elements `segment1`, `segment2`, and `segment3`. Akasio will then lookup `/segment1` within the redirect table and return `redirected target URL + /segment2/segment3`. If a `/` is not present at the end of the target URL, one will be appended automatically.

Continuing the example in the previous section, if the user visits `https://yourwebsite.com/g/akasio-go`, the user will be redirected to https://github.com/k4yt3x/akasio-go.

### Website Setup

The recommended setup is to start Akasio as a service behind reverse proxy web server like Apache, Nginx or Caddy. You can find an example service file at `configs/akasio.service`.

A typical stand-alone setup process will look like the following.

1. Build the `akasio` binary or download the `akasio` binary from [releases](https://github.com/k4yt3x/akasio-rust/releases).
1. Move the `akasio` binary to `/usr/local/bin/akasio`.
1. Modify the configuration file and put it at `/etc/akasio.json`.
1. Move the service file to `/etc/systemd/system/akasio.service`.
1. Reload systemd with `systemctl daemon-reload`.
1. Enable and start the service with `systemctl enable --now akasio`.
1. Verify that the service has been started successfully via `curl -v 127.0.0.1:8000`.
1. Configure front-end web server to reverse proxy to http://127.0.0.1:8000.

### Binary Usages

The binary's usage is as following. You can also invoke `akasio -h` to see the usages.

```console
A simple Rust program that redirects HTTP requests

Usage: akasio [OPTIONS]

Options:
  -b, --bind <BIND>    [default: 127.0.0.1:8000]
  -t, --table <TABLE>  [default: akasio.json]
  -h, --help           Print help
  -V, --version        Print version
```

The command below, for instance, launches Akasio, reading configurations from the file `/etc/akasio.json`.

```shell
/usr/local/bin/akasio -t /etc/akasio.json
```

### Running from Docker

Akasio is also available as a container image. Below is an example how you can run Akasio with Docker. Be sure to create the redirect table and change the redirect table's path in the command below.

```shell
docker run -it -p 8000:8000 -v $PWD/akasio.json:/etc/akasio.json -h akasio --name akasio ghcr.io/k4yt3x/akasio:2.1.0

docker run -it \                                            # interactive
           -p 8000:8000 \                                   # bind container port to host's port 8000
           -v $PWD/akasio.json:/etc/akasio.json \           # bind mount host's akasio.json file under the current directory to container's /etc/akasio.json
           -h akasio \                                      # set container hostname akasio
           --name akasio \                                  # set container name akasio
           ghcr.io/k4yt3x/akasio:latest \                   # container name
```

After spinning the container up, you can verify that it's running correctly by making a query with `curl` or any other tool of your preference.

## Building From Source

The following commands will build Akasio binary at `target/release/akasio`.

```shell
git clone https://github.com/k4yt3x/akasio-rust.git
cd akasio-rust
cargo build --release
```

After building, you may also use `sudo make install` to install `akasio` to `/usr/local/bin/akasio`.
