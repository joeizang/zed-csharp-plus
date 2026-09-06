# Zed C# Plus

A [C#](https://learn.microsoft.com/en-us/dotnet/csharp/) and
[Razor](https://learn.microsoft.com/en-us/aspnet/core/mvc/views/razor) extension
for [Zed](https://zed.dev), covering C#, Razor (`.cshtml` / `.razor`), and .NET
project files.

## Features

- **C#** — language servers (Roslyn, OmniSharp, `csharp-ls`), syntax
  highlighting, outline, bracket matching, auto-indent, and text objects, as
  before.
- **Razor (`.cshtml` / `.razor`)** — tree-sitter syntax highlighting with HTML,
  CSS, and JavaScript injections, document outline, bracket matching,
  auto-indentation, text objects, and snippets. This is **editing-only**
  support: no language server runs for Razor buffers yet
  ([known limits](docs/known-limits.md)).
- **.NET project files** — `.csproj`, `.slnx`, and MSBuild `.proj` / `.props` /
  `.targets` with restore/build tasks.

## Supported files

| Suffixes | Language | Notes |
| --- | --- | --- |
| `.cs` | CSharp | Language servers per your `lsp.*` settings |
| `.razor`, `.cshtml` | Razor | Editing-only in this release; no language server |
| `.csproj` | C# Project File | Restore/build tasks |
| `.proj`, `.props`, `.targets` | MSBuild File | |
| `.slnx` | C# Solution File | Restore/build tasks |

HTML, CSS, JavaScript, TypeScript, and debugging remain owned by their own
extensions; C# Plus does not register them. See
[docs/migration.md](docs/migration.md) for moving from the upstream C#
extension or from separate Razor extensions, and
[docs/known-limits.md](docs/known-limits.md) for what Razor support does not
include yet.

## Provenance

This is a fork of [zed-extensions/csharp](https://github.com/zed-extensions/csharp)
at commit `88597e1` (version 1.2.2), which remains under the Apache-2.0 licence
reproduced in [LICENSE](LICENSE). Original authorship is retained in
`extension.toml`. Modifications made in this fork are tracked in the Git history
and summarised per release in the changelog.

## Relationship to the upstream C# extension

C# Plus is a **superset** of the upstream extension, not a companion to it. It
registers the same file suffixes (`.cs`, `.csproj`, `.slnx`, MSBuild files), so
installing both will produce duplicate language ownership. Install one or the
other:

1. Uninstall the **C#** extension from Zed's extension list.
2. Install **C# Plus**.

Your existing `lsp.omnisharp` / `lsp.roslyn` / `lsp.csharp-ls` settings continue
to work unchanged. Full step-by-step instructions, plus how to migrate from
separate Razor extensions, are in [docs/migration.md](docs/migration.md).

## Development

To develop this extension, see the [Developing Extensions](https://zed.dev/docs/extensions/developing-extensions)
section of the Zed docs.

### Staying current with upstream

This fork tracks two remotes: `origin` is
[joeizang/zed-csharp-plus](https://github.com/joeizang/zed-csharp-plus),
`upstream` is [zed-extensions/csharp](https://github.com/zed-extensions/csharp).

```sh
git fetch upstream && git merge upstream/main
```

See `BACKLOG.md`, item D0.1, for the upstream-sync policy.
