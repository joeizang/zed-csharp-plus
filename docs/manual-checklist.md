# Manual per-release checklist

The "Manual, per release, checklist-driven" bucket of the quality gates:
the semantic-operation row and the lifecycle row, on the developer's primary
OS, with the other two OSes covered by CI-executable smoke only. Everything
with no home in a bucket is either automated as part of the milestone that
introduced it or removed from the matrix — an unrunnable gate is not a gate.

Fill in the date and OS at the bottom; paste the filled checklist into the
release's changelog entry.

## Scope for 1.3.0 (editing-only release)

No Razor language server exists in this release, so the semantic-operation
row applies to **C# files**; Razor is verified for the editing row only.

## Editing row (Razor)

- [ ] Open `.razor` and `.cshtml` from `corpus/razor/well-formed/`: directives,
      expressions, `@code`/`@functions`, components, Tag Helpers, comments,
      `@@` escapes, `@media`, `<script>` all style sensibly; no garbage
      highlight leakage across language boundaries.
- [ ] Outline shows `@section` entries and C# members inside `@code`.
- [ ] Bracket matching pairs `{}`, `()`, `[]`, quoted attribute values, and
      element `<` ↔ `</`.
- [ ] Auto-indent: Enter inside an `@if {` block and inside a `<div>` indents
      sensibly; the closing `}` / `</div>` dedents.
- [ ] Typing a half-typed component tag (`<Foo` then newline) does not
      auto-insert a broken close tag or corrupt the buffer.
- [ ] Snippets: `@page`, `@model`, `@code` expand with correct tabstops; the
      snippet list does NOT appear in `.cs` or HTML buffers (scoped to Razor).
- [ ] Pathological files from `corpus/razor/pathological/` open without
      breaking the editor; editing after the broken construct works.

## Semantic-operation row (C# files, Roslyn)

Completion, hover, diagnostics, definition, references, rename, code actions,
semantic tokens, folding — exercise each once on a fixture project
(M0.3b fixtures when available; any project otherwise):

- [ ] Completion offers C# symbols from the project.
- [ ] Hover shows types/docs.
- [ ] Diagnostics appear and clear.
- [ ] Go-to-definition lands correctly (project + dependency symbol).
- [ ] Find-references returns project-local results.
- [ ] Rename applies within the project.
- [ ] Code actions apply a fix without corrupting the file.
- [ ] Folding ranges work inside methods/regions.

## Lifecycle row

- [ ] Cold open of a fixture solution; server becomes ready without errors.
- [ ] Restore/build run via the shipped tasks (or manually pre-M0.3b).
- [ ] Project reload: add a new `.cs` file; new symbols resolve.
- [ ] External edits (git branch switch) reflected without restart errors.
- [ ] Two worktrees open on the same solution: independent servers, no
      cross-talk errors.
- [ ] Offline mode after first run: server starts from cache.
- [ ] Stale/missing generated state: `obj/`/`bin/` deleted, then reopen —
      recover or explain, never a broken buffer.
- [ ] Server mismatch: point `lsp.roslyn.binary.path` at a non-server binary —
      a clear error, files stay editable.

## Failure row (editing-support invariants)

- [ ] Unsupported SDK: set `global.json` to a nonexistent SDK; open a `.cs`
      file — files stay editable, the failure names a concrete remediation.
- [ ] Server launch failure: invalid `lsp.roslyn.binary.path` — clear error,
      no retry storm.
- [ ] Disabled Roslyn: `"language_servers": ["!roslyn"]` for CSharp —
      OmniSharp/csharp-ls path still works or fails cleanly.
- [ ] Conflicting Razor extension installed (community Razor) — association
      ambiguity is visible/explainable; C# Plus never silently modifies
      settings or other extensions.
- [ ] No telemetry, no automatic restore/build on file open, no project
      content upload.

## Release blockers check (must all be "no")

- [ ] Incorrect mapped edit, range, diagnostic, or navigation target observed?
- [ ] Formatting outside the intended range? (n/a this release — no Razor
      formatting)
- [ ] Crash, unbounded memory growth, or material startup/large-solution
      regression?
- [ ] Silent server fallback, automatic build/restore, telemetry, or
      project-content upload?
- [ ] Regression in ordinary C# behavior (harness covers queries; this covers
      runtime behavior)?
- [ ] Any language server starting for Razor buffers in a release that does
      not advertise Razor semantics? (must be NO — verify in the process list)

## Record

- Date: ____
- Primary OS/arch: ____
- Harness/CI commit: ____
- Result: ____
