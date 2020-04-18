# Hagen ![Master CI](https://github.com/ctron/hagen/workflows/Master%20CI/badge.svg) ![Crates.io](https://img.shields.io/crates/v/hagen)

"Hagen" is a generator for static homepages, written in Rust.

For more information see: [https://ctron.github.io/hagen](https://ctron.github.io/hagen)

## Example website

You can find an example web site in the folder [/website](website). It contains
the source code to site published on [https://ctron.github.io/hagen](https://ctron.github.io/hagen).
The goal of the website is to showcase and document the capabilities of Hagen.
But I know, the documentation needs more work :-)

## Installing

You can install Hagen with `cargo`:

    cargo install hagen

## Container image

There is also a container image, which you can run with docker:

    docker run quay.io/ctron/hagen

For example:

    docker run -v $(pwd)/website:/homepage --rm -ti quay.io/ctron/hagen

## Running

Calling `hagen` with `--help` will give you more information about the
arguments.
