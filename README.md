# template-eyre

An error reporter for panics and `eyre::Report` with handlebars template support.

Ever used [eyre](https://github.com/yaahc/eyre) and finding existing handlers too boring or complex?
This crate enables you to customize your error report and add custom fields
in seconds.

## Features
- [x] Handlebars powered error reports
- [x] Color support
- [x] Custom indent support
- [x] Sensible bundled handlers
- [ ] Backtrace support
- [ ] Panic handler
- [ ] Custom fields support

## Write your own template

This crate includes two templates `Hook::simple` and `Hook::colored_simple`,
but you can always create your own templates easily!

First, get started by reading the [handlebars guide](https://handlebarsjs.com/guide/).
Handlebars is a simple template language, so this won't be hard.

Next, start writing your own template! You may gain some idea from [builtin templates](src/templates.rs).

Notice that this crate provides you with [some handy helpers](#additional-helpers).
Also, the `handlebars` crate this crate depends on also has some [custom helpers](https://docs.rs/handlebars/latest/handlebars/#built-in-helpers).

## Example

A minimal handler can be built with a template like this:

```handlebars
Oh no, this program crashed!

{{style "red" error}}
Caused by:
{{#each sources}}
{{indent @index (style "yellow" this)}}
{{/each}}

{{style "cyan" "Please report this issue to ..."}}
```

and you get a flavored error report:

<pre>
Oh no, this program crashed!

<span style="color: #cc0000">Unable to talk to daemon</span>
Caused by:
   0: <span style="color: #c4a000">Connection refused</span>
   1: <span style="color: #c4a000">os error 61</span>

<span style="color: #06989a">Please report this issue to ...</span>
</pre>

## Additional helpers

### `style` helper

Color the output. Styles should be written at its "dotted" form.
 
See [console's document](https://docs.rs/console/latest/console/struct.Style.html#implementations) for details.

e.g, `{{style "black.bold.on_red" error}}`

### `indent` helper

Indent a block.

This helper has three forms:

- `indent content` - Indent the content by four spaces.
- `indent <number> content` - Insert a number before the first line with the same indentation level as backtraces.
- `indent <string> content` - Insert given string before every line.

e.g, `{{indent @index this}}`

## License
This project is licensed under [MIT License](LICENSE.txt).