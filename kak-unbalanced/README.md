# kak-unbalanced

`kak-unbalanced` reads a Kakoune buffer from standard input and prints commands
that highlight unmatched `()`, `[]`, and `{}` delimiters with the `Unbalanced`
face.
When provided, the first positional file argument takes priority over standard
input.

Delimiters inside double quotes, triple double quotes, and triple-backtick
fences are ignored. Single quotes and comments do not suppress checking.

Build it with:

```sh
zig build -Doptimize=ReleaseSafe
```

Evaluate its output from Kakoune:

```kak
evaluate-commands %sh{ kak_unbalanced < "$kak_buffile" }
```

It can also scan a file directly:

```sh
kak_unbalanced path/to/file
```

Source `plugin/kak-unbalanced.kak` to install idle hooks that scan live buffer
contents after `NormalIdle` and `InsertIdle`. The plugin defines `Unbalanced` as
an alias of `Error`; override it with `set-face` to customize its appearance.

The same script is embedded from `plugin/kak-unbalanced.kak` at build time and
can be included directly from the executable:

```kak
evaluate-commands %sh{ kak_unbalanced init }
```

Set `kak_unbalanced_command` when the executable is not available as
`kak_unbalanced` on `PATH`.

The plugin declares `kak_unbalanced_ranges` and installs the
`buffer/kak-unbalanced` ranges highlighter. CLI scan output only updates the
range option.
