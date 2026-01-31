provide-module buffer-list %{
  define-command buffer-list-arrange-at-point -hidden -override %{
    eval -draft %{
      exec x_y
      arrange-buffers %val{selection}
      buffer-list-insert-reload
    }
    exec gk
  }
  define-command buffer-list-delete-at-point -hidden -override %{
    eval -draft %{
      exec x_y
      delete-buffer %val{selection}
      exec xd
    }
  }
  define-command buffer-list-insert-reload -override %{
    eval -draft %{
      exec "%%d"
      set-register dquote %val{buflist}
      exec %|<a-P>i<ret><esc>gkd|
    }
  }
  define-command buffer-list-show -docstring "Show Buffer List" -override %{
    edit! -scratch *buffer-list*
    map buffer normal <ret> %|x_y:b <c-r>"<ret>|
    buffer-list-insert-reload
    exec gk
    map buffer normal a :buffer-list-arrange-at-point<ret>
    map buffer normal d :buffer-list-delete-at-point<ret>
    map buffer normal q :db<ret>
  }
}
