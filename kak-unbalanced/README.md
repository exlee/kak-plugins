# kak-unbalanced

`kak-unbalanced` highlights unmatched `()`, `[]`, and `{}` delimiters in
Kakoune using the `Unbalanced` face.

Delimiters inside double quotes, triple double quotes, and triple-backtick
fences are ignored. Single quotes and comments do not suppress checking.

## Build

```sh
zig build -Doptimize=ReleaseSafe
```

## Usage

Make the `kak_unbalanced` executable available on your `PATH`, then load the
plugin from your `kakrc`:

```kak
evaluate-commands %sh{ kak_unbalanced init }
require-module kak-unbalanced
```

`kak_unbalanced init` prints the bundled Kakoune script (the same content as
`plugin/kak-unbalanced.kak`, embedded into the binary at build time), which
declares the `kak-unbalanced` module with `provide-module`. Calling
`require-module kak-unbalanced` activates it: it installs idle hooks that scan
the live buffer after `NormalIdle` and `InsertIdle`, declares the
`kak_unbalanced_ranges` option, and adds the `buffer/kak-unbalanced` ranges
highlighter.

`Unbalanced` is an alias of `Error`; override it with `set-face` to customize
its appearance:

```kak
set-face global Unbalanced red+b
```

If the executable is not available as `kak_unbalanced` on `PATH`, point the
plugin at it with:

```kak
set-option global kak_unbalanced_command /path/to/kak_unbalanced
```

## Standalone scanning

The executable reads a buffer from standard input (or from a positional file
argument, which takes priority) and prints Kakoune commands that update the
`kak_unbalanced_ranges` option without installing any hooks:

```sh
kak_unbalanced path/to/file
# or
some-cmd | kak_unbalanced
```

Evaluate the output to apply the ranges:

```kak
evaluate-commands %sh{ kak_unbalanced < "$kak_buffile" }
```
