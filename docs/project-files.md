# Project files (M2.4)

How C# Plus handles .NET project files: which suffixes map to which language,
what editing support each gets, and why classic `.sln` deliberately has none.
The task templates referenced here are documented in
[`docs/dotnet-workflow.md`](./dotnet-workflow.md); per-release verification is
[`docs/release-smoke.md`](./release-smoke.md).

## Coverage

| Suffix | Zed language | Grammar | Highlights | Outline / indents | Tasks | Known gaps |
| --- | --- | --- | --- | --- | --- | --- |
| `.csproj` | C# Project File | `tree-sitter-xml` @ `863dbc38` | generic XML + MSBuild vocabulary | yes (curated) / yes | restore, build, test, run, watch, clean, publish, format, EF migration (on the project file) | no MSBuild property/expression semantics — `$(...)` and conditions are highlighted as text, not evaluated or completed |
| `.props` `.targets` `.proj` | MSBuild File | `tree-sitter-xml` @ `863dbc38` | generic XML + MSBuild vocabulary | yes (curated) / yes | none by design — these are imported fragments; `dotnet build` against one fails | same as `.csproj` |
| `.slnx` | C# Solution File | `tree-sitter-xml` @ `863dbc38` | generic XML + solution vocabulary | yes (projects + deployment targets) / yes | restore, build, test, clean, publish, format (on the solution file) | minimal vocabulary: only `Solution`, `Project`, `Deployment`, `Target` elements and `Path`/`Configuration`/`Platform`/`Name` attributes are styled beyond generic XML |
| `.sln` | — (plain text) | — | none | none | none on the file itself | see the decision below; solution-level commands run against `.slnx` or via any task with an explicit path |

All three XML-backed languages also get XML block comments, `<`/`>` bracket
pairs, and hard tabs (matching MSBuild conventions) from their `config.toml`.

## What the highlighting/outline actually does

The xml grammar is generic: element and attribute names are opaque `Name`
tokens. Meaning is attached afterwards with text predicates — case-sensitive,
on purpose, because MSBuild and `.slnx` element names are PascalCase by
convention. In practice:

- Structural elements (`Project`, `PropertyGroup`, `ItemGroup`, `Target`,
  `Import`, `Using`, `Choose`/`When`/`Otherwise`, …) read as keywords;
  `Solution` and `Deployment` in `.slnx` files likewise.
- Elements inside a `Target` read as function calls (`Message`, `Copy`, `Exec`).
- Controlling attributes (`Condition`, `Include`, `Version`, `Sdk`,
  `DependsOnTargets`, …; `Path`/`Configuration`/`Platform` in solutions) read
  as attributes.
- The outline lists what you navigate by, not every element: property/item
  groups, `Target` by its `Name`, `PackageReference`/`ProjectReference` by
  their `Include`, the root `Project` by its `Sdk`, and each `.slnx` project
  entry by its `Path`.

## Why one shared query file

Zed resolves a language's query files (`highlights.scm`, `outline.scm`,
`indents.scm`, …) from that language's own directory only. A query file
dropped into `languages/msbuild/` does nothing for `.csproj` buffers even
though both use the xml grammar. The extension therefore keeps one canonical
set of project-file queries in `languages/msbuild/`, and `languages/csproj/`
and `languages/slnx/` point at them with symlinks — the same mechanism the
upstream C# extension shipped. Editing the query files means editing
`languages/msbuild/*.scm` only; the other directories follow, and the MSBuild
and `.slnx` vocabulary sections inside them are deliberately disjoint so no
element matches (or snapshots) twice.

Every change is enforced by the snapshot harness (`harness/run.py`): the
`msbuild`, `csproj`, and `slnx` languages each run their resolved queries over `corpus/msbuild/`
against committed goldens. After editing queries:

```sh
python3 harness/run.py --update
python3 harness/run.py --check
```

## Classic `.sln`: deliberately not owned (decided 2026-09-06)

C# Plus does **not** register a language for classic `.sln` solution files.
They open as plain text. Evidence checked on 2026-09-06:

- **Zed core gives `.sln` nothing.** A full path listing of
  `zed-industries/zed` @ `main` contains no `sln`-related file, the built-in
  language set (`crates/grammars/src/*/config.toml` — 22 languages: bash, c,
  cpp, css, diff, gitcommit, go, gomod, gowork, javascript, jsdoc, json,
  jsonc, markdown, markdown-inline, python, regex, rust, tsx, typescript,
  yaml, zed-keybind-context) includes neither XML nor C#, and there is no
  hardcoded `sln` handling in `crates/project/src` or `crates/language/src`.
- **No other extension claims it.** The XML extension
  (`sweetppro/zed-xml`, registry entry `[xml]`) declares
  `path_suffixes = ["xml"]` only. The upstream C# extension
  (`zed-extensions/csharp`, fork point `88597e1`) registered `csproj`,
  `proj`, `props`, `targets`, `slnx` — never `sln`.

So taking ownership of `.sln` would not duplicate anyone's work — it would
*create* ownership with nothing behind it:

1. `.sln` is not XML. The xml grammar and every query above are inapplicable,
   so "ownership" would mean writing and maintaining a bespoke parser for a
   legacy proprietary format indefinitely — the kind of scope this roadmap
   explicitly avoids.
2. It buys nothing functional. Solution discovery for the language servers is
   server-side (Roslyn's `--autoLoadProjects`, OmniSharp's solution handling),
   independent of Zed's syntax association; a plain-text `.sln` loses no
   capability.
3. The extension's solution story is `.slnx`: the task templates attach to
   solution files there (`languages/slnx/tasks.json`), matching where the
   .NET SDK itself is heading. For repositories still on `.sln`, any task
   from [`docs/dotnet-workflow.md`](./dotnet-workflow.md) can be run with an
   explicit path, e.g. `dotnet build MySolution.sln`.

**Revisit trigger:** a built-in Zed `.sln` association, a maintained
tree-sitter grammar for the format, or user evidence that plain text is a
real drag on solution navigation would be reasons to re-open this.
