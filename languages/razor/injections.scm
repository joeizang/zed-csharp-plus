; Razor injections for Zed — C# Plus (M1.2).
;
; Adapted from the vendored grammar's own queries,
; queries/injections.scm of joeizang/tree-sitter-razor, the fork of
; tris203/tree-sitter-razor @ d4664e409caaea12f73c9525484e3cf88b1cf718
; (MIT, © 2023 Tristan Knight). This file is deliberately a no-op; the
; upstream patterns are not carried over, for the reasons below.
;
; 1. HTML: the razor grammar parses HTML elements natively (`element` nodes
;    with attributes and children), so injecting `html` would double-
;    highlight markup and would duplicate the HTML extension's ownership.
;    Upstream's `(element) @injection.content` html injection is therefore
;    deliberately NOT carried over.
; 2. CSS/JS: `<style>` and `<script>` bodies are sequences of `_html_text`
;    tokens, which are UNNAMED in the grammar (verified in
;    the grammar's src/node-types.json), so no named
;    injection.content node exists for them — CSS/JS injections are
;    impossible at grammar level. Partial coverage instead: the fork's
;    `css_at_rule` node (fork fix 2) is highlighted in highlights.scm, and
;    the rest of those bodies renders as plain text. Naming a style/script
;    content node in the grammar fork is the only route to real CSS/JS
;    injections; that is a grammar-level change, tracked with the fork.
; 3. C#: embedded natively via tree-sitter-c-sharp. highlights.scm keeps
;    `; inherits: c_sharp`, so no C# injection is needed here.
; 4. Comments: `razor_comment`/`html_comment` bodies are unnamed text and
;    are styled directly as @comment in highlights.scm. Upstream's
;    `(#set! injection.language "comment")` is not carried over: Zed core
;    registers no `comment` language, so that injection is a guaranteed
;    no-op in Zed.
