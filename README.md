# URL-Shortener-rs

[![Build](https://github.com/AntoniosBarotsis/url-shortener-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/AntoniosBarotsis/url-shortener-rs/actions/workflows/ci.yml)

A very simple URL shortener, mostly built to familiarize myself with
[shuttle.rs](https://www.shuttle.rs/).

This somewhat follows
[one of the official tutorials](https://docs.shuttle.rs/tutorials/url-shortener) only using Axum
instead of Rocket and also adding a metadata component to the whole thing.

## Requirements

You will need to [install `protoc`](https://docs.shuttle.rs/support/installing-protoc) as well as
[the Shuttle cli](https://docs.shuttle.rs/introduction/installation).

## Running the App Locally

> You need to have Docker running!

```sh
cargo shuttle run
```

## Docs

```sh
$ curl 'localhost:8000/help'

  [POST] /shorten      - Shortens a URL                  | Body should contain the URL in raw text.
  [GET]  /:id          - Redirects to the URL
  [GET]  /metadata/:id - Returns the metadata of the URL
```

## Production

This is currently also hosted at `https://gdsc-tud-url.shuttleapp.rs` however I will make no guarantees on its
stability/longevity as I made this over a couple of hours as an experiment 😅
