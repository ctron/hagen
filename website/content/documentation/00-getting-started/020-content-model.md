---
title: Content Model
layout: documentation
timestamp:
  published: 2020-04-21T20:11:00+02:00
---

## Content

The internal content model of Hagen is based on JSON structures. This is coming
from the fact that Hagen uses various technologies, like Handlebars and JSON path,
which are all based around JSON. And while you still can create complex objects
in JSON, the content model is still limited to the JSON primitive types
(`string`, `number`, `boolean`, …).

{{#>note type="info" title="Using YAML"}}
While internally everything works with the JSON data model, authoring and reading
JSON can be quite painful. Thus Hagen uses YAML, instead of JSON, when it comes
to reading and writing files, based on the JSON data model.
{{/note}}

When building, Hagen it will load all content from the `content` directory in the
root of the project. It will iterate through the directory structure, and build
a single model out of all information it can find, and understand. Including the
directories themselves.

## Different content types

The content loader will iterate through the `content` directory, and start loading
all files and directory entries. The initial entry in the content tree will
be the `content` directory itself.

The basic structure of a content entry is:

~~~yaml
metadata:
  name: index         # The basename of the file
  parent: /about      # The parent directory, relative to the content root
  type: md            # The content type
  filename: index.md  # The name of the source file
frontMatter:          # A map of all front matter, may be an empty map '{}'
  title: About
content:              # The section specific to the content type
~~~

### Directory

A directory will load the content section with a map of all files and directories
it can find an parse:

~~~yaml
content:
  file-1: {}
  file-2: {}
  directory: {}
~~~

It will also iterate through all sub-directories, and load them into the map as well,
so a more populated directory might look like this:

~~~yaml
metadata:
  name: about
  type: directory
  parent: /
  filename: about
content:
  index:
    metadata:
      name: index
      type: md
      parent: /about
      filename: index.md
    frontMatter:
      title: About
    content: "## About\n\nAbout this site..."
  help:
    metadata:
      name: help
      type: directory
      parent: /about
      filename: help
    frontMatter: {}
    content:
      index:
        metadata:
          name: index
          type: md
          parent: /about/help
          filename: index.md
        frontMatter:
          title: Getting help
        content: "## Help\n\nContact us..."
~~~

### Plain text

A plain text file will simply load the content of the file into the content section:

~~~yaml
metadata:
  name: robots
  parent: /
  type: txt
  filename: robots.txt
frontMatter: {}
content: "{{{{raw}}}}Sitemap: {{ absolute_url "/sitemap.xml" }}{{{{/raw}}}}"
~~~

Front matter will be loaded as described in [Front matter](#front-matter).

### YAML

YAML files are parsed and the content loaded as content section. Assuming you have a
file `site.yaml` like this:

~~~yaml
title: My Site
language: en-US
menu:
  - label: About
    url: /about/
~~~

The the content will be:

~~~yaml
metadata:
  name: site
  parent: /
  type: yaml
  filename: site.yaml
frontMatter: {}
content:
  title: My Site
  language: en-US
  menu:
    - label: About
      url: /about/
~~~

### Markdown

Markdown is similar to plain text, as the mark down content will be parsed later on, using a helper function
named `markdownify`. So a simple markdown file named `index.md` like this:

~~~markdown
---
title: About
---
# About

About this site...
~~~

Would translate into:

~~~yaml
metadata:
  name: index
  parent: /about
  type: md
  filename: index.md
frontMatter:
  title: About
content: "#About\n\nAbout this site..."
~~~

## <span id="front-matter">Front matter

Front matter is additional information, metadata, which can be attached to a page.
It is not content, which is should render as part of the normal content, but used
elsewhere, like a publishing timestamp, additional information for Twitter, the author, …

Text files (like `md` files), allow to prefix the main content, with front matter like this:

~~~text
---
foo: bar
---
Actual content
~~~

The first line of the file must be `---` (three normal dashes). Everything after this line, until
the next occurrence of `---` will be parsed as YAML, and set into the front matter field.

If the YAML cannot be parsed, Hagen will abort. If the front matter section is missing, then
the front matter section will simply be empty.

## Different representations

In addition to the *full* data model, Hagen also transforms this into a *compact*
form, which might be handy in some scenarios. The compact format is generated
from the complete one by only using the `content` sections.

The following *full* model:

~~~yaml
metadata:
  name: content
  parent: /path/to/root
  filename: content
  type: directory
frontMatter: {}
content:
  about:
    metadata:
      name: about
      parent: /
      filename: about
      type: directory
    frontMatter: {}
    content:
      index:
        metadata:
          name: index
          parent: /about
          filename: index.md
          type: md
        frontMatter:
          title: About
          layout: default
        content: "## About\n\nSome more content"
  site:
    metadata:
      name: site
      parent: /
      filename: site.yaml
      type: yaml
    frontMatter: {}
    content:
      title: My Site
      language: en-US
      menu:
        - label: Home
          url: /
        - label: About
          url: /about/
~~~

Would be translated into the *compact* model as:

~~~yaml
about:
  index: "## About\n\nSome more content"
site:
  title: My Site
  language: en-US
  menu:
    - label: Home
      url: /
    - label: About
      url: /about/
~~~

So instead of using `.full.content.site.content.title`
you could also use the shorter form `.compact.site.title`.

## Dumping

In order to better understand how the content directory translates into the
content model and to make it easier to debug, you can output the internal
content model as a YAML structure by passing `-D` or `--dump` as a command
line argument. This will generate the files `content.yaml` and `compact.yaml`
in the output directory.