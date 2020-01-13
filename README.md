# A homepage generator written in Rust

"Hagen" is a generator for static homepages, written in Rust.

## Minimal example setup

`render.yaml`:
~~~yaml
rules:
  - selectorType: layout
    template: "{{ frontMatter.layout }}"
    outputPattern: "{{ metadata.parent }}/{{ metadata.name }}.html"

assets:
  - dir: assets
    to: assets
~~~

`templates/default.hbs`:
~~~handlebars
<!doctype html>
<html>
  <head>
    <title>{{compact.site.info.title}}</title>
  </head>
  <body>
    <header>
      <h1>{{context.pageTitle}}</h1>
    </header>
    <main>
      {{ expand ( markdownify context.content) }}
    </main>
    <footer>
      Copyright 2019-{{time "%Y"}} ACME Inc. All rights reserved.
    </footer>
  </body>
</html>
~~~

`content/site.yaml`:
~~~yaml
info:
  title: Example site
  clain: Just testing.
~~~

`content/index.md`:
~~~markdown
---
layout: default
pageTitle: Page Title
---
## Foo Bar

**Welcome** to my test.
~~~

## Variables

Different contexts have different variables. All values are JSON based.

### Page

<dl>

<dt><code>full</code></dt>
<dd>The full content tree.</dd>

<dt><code>compact</code></dt>
<dd>The compacted content tree. Containing only content sections.</dd>

<dt><code>context</code>
<dd>The content of the point selected from the content tree.</dd>

<dt><code>output</code></dt>
<dd>The output information.

<dl>
<dt><code>path</code></dt>
<dd>The path of the output page.</dd>
<dt><code>site_url</code></dt>
<dd>The base site URL.</dd>
</dl>

</dd>

</dl>

