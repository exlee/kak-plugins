provide-module -override kak-univ-search %{
  define-command -hidden -override universal-search-prompt-done %{
    select 1.1,1.1
    unset-face window PrimaryCursor
  }
  define-command -hidden -override universal-search-prompt-setup %{
    ansi-enable
    set-face window PrimaryCursor background
  }
  define-command restart-univ-search -docstring "(Re)starts kak-univ-search helper" %{
    nop %sh{
      kak-univ-search -d
    }
  }
  restart-univ-search
  define-command -override universal-search-rg %{
    edit! -scratch *rg*
    universal-search-prompt-setup
    set buffer filetype grep
    prompt -on-change %{
      nop %sh{
      printf "rg\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_text" > /tmp/kak-univ-search
      }
    } "(rg)> " universal-search-prompt-done
  }

  define-command -override universal-search-fd %{
    edit! -scratch *fd*
    universal-search-prompt-setup
    map buffer normal <ret> x_gf
    prompt -on-change %{
      nop %sh{
      printf "fd\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_text" > /tmp/kak-univ-search
      }
    } "(fd)> " universal-search-prompt-done
  }
  define-command -override universal-search-fzf %{
    edit! -scratch *fzf*
    universal-search-prompt-setup
    map buffer normal <ret> x_gf
    prompt -on-change %{
      nop %sh{
      printf "fzf\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_text" > /tmp/kak-univ-search
      }
    } "(fzf)> " universal-search-prompt-done
  }
  define-command -override universal-search-global-definition %{
    edit! -scratch *global*
    universal-search-prompt-setup
    set buffer filetype grep
    prompt -on-change %{
      nop %sh{
      printf "global\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "-d $kak_text" > /tmp/kak-univ-search
      }
    } "(def)> " universal-search-prompt-done
  }

  define-command -override universal-search-global-grep %{
    edit! -scratch *global*
    universal-search-prompt-setup
    set buffer filetype grep
    prompt -on-change %{
      nop %sh{
      printf "global\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "-g $kak_text" > /tmp/kak-univ-search
      }
    } "(grep)> " universal-search-prompt-done
  }
  define-command -override universal-search-global-ref %{
    edit! -scratch *global*
    universal-search-prompt-setup
    set buffer filetype grep
    prompt -on-change %{
      nop %sh{
      printf "global\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_text" > /tmp/kak-univ-search
      }
    } "(ref)> " universal-search-prompt-done
  }
  declare-option str universal_search_buffer_search 
  declare-option str universal_search_temp_buffer_file "/tmp/kak-univ.%val{session}"
  define-command -override universal-search-buffer %{
    set-option global universal_search_buffer_search %val{buffile}
    write -sync -force -method overwrite %opt{universal_search_temp_buffer_file}
    edit! -scratch *buffer-search*
    universal-search-prompt-setup
    map buffer normal <ret> %|ght:y:edit %opt{universal_search_buffer_search} <c-r>"<ret>|
    set buffer filetype grep
    prompt -on-change %{
      nop %sh{
      printf "buffer-search\0%s\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_opt_universal_search_temp_buffer_file" "$kak_text" > /tmp/kak-univ-search
      }
    } "(buffer)> " %{
      nop %sh{"rm $kak_opt_universal_search_temp_buffer_file"}
      universal-search-prompt-done
    }
  }
  define-command -override universal-search-open-buffer-list %{
    edit! -scratch *buffer-list-search*
    map buffer normal <ret> %|x_y:buffer <c-r>"<ret>|
    eval -draft %{
      set-register a %val{buflist}
      exec %|"a<a-p>i<ret><esc>gkdd|
    }
  }
  define-command -override universal-search-buffer-list %{
		universal-search-open-buffer-list
    write -sync -force -method overwrite %opt{universal_search_temp_buffer_file}
    universal-search-prompt-setup
    prompt -on-change %{
      nop %sh{
      printf "buffer-list-search\0%s\0%s\0%s\0%s\0%s\0\0" $kak_session $kak_client "$PWD" "$kak_opt_universal_search_temp_buffer_file" "$kak_text" > /tmp/kak-univ-search
      }
    } "(buffer)> " %{
      nop %sh{"rm $kak_opt_universal_search_temp_buffer_file"}
      universal-search-prompt-done
    }
  }


  try %{
    declare-user-mode universal-search
    declare-user-mode universal-search-global
  }
  map global universal-search f ":universal-search-fd<ret>" -docstring "Search using fd"
  map global universal-search z ":universal-search-fzf<ret>" -docstring "Search using fzf"
  map global universal-search r ":universal-search-rg<ret>" -docstring "Search using rg"
  map global universal-search s ":universal-search-buffer<ret>" -docstring "Search buffer"
  map global universal-search b ":universal-search-open-buffer-list<ret>" -docstring "Open buffer List"
  map global universal-search v ":universal-search-buffer-list<ret>" -docstring "Search buffer List"
  map global universal-search g ":enter-user-mode universal-search-global<ret>" -docstring "Search using global"
  map global universal-search-global d ":universal-search-global-definition<ret>" -docstring "Search definition"
  map global universal-search-global g ":universal-search-global-grep<ret>" -docstring "Global grep"
  map global universal-search-global r ":universal-search-global-ref<ret>" -docstring "Global ref"
  #map global normal <F2> ":universal-search-buffer<ret>"
}

provide-module kak-univ-search-maps %{
  require-module kak-univ-search
  map global user s ":enter-user-mode universal-search<ret>" -docstring "Universal Search"
  map global user f ":universal-search-fd<ret>" -docstring "Search using fd"
  map global user / ":universal-search-rg<ret>" -docstring "Search using rg"
}
