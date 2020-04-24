---
title: Live reload
layout: documentation
---

Live reload is feature that would be nice to have. But for the moment you
can have a quick workaround with a single line of Python:

    python3 -m http.server 8080

Of course you can add this again to your `Makefile`:

~~~
build-dev: assets
	hagen -b http://localhost:8080 -D

run: build-dev
	cd output && python3 -m http.server 8080
~~~

You you can run `make run` to build and run the server. And `make build-dev`
to rebuild the content.

You can see the full example here: [/website/Makefile](https://github.com/ctron/hagen/blob/master/website/Makefile).
