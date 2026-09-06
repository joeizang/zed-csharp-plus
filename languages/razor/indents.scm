(_
  "{"
  "}" @end) @indent

(_
  "["
  "]" @end) @indent

(_
  "("
  ")" @end) @indent

; Indent element content between the open and close tags. Self-closing
; elements have no "</" sibling token and therefore never match.
(element
  "<" @start
  "</" @end) @indent
