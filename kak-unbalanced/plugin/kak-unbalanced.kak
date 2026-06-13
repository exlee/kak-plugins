provide-module kak-unbalanced %{
    set-face global Unbalanced Error

    declare-option -hidden range-specs kak_unbalanced_ranges
    declare-option -docstring "Command used to run the kak-unbalanced utility" \
        str kak_unbalanced_command kak_unbalanced

    define-command -hidden kak-unbalanced-update %{
        try %{ add-highlighter buffer/kak-unbalanced ranges kak_unbalanced_ranges }
        evaluate-commands -draft %{
            execute-keys <percent>
            evaluate-commands %sh{
                printf %s "$kak_selection" | "$kak_opt_kak_unbalanced_command"
            }
        }
    }

    hook -group kak-unbalanced global NormalIdle .* kak-unbalanced-update
    hook -group kak-unbalanced global InsertIdle .* kak-unbalanced-update
    hook -group kak-unbalanced global WinCreate .* %{
        try %{ add-highlighter buffer/kak-unbalanced ranges kak_unbalanced_ranges }
    }
}
