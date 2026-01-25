provide-module kak-folder %{ 
  declare-option range-specs fold_regions
  declare-user-mode fold-mode
  define-command fold-add-selection -docstring "Add selection to folding" -override %{
    #echo -debug %sh{ echo kak-folder add "$kak_opt_fold_regions" "$kak_selections_desc" }
    #echo -debug %sh{ kak-folder add "$kak_opt_fold_regions" "$kak_selections_desc" }
    eval %exp{ set-option window fold_regions %sh{ kak-folder add "$kak_opt_fold_regions" "$kak_selections_desc" } }
  }
  define-command fold-remove-selection -docstring "Remove selection from folding" -override %{
    #echo -debug %sh{ echo kak-folder remove "$kak_opt_fold_regions" "$kak_selections_desc" }
    #echo -debug %sh{ kak-folder remove "$kak_opt_fold_regions" "$kak_selections_desc" }
    eval %exp{ set-option window fold_regions %sh{ kak-folder remove "$kak_opt_fold_regions" "$kak_selections_desc" } }
  }

  define-command fold-enable -docstring "Enable folding" -override %{
    add-highlighter window/folds replace-ranges fold_regions

  }
  define-command fold-disable -docstring "Disable folding" -override %{
    remove-highlighter window/folds
  }
  define-command fold-reset -docstring "Reset folds" -override %{
    set-option window fold_regions
  }
  define-command fold-select -docstring "Select folds" -override %{
    fold-disable
    select %sh{echo $kak_opt_fold_regions | xargs -n 1 echo | sed -e '1d' | cut -d'|' -f1 | xargs echo }
  }

  define-command fold-test -override %{
    echo %sh{ echo $kak_selections_desc  | xargs -n 1 -I {} echo "{}|..." | xargs echo }
  }
  define-command fold-matching -override %{
      exec "mH<a-;>L"
      fold-add-selection
  }

	hook -group kak-fold global WinCreate .* %{
  	fold-enable
	}
  map global fold-mode e :fold-enable<ret>  -docstring "Fold Enable"
  map global fold-mode d :fold-disable<ret>  -docstring "Fold Disable"
  map global fold-mode f :fold-add-selection<ret>  -docstring "Fold Selection"
  map global fold-mode x :fold-remove-selection<ret>  -docstring "Unfold Selection"
  map global fold-mode r :fold-reset<ret>  -docstring "Reset Folds"
  map global fold-mode m :fold-matching<ret>  -docstring "Fold 'm'"
}
