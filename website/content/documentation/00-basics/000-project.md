---
title: Basic Project
layout: documentation
timestamp:
  published: 2020-04-21T20:11:00+02:00
---

## Project layout

The basic project structure of a Hagen project is:

~~~text
/hagen.yaml      # the main configuration file
/content/        # the content directory
~~~

## Minimal Hagen configuration

The minimal `hagen.yaml` file might look like this:

{{{{raw}}}}
~~~yaml
site:
    basename: https://ctron.github.io/hagen
rules:
  - selectorType: layout
    template: "{{ frontMatter.layout }}"
    outputPattern: "{{ metadata.parent }}/{{ metadata.name }}.html"
    context:
      page: $
      content: $.content
      timestamp: $.frontMatter.timestamp
~~~
{{{{/raw}}}}

The value `.site.basename` is mandatory. It contains the base name
of the site where the page will be hosted on. You can always override
this from the command line for testing or staging sites though.

The field `.rules` is an array of rules, which will be processed when
the generator is running. In a nutshell Hagen will:

* Load all content
* Select what needs to be rendered
* Render the selected content

## Rules

Each rule consists of:

* The **selector type** (`selectorType`) which specifies the type
  of selector being used for selecting content. Some selector types might
  require additional information (like an expression), others don't.

* An **output pattern** (`outputPattern`) which defines the filename,
  relative to the root of the output folder, to which the content
  should be rendered. The output pattern is actually template so a value
  of `{{{{raw}}}}{{ metadata.parent }}/{{ metadata.name }}.html{{{{/raw}}}}` would simply construct
  a name based on the location in the content tree (`parent`), the name of the
  content file (`name`) and the static suffix `.html`.

Additionally a rule might have:

* Additional information for the selector can be provided in the field
  `selector`. The content is specific to the type of the selector.

* The name of a template in the field `template`. This field holds a
  template expression, so that the name can be generated. For example
  `{{{{raw}}}}{{ frontMatter.layout }}{{{{/raw}}}}` would use the field `layout` from the
  *front matter* as template name. Of course this can also be a static
  string.
  
  If no template is specified, then the selector must point to an object
  which has a field named `content`, which will be used as an ad hoc
  template instead.

* The field `context` can be used to customize the page context, which is
  provided to the template rendering the page. This may be used to prepare
  and customize fields for the page context.

  If no context is set, then the current point in the content tree, where
  the selector matched, will be used as the page context.

### Selector types

The following selector types are available:

* `layout` &ndash; Matches if the current object being evaluated has the field `.frontMatter.layout`.
  If an additional value is configured in the `selector` field of the rule, then not only must the
  field exists, but also the value must match.
* `type` &ndash; Matches if the current object being evaluated has the field `.metadata.type`.
  If an additional value is configured in the `selector` field of the rule, then not only must the
  field exists, but also the value must match.
* `jsonpath` &ndash; Matches of the current object being evaluated matches the JSON path expression.
   A valid JSON path expression must be provided in the `selector` field of the rule.
