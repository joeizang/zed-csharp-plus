# Known limits (M1.4)

What this release of C# Plus does and does not do for Razor files. Each limit
is a scope decision, not a bug; where a limit comes from the vendored grammar,
the source is [razor-grammar-audit.md](razor-grammar-audit.md).

## This release is editing-only

**No language server starts for Razor buffers.** Opening, editing, or saving a
`.razor` or `.cshtml` file never launches Roslyn, OmniSharp, `csharp-ls`, or
any other server for that buffer, and never triggers restore or build work.
What you get is what tree-sitter can provide offline:

- syntax highlighting for Razor, HTML, CSS, and embedded C#,
- document outline (sections and C# members inside `@code`/`@functions` blocks),
- bracket matching, auto-indentation, and text objects,
- the Razor snippets documented in the README.

There is **no completion, hover, diagnostics, go-to-definition, find-references,
rename, or code actions for Razor content** in this release.

Semantic Razor support (project-aware completion, hover, diagnostics, and so
on, served by Roslyn) is planned as a separate, explicit opt-in release —
see `BACKLOG.md`, Milestones 3 and 4. It will never be switched on silently.

## No Razor formatting

C# Plus ships no formatter for Razor files, and format-on-save will not
reformat them. Razor formatting needs projected-document mapping that does not
exist yet (see the backlog's M3.4/M4.3); enabling it before that is proven
correct is a release blocker we refuse on purpose.

## Legacy ASP.NET / .NET Framework Razor

Razor targeting classic ASP.NET or .NET Framework (`.aspx` Web Forms, `.vbhtml`,
`@helper` syntax, and pre-v3 compilers) is **unsupported**. Some files may
highlight acceptably because the grammar covers the shared core syntax, but no
behavior is tested or promised for them, and they will never receive semantic
support. .NET 8 LTS and newer is the supported target.

## Grammar-level limits

The Razor grammar is a vendored fork of
`tris203/tree-sitter-razor` (MIT), pinned and fixed as described in
[razor-grammar-audit.md](razor-grammar-audit.md). Its known containment limits
apply unchanged:

- **Unterminated attribute quote.** If a quoted attribute value is left open
  (for example `<a href="` typed and not closed), the rest of the file can be
  swallowed into one error span and the trailing markup does not recover to
  correct node types (`broken-attribute.cshtml` in the corpus demonstrates
  this). The failure is contained to highlighting/structure appearance; it
  does not affect editability. A grammar-level fix was evaluated and rejected
  because terminating quoted values at `>` would break legitimate `>` inside
  attribute values.
- **Pathological files can error to end-of-file.** In the 10-file pathological
  corpus, 3 of 10 files have an outermost error span reaching EOF. For two of
  them that is the only correct parse (the file really is unterminated); the
  third is the attribute-quote case above. Constructs *after* a broken region
  generally still parse to their correct node types.

## Editing never breaks

Whatever the grammar does with a malformed file, the buffer remains fully
editable: queries cannot reject keystrokes, and the structural-editing queries
(brackets, indents, outline, text objects) are read-only views over the parse
tree. Worst case, highlighting and indentation look wrong until the file is
well-formed again; nothing is corrupted, and no server restart or setting
change is involved.
