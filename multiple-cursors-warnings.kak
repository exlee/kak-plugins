provide-module multiple-cursors-warning %{
  try %{
    remove-hooks multiple-cursors-warning
  }
  hook -group multiple-cursors-warning global NormalIdle .* %{
      evaluate-commands %sh{
          if [ "$kak_selection_count" -gt 1 ]; then
              # Multi-cursor mode: Force PrimaryCursor to Red
              # echo "set-face window PrimaryCursor white,red+F"
              # echo "set-face window PrimaryCursorEol white,red+F"
              echo "set-face window StatusLine black,yellow+F"
              echo "set-face window StatusCursor yellow,black+F"
          else
              # Single-cursor mode: Remove window override (fall back to theme)
              # echo "unset-face window PrimaryCursor"
              # echo "unset-face window PrimaryCursorEol"
              echo "unset-face window StatusLine"
              echo "unset-face window StatusCursor"
          fi
      }
  }
}
