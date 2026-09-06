# Razor compatibility contract (M0.4)

Status: **decided 2026-09-05**. This document is the deliverable for M0.4; it
resolves the opt-in contradiction and defines the user-visible states. The
M1.1 manifest already implements the editing-only state.

## The opt-in contradiction, resolved

Attaching Roslyn to Razor requires `extension.toml` to carry
`languages = ["CSharp", "Razor"]` under `[language_servers.roslyn]` plus a
`[language_servers.roslyn.language_ids]` map (`"Razor" = "aspnetcorerazor"`).
The moment `"Razor"` appears there, Zed starts Roslyn for every Razor buffer —
so an "editing-only" release (M1.5) and an "opt-in semantics" release (M3.4)
cannot both hold under one manifest.

**Extension-side conditional attachment is rejected.** The only candidate
mechanism — returning an error from `language_server_command` when a worktree
setting is absent — surfaces as an error dialog in Zed, not a graceful no-op,
and the backlog explicitly rules that out as an opt-in experience.

**Chosen mechanism (named for the record):**

1. **Release-gated manifest attachment.** Razor is attached to Roslyn only in
   the release that ships experimental semantics (M3.4). Editing-only
   releases ship the Razor language with **no server attached** — this is
   what "opt-in" means for M1: the opt-in is upgrading to a later release.
2. **User-side off switch.** From the release that attaches Roslyn to Razor,
   the documented way to turn semantics off is the Zed settings edit:
   ```json
   {
     "languages": {
       "Razor": {
         "language_servers": ["!roslyn"]
       }
     }
   }
   ```
   This is graceful (Zed simply does not attach the server), reversible, and
   requires no extension-side code. It will be demonstrated in the M3.4
   release notes and in `docs/known-limits.md`.
3. **Demonstrated.** The editing-only state is demonstrated by the current
   manifest: `[grammars.razor]` exists, `languages/razor/` is registered via
   `config.toml`, and no `language_servers.*` entry mentions Razor — so no
   server can start for a Razor buffer. The experimental state is
   demonstrated by the M3.4 manifest delta recorded below.

**M3.4 manifest delta (planned, not applied in this release):**

```toml
[language_servers.roslyn]
name = "Roslyn"
languages = ["CSharp", "Razor"]

[language_servers.roslyn.language_ids]
"Razor" = "aspnetcorerazor"
```

## User-visible states

Each state: exact message, enabled capabilities, recovery action. Messages are
the *contract* — user-facing copy must match them.

### 1. Editing-only (this release)

- **Message:** "Razor editing is active: syntax highlighting, outline,
  brackets, auto-indent, and snippets. Project-aware features (completion,
  hover, diagnostics) for `.razor`/`.cshtml` are not available yet; C# files
  have full language-server support."
- **Capabilities:** grammar highlighting, injections design (native C#
  highlighting), brackets, indentation, outline, text objects, snippets.
- **Server state:** none attached to Razor. Ordinary `.cs` behavior unchanged.
- **Recovery action:** none needed; to get semantics, upgrade to the release
  advertising experimental Razor semantics.

### 2. Experimental semantics (M3.4 release)

- **Message:** "Experimental Razor semantics are enabled: Roslyn serves
  `.razor`/`.cshtml` with the `aspnetcorerazor` language id. Capabilities are
  limited to those proven safe (see the release notes); formatting and mapped
  edits are withheld. To turn this off: set
  `\"languages\": { \"Razor\": { \"language_servers\": [\"!roslyn\"] } }`."
- **Capabilities:** only the operations proven safe by the M3.2 probe; the
  release notes carry the evidence table. Formatting and mapped edits are
  withheld unless virtual/projected document mapping is verified.
- **Recovery action:** the settings edit above; turning it off returns
  immediately to complete editing support with no data loss and no
  configuration cleanup.

### 3. Stable semantics (M4.3 release, gated)

- **Message:** "Razor semantics are stable, backed by Roslyn. Formatting is
  enabled only after correct formatting edits are proven. OmniSharp and
  csharp-ls remain C#-only."
- **Capabilities:** full proven matrix per `docs/quality-gates.md`.
- **Recovery action:** none; the same settings edit remains available.

### 4. Unsupported environment

Trigger: missing/unsupported .NET SDK, invalid `global.json`, project-load
failure, legacy ASP.NET/.NET Framework Razor.

- **Message:** names the failure and one concrete next action, e.g. "Roslyn
  could not load any project in this worktree (SDK 8.0.100 found,
  `global.json` requires 9.0.x). Files stay fully editable. Fix: update the
  SDK or `global.json`."
- **Capabilities:** syntax editing is ALWAYS preserved. Never a silent
  language-server switch (backlog decision).
- **Recovery action:** the stated one; `docs/dotnet-workflow.md` covers the
  common remediations.

### 5. Conflicting extension

Trigger: another extension also registers `.razor`/`.cshtml` (the community
Razor extensions) or the upstream "C#" extension alongside C# Plus.

- **Message:** "Another extension also claims Razor files
  (`<extension name>`). File associations are ambiguous until it is
  uninstalled. See the migration guide — C# Plus never modifies your
  installed extensions or settings."
- **Capabilities:** whatever association Zed resolves; editing support always
  intact.
- **Recovery action:** `docs/migration.md` (uninstall the conflicting
  extension; steps per source extension).

## Constraint recap (why the contract looks like this)

- A Zed extension's entire LSP surface is `language_server_command`, the
  initialization/workspace-configuration hooks, and the label hooks — no
  traffic observation, no virtual documents (zed#21133, not planned).
- Attaching a server to a language is a manifest fact; opting out is a
  user-side setting, not an extension-side toggle.
- Zed only opens real files; any future proxy owns the virtual-document
  lifecycle outright (M4.2B).
