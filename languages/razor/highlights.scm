; Razor highlights for Zed — C# Plus (M1.2).
;
; Adapted for Zed from the vendored grammar's own queries,
; queries/highlights.scm of joeizang/tree-sitter-razor, the fork of
; tris203/tree-sitter-razor @ d4664e409caaea12f73c9525484e3cf88b1cf718
; (MIT, © 2023 Tristan Knight; see that repository's LICENSE and
; docs/razor-grammar-audit.md). Changes vs upstream:
;   - directive tokens: @constant.macro → @keyword.directive; control-flow
;     groups collapsed to @keyword, matching this repository's
;     languages/csharp/highlights.scm conventions;
;   - upstream's @keyword.coroutine special case for `await` inside implicit
;     expressions is dropped: the inherited C# patterns already highlight it;
;   - upstream's (taghelper_wildcard) @character.special is dropped: it fires
;     inside directive target values, which C# string styling covers;
;   - "at_at_escape" (the `@@` escape) moved to @string.escape;
;   - added element/attribute-delimiter patterns (upstream has none): tag
;     names and plain HTML attribute names are unnamed `_tag_name`/
;     `_html_attribute_name` tokens and CANNOT be captured — query
;     wildcards match named nodes only, so a noundry-style
;     `(element "<" . (_) @tag)` would wrongly capture whole child
;     elements instead (verified with the tree-sitter CLI). `<style>`/
;     `<script>` bodies are likewise unnamed (see injections.scm for the
;     CSS/JS injection limitation);
;   - deliberately no (ERROR) capture: highlighting incomplete templates
;     red while typing would be noisier than the signal it gives.
;
; `; inherits: c_sharp` — the razor grammar embeds tree-sitter-c-sharp rules
; natively, so Zed applies the registered CSharp language's patterns to all
; embedded C# node types. (The tree-sitter CLI treats this line as a comment
; in `query` runs; it is resolved by Zed at query load time.)

; --- Comments ---
[
  (razor_comment)
  (html_comment)
] @comment

; --- Page-level directives (@page, @model, @using, @inject, @inherits,
; @attribute, @implements, @layout, @namespace, @typeparam, @rendermode,
; @preservewhitespace, the TagHelper directives, and @code/@functions) ---
[
  "at_page"
  "at_using"
  "at_model"
  "at_rendermode"
  "at_inject"
  "at_implements"
  "at_layout"
  "at_inherits"
  "at_attribute"
  "at_typeparam"
  "at_namespace"
  "at_preservewhitespace"
  "at_addtaghelper"
  "at_removetaghelper"
  "at_taghelperprefix"
  "at_block"
] @keyword.directive

; --- Control flow in blended markup ---
[
  "at_if"
  "at_switch"
  "at_for"
  "at_foreach"
  "at_while"
  "at_do"
  "at_lock"
  "at_section"
  "at_try"
  "catch"
  "finally"
] @keyword

; --- Transitions and escapes ---
; The bare `@` sigil of implicit (`@x`) and explicit (`@(x)`) expressions;
; upstream's @variable is kept so the sigil renders with the expression
; it opens.
[
  "at_implicit"
  "at_explicit"
] @variable

; `@:` explicit line transition back to markup
"at_colon_transition" @keyword

; `@@` literal-@ escape
"at_at_escape" @string.escape

; --- Blazor render modes (@rendermode InteractiveServer, ...) ---
(razor_rendermode) @constant

; --- Razor attributes (@bind, @onclick, @key, @ref, @attributes, ...) and
; their event modifiers (:preventDefault, :stopPropagation, :culture) ---
[
  (razor_attribute_name)
  (razor_attribute_modifier)
] @attribute

; --- CSS at-rules in markup (grammar fork fix 2: @media, @keyframes, ...) ---
(css_at_rule
  "@" @keyword
  (element) @keyword)

; --- HTML elements ---
; Tag names are unnamed tokens and cannot be captured (header note); the
; delimiters, separators, and attribute-value quotes below still give the
; tag structure its color.
(element "<" @punctuation.delimiter)
(element "</" @punctuation.delimiter)
(element ">" @punctuation.delimiter)
(element "/>" @punctuation.delimiter)
(element "=" @punctuation.delimiter)
; Attribute-value quotes; the value text itself is unnamed and unstylable,
; while Razor expressions inside values are styled by the inherited C# rules.
(element "\"" @string)
