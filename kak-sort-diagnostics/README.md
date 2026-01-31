# kak-sort-diagnostics

Sort kakoune-lsp `*diagnostics*` buffer.

## Kakoune Script

```kak
define-command lsp-sort-diagnostics -docstring "Sort *diagnostics* buffer" -override %{
  exec -draft "%%|kak-sort-diagnostics<ret>"
  select 1.1,1.1
}

hook -group diagnostics-sort global BufCreate "\*diagnostics\*" %{
  hook -once buffer NormalIdle .* %{
    remove-highlighter window/wrap
    lsp-sort-diagnostics
  }
}
```

