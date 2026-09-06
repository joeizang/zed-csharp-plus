("(" @open
  ")" @close)

("[" @open
  "]" @close)

("{" @open
  "}" @close)

("\"" @open
  "\"" @close)

; HTML element open/close pair: "<" and "</" are sibling anonymous tokens of
; the same (element) node. A generic ("<" ">" @close) pair is deliberately
; omitted: in Razor markup it would mispair "<" from comparisons and the open
; tag with unrelated ">" tokens across mixed HTML and C# content.
("<" @open
  "</" @close)
