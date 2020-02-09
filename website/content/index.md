---
layout: default
title: Hagen
timestamp:
  published: 2020-02-01T19:42:00+02:00
description: |
  Hagen is a static website generator, written in Rust.
---

## Welcome to Hagen

Hagen is a static website generator, written the [Rust language](https://www.rust-lang.org/). It's goal is
to be flexible and extensible, making it possible to generate all kinds of web sites, not only blogs. 

## Installing

You can install Hagen using cargo:

    cargo install hagen

## Container

Hagen can also be run directly using docker:

    docker run -v $(pwd)/my-page:/homepage --rm -ti quay.io/ctron/hagen

## Live preview and asset processing

The current focus of Hagen is to generate homepages. Loading content, and providing
an easy, yet flexible way to render pages.

This is why there is no live preview, SCSS, or image processing. There are other
tools out there, which can do the same job. For example the "node-sass" processor
does a great job, processing the SCSS files. So why replicate all the effort?

And with e.g. Python, you can quickly run a local web server.

In the source code of this webpage, you will find a template setup, which makes
use of NodeJS, Yarn, SCSS, and Python to get you started.