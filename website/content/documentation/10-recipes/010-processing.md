---
title: Processing assets
layout: documentation
---

While Hagen can copy assets, it currently does not process any SCSS,
or minimize CSS/Javascript.

One of the reasons for that is, that other tools can do this quite well,
and so what is the benefit of replicating that.

With a simple `Makefile`, you easily trigger and full blown process. For
example, this web page processes SCSS simply by:

~~~makefile
build/assets/bootstrap/bootstrap.min.css: scss/custom.scss node_modules/bootstrap/scss/bootstrap.scss package.json postcss.config.js
	yarn css:compile
	yarn css:prefix
	yarn css:minify
~~~

You can see the full example here: [/website/Makefile](https://github.com/ctron/hagen/blob/master/website/Makefile).
