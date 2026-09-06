# Migration and coexistence (M1.4)

How to move to C# Plus without conflicts, and which extensions are meant to
stay installed alongside it. None of the steps below modify your `settings.json`
or any file on disk; everything happens through Zed's extension list
(`zed: extensions` from the command palette, or **Zed > Extensions**).

## (a) From the upstream "C#" extension — required

**Uninstalling is required, not optional.** C# Plus is a fork of
[zed-extensions/csharp](https://github.com/zed-extensions/csharp) and registers
the same languages and file suffixes: `.cs`, `.csproj`, `.slnx`, and MSBuild
files (`.proj`, `.props`, `.targets`). If both are installed, two extensions
claim ownership of the same files: you get duplicate language entries, and Zed
may attach the wrong grammar or offer both extensions' language-server entries
for the same buffer. The conflict is silent enough to be confusing and serves
no purpose — C# Plus is a superset of the upstream extension.

Steps:

1. Open the extensions list: `zed: extensions` from the command palette.
2. Search for **C#** (extension id `csharp`, published by Zed Industries) and
   choose **Uninstall**.
3. Install **C# Plus** (extension id `csharp-plus`) from the same page.
4. Restart Zed (or run `zed: reload`) so no stale grammar registrations remain.
5. Open a `.cs` file and confirm the status-bar language shows **CSharp**, and
   open a `.razor` or `.cshtml` file and confirm it shows **Razor**.

**Your settings carry over unchanged.** C# Plus inherits the upstream server
ids verbatim, so every `lsp.omnisharp`, `lsp.roslyn`, and `lsp.csharp-ls`
block in your `settings.json` — including `binary.path` overrides,
initialization options, and Roslyn settings mapped through the `csharp|`
prefix — is read exactly as before. `languages.CSharp` settings (tab size,
bracket behavior, and so on) also apply unchanged because the language name is
identical. Do not edit or duplicate these settings during migration; nothing
about them needs to change.

The one visible difference: C# Plus additionally registers the Razor language
(`.razor`, `.cshtml`), which the upstream extension does not provide at all.

## (b) From separate Razor extensions — uninstall before installing

Three community extensions publish Razor support for Zed. Each registers the
`.razor` and/or `.cshtml` suffixes, so they conflict with C# Plus on Razor file
associations the same way two owners of `.cs` conflict. If any of them is
installed, uninstall it **before** installing C# Plus:

| Extension name in the list | Extension id | Source |
| --- | --- | --- |
| Razor Pages | `razor` | [noundry/zed-razor](https://github.com/noundry/zed-razor) |
| ASP.NET Razor Syntax | `razor-syntax` | [NoisKung/razor-syntex-zed](https://github.com/NoisKung/razor-syntex-zed) |
| Razor & Blazor | `razor` | [IbrahimSabriOrene/zed-razor-treesitter](https://github.com/IbrahimSabriOrene/zed-razor-treesitter) |

Two of these have historically been published under the id `razor`, so match
by the **name shown in your extension list** rather than by id alone.

Steps:

1. Open the extensions list and search for each name above.
2. For every one that appears under **Installed**, choose **Uninstall**.
3. Install **C# Plus**, then restart Zed (or run `zed: reload`).
4. Verify: re-open the extensions list, clear the search, and confirm none of
   the three names remain in the installed section. Then open a `.razor` or
   `.cshtml` file and confirm the status-bar language is **Razor** and that
   highlighting, the outline panel, and snippets come from C# Plus.

Leaving one installed does not usually break either extension outright, but it
produces duplicate grammars for the same suffixes and unpredictable wins for
which one owns the buffer — the same duplicate-ownership problem as (a), and
just as pointless to keep.

## (c) Coexistence — what stays installed

C# Plus deliberately does **not** register HTML, CSS, JavaScript, TypeScript,
or debugging support, and does not replace the extensions that provide them:

| Concern | Owner | C# Plus role |
| --- | --- | --- |
| HTML (`.html`) | Zed's built-in HTML language or your HTML extension | none — Razor files inject HTML highlighting only |
| CSS (`.css`) | Zed's built-in CSS language or your CSS extension | none — `<style>` content in Razor is injected CSS |
| JavaScript (`.js`) | Zed's built-in JavaScript language | none — `<script>` content in Razor is injected JS |
| TypeScript (`.ts`, `.tsx`) | Zed's built-in TypeScript language | none |
| Debugging | Third-party debug adapter extensions | none — C# Plus ships no debugger |

Keep those extensions installed. Inside a Razor buffer, HTML/CSS/JS content is
highlighted by injecting their grammars into the corresponding ranges; no
registration or setting change is needed for that, and C# Plus never takes
file associations away from those languages. If you previously installed a
Razor extension from table (b) purely to get HTML-like highlighting in
`.cshtml` files, C# Plus replaces it outright.
