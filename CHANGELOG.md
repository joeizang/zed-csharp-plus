# Changelog

All notable changes to the C# Plus fork are documented here. Each entry that
followed an upstream sync records the merged upstream commit (D0.1 policy).

## 1.3.0 — Razor editing and the .NET workflow (Milestones 0 + 1 + 2)

The first fork release. Delivers the Milestone 0 foundation (fork hygiene,
corpus, grammar decision, fixtures, compatibility contract), first-class Razor
*editing* (Milestone 1), and the everyday .NET workflow improvements
(Milestone 2). Razor support in this release is editing only, not full Razor
IDE semantics: **no language server starts for Razor buffers**
(see `docs/razor-contract.md`). The M0.0 feasibility spike remains open and
does not gate this release.

No upstream commits were merged for this release; the fork point remains
upstream `88597e1` (v1.2.2).

#### Added
- Razor language (`Razor`) registered for `.razor` and `.cshtml` with the
  forked `tree-sitter-razor` grammar, pinned by commit from its own
  repository (M1.1; decision and fitness numbers in
  `docs/razor-grammar-audit.md`, gate **G1 = fork**).
- Razor syntax highlighting (`highlights.scm`) with embedded C# highlighted
  via `; inherits: c_sharp` — directives, expressions, control flow, `@code`/
  `@functions` blocks, comments, `@@` escapes, CSS at-rules (M1.2).
- Razor structural editing: bracket pairs (including `<` ↔ `</` element
  pairs), auto-indentation for blocks and elements, outline items for
  `@section` and C# members inside `@code`/`@functions`, text objects, and
  Razor comments as comment text objects (M1.3).
- 14 conservative Razor snippets, scoped to the Razor language
  (`languages/razor/razor.json`) (M1.4).
- Highlight snapshot harness: golden files over the Razor corpus plus the
  existing `csharp` and `msbuild` queries, `--check`/`--update` modes, and a
  deliberate-regression verification (M1.0). First real CI
  (`.github/workflows/ci.yml`): snapshot check + grammar's 79 upstream tests
  on Linux, crate build on macOS/Linux/Windows, `cargo test`.
- Sample corpora for C# and MSBuild snapshots (`corpus/csharp/`,
  `corpus/msbuild/`).
- Documentation: migration guide (off the upstream C# extension and off the
  community Razor extensions) and known limits (M1.4).
- Buildable fixture solutions (M0.3b): MVC, Razor Pages, Blazor Web App,
  Blazor WASM, Razor Class Library, and a multi-project `.slnx` solution,
  all targeting .NET 8 and verified to build offline after first restore
  (`scripts/verify-fixtures.sh`), plus generated-state and project-reload
  lifecycle cases.
- Everyday .NET workflow (Milestone 2): Roslyn-first guidance with a
  remediation matrix; actionable diagnostics for every server failure mode
  (missing SDK / invalid `global.json` / feed unreachable / corrupt package
  layout / missing binary — each names itself and one concrete fix); nine
  project task templates and six solution task templates (restore, build,
  test, run, watch, clean, publish, format, EF migrations) with no auto-run
  and no secrets; debugging state documentation (no usable .NET debug
  adapter for Zed today, dated and evidenced); project-file coverage audit
  (MSBuild highlighting/outline strengthened, `.sln` deliberately not
  owned).

#### Fixed
- Task templates quote `$ZED_FILE` / `$ZED_WORKTREE_ROOT`; tasks run through
  the system shell, so an unquoted path containing spaces word-split and the
  task acted on the wrong path (or failed).

#### Decisions recorded
- **G1** (M0.2): fork `tris203/tree-sitter-razor` @ `d4664e4` into
  [joeizang/tree-sitter-razor](https://github.com/joeizang/tree-sitter-razor)
  (MIT); three grammar fixes applied; post-fix the fork parses 14/14
  well-formed corpus files with zero errors and passes upstream's 79 tests.
- **M0.4**: the Razor opt-in mechanism is release-gated manifest attachment
  with a user-side off switch (`languages.Razor.language_servers =
  ["!roslyn"]`); extension-side conditional attachment rejected.
  See `docs/razor-contract.md`.

#### Fork infrastructure (D0.1)
- Replaced upstream's `bump_version` workflow (gated on Zed org ownership and
  Zed-internal secrets; could never run in the fork) with the manual
  convention in `scripts/bump-version.sh` + `docs/versioning.md`.
- The Razor grammar fork lives in its own repository, [joeizang/tree-sitter-razor](https://github.com/joeizang/tree-sitter-razor), and is pinned
  by commit from `extension.toml` exactly as `c_sharp` and `xml` already are.
  It is deliberately not vendored: a vendored copy is ~48 MB that every user
  would clone at install, and an in-repo pin can only name a commit that
  exists after the merge introducing it.
- The snapshot harness clones every pinned grammar (razor, c_sharp, xml) into
  `harness/workspaces/`; goldens are tree-sitter CLI version-sensitive and CI
  pins the CLI exactly (0.27.0).

#### Known limits (details in `docs/known-limits.md`)
- Tag names and plain HTML attribute names are unnamed tokens in the grammar,
  so they are styled via delimiters/quotes only; real tag-name highlighting
  requires a grammar change (tracked with the CSS/JS content-node limitation).
- CSS/JS injections into `<style>`/`<script>` are not possible at grammar
  level (bodies are unnamed text tokens); the `css_at_rule` fork fix is the
  partial mitigation.
- Unterminated attribute quotes can swallow the remainder of a file during
  error recovery (inherited from upstream; measured and documented).

## 1.2.2

Fork point: upstream `zed-extensions/csharp` at commit `88597e1` (v1.2.2).
Upstream history before this entry belongs to the upstream project.
