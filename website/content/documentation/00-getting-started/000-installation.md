---
title: Installation
layout: documentation
timestamp:
  published: 2020-04-24T21:11:00+02:00
---

## Download a binary

Go to the release page on GitHub and download the most recent release:

<div class="px-3 my-3">
<a href="https://github.com/ctron/hagen/releases">https://github.com/ctron/hagen/releases</a>
</div>

## Using Cargo

If you have Rust and Cargo installed, you can simply install Hagen using cargo:

    cargo install hagen

## Using Docker

Hagen can also be run directly using docker:

    docker run -v /path/to/my-page:/homepage --rm -ti quay.io/ctron/hagen

`/homepage` is the path inside the container which Hagen will
use as the root.
