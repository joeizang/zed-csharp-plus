# Debugging .NET (M2.3)

**Status (checked 2026-09-06): no usable .NET debug adapter for Zed exists
today.** Zed's debugger speaks the Debug Adapter Protocol (DAP) and can drive
any adapter, but C#/.NET is not among its built-in adapters, and no .NET
adapter is published in Zed's extension registry. **Debugging may not work in
a C# Plus project.** The everyday .NET loop — restore, build, run, watch,
test — does not need a debugger and is covered by
[docs/dotnet-workflow.md](dotnet-workflow.md).

This page is the dated decision record for backlog item M2.3 (conditional
branch: "none exists"). It states the current state, how to troubleshoot a
missing or incompatible adapter, and what would change the picture. It is not
a promise that any of the third-party items below work.

## What Zed's debugger supports today (checked 2026-09-06)

- Zed implements the DAP client side. Adapters come **built-in** (C, C++, Go,
  JavaScript, PHP, Python, Rust, TypeScript) or from **debug-adapter
  extensions** (Java, Ruby, Swift). The official supported-language list has
  no C#/.NET entry — [zed.dev/docs/debugger](https://zed.dev/docs/debugger).
- The registry's debug-adapter filter listed 15 extensions and none targets
  .NET — [zed.dev/extensions?filter=debug-adapters](https://zed.dev/extensions?filter=debug-adapters).
- A full scan of the `zed-industries/extensions` repository (1,464 extension
  entries, GitHub API, 2026-09-06) finds one debug-adjacent entry,
  `unity-debugger` (Unity/Mono, attach-to-Editor only, requires sourcing your
  own adapter binary) — nothing for ASP.NET Core or general .NET.
- Third-party adapters register via `[debug_adapters.*]` in `extension.toml`
  plus `get_dap_binary` / `dap_request_kind` hooks
  ([zed.dev/docs/extensions/debugger-extensions](https://zed.dev/docs/extensions/debugger-extensions)).
  No published extension does this for .NET.

Consequence: running `debugger: start` (f4 f4) in a C# project offers no
.NET target.

## The candidates, and why none is usable today

**vsdbg (Microsoft) — licence forbids it.** The .NET Core Debugger Components
that VS Code's C# extension ships are licensed only "for use with Visual
Studio Code, Visual Studio or Xamarin Studio software to help you develop and
test your applications" ([dotnet/core#505](https://github.com/dotnet/core/issues/505),
open since 2017; the issue itself records JetBrains being forced to drop
debugging support in a Rider EAP over this licence). A community proxy
exposing vsdbg to
Zed exists (`nmannaii/vsdbg-zed`, created 2026-08-18, no licence file), but
using vsdbg outside Microsoft's products is each user's own legal question.
C# Plus does not bundle, download, or instruct you to wire vsdbg. (The
`dotnet/vsdbg` repository itself returned 404 when checked on 2026-09-06; the
licence terms above remain the operative restriction.)

**netcoredbg (Samsung) — a viable engine, but no published Zed extension.**
MIT-licensed, actively maintained (release 3.2.0-1092, 2026-06-25), speaks DAP
(`--interpreter=vscode`, or `--server[=port]` for TCP), with release binaries
for linux-amd64, linux-arm64, osx-arm64, and win64 — note **no macOS Intel
binary** in that release. [github.com/Samsung/netcoredbg](https://github.com/Samsung/netcoredbg)

**Community Zed extensions exist but none is published in the registry.** The
most maintained is
[`qwadrox/zed-netcoredbg`](https://github.com/qwadrox/zed-netcoredbg)
(MIT, 96 stars, last push 2026-02-04), installable only as a Zed **dev
extension** — clone the repository, have the Rust toolchain installed, and use
"Install Dev Extension" in Zed's Extensions pane. Everything else found by
GitHub search ("zed netcoredbg", 2026-09-06) had single-digit stars, was
empty, lacked a licence, or was a vsdbg proxy. None of these has been
verified end-to-end by this project.

**Conclusion: branch (b) applies.** The pieces for branch (a) exist — a
permissively licensed adapter engine plus at least one community extension
wiring it into Zed — but there is no installable-from-registry, maintained
.NET debug extension, and M2.3's "verify the documented path launches and
debugs a fixture app" criterion cannot be met without first installing
unverified third-party code as a dev extension. Milestone 2 does not block on
this.

## If you want to try anyway (unverified, DIY)

Not endorsed, not tested by this project, and may break at any Zed or
extension update. The pattern, based on the community extensions' own
documentation, is:

1. Install a netcoredbg-based extension as a dev extension (clone, Rust
   toolchain, "Install Dev Extension"). `qwadrox/zed-netcoredbg` is currently
   the most maintained candidate; check for others at
   [zed.dev/extensions?filter=debug-adapters](https://zed.dev/extensions?filter=debug-adapters)
   first — a registry-published one supersedes all of this.
2. netcoredbg itself is auto-downloaded by that extension, or pointed at via
   a `dap` setting (`"dap": { "netcoredbg": { "binary": "/path/to/netcoredbg" } }`
   in `settings.json`).
3. Define a launch configuration in `.zed/debug.json` with
   `"adapter": "netcoredbg"`, a `build` step running `dotnet build`, and
   `"program"` pointing at the built DLL (e.g.
   `bin/Debug/net8.0/MyApp.dll`). See the community README for a concrete
   example; do not copy secrets into it (see below).

## Troubleshooting a missing or incompatible adapter

- **No .NET target offered:** run `debugger: start` (f4 f4). If nothing
  .NET-related appears, no .NET debug adapter is installed — that is the
  expected state today (see the status line above).
- **Installed a dev extension, still nothing:** dev extensions must compile
  (Rust toolchain required) and appear under "Dev Extensions" in the Extensions
  pane. If the adapter name in `.zed/debug.json` does not exactly match the
  name the extension registers, Zed cannot start it.
- **VS Code `launch.json` configs do not work as-is:** Zed also loads
  `.vscode/launch.json`, but entries with `"type": "coreclr"` or `"dotnet"`
  depend on the VS Code C# extension's vsdbg adapter, which Zed does not have
  and licence-wise cannot ship (above). Those entries will not start.
- **Zed-side diagnostics:** `dev: copy debug adapter arguments` (captures how
  Zed tried to initialize the session), `dev: open debug adapter logs`
  (DAP trace of the most recent sessions), and the
  `debugger.log_dap_communications` / `debugger.timeout` settings
  ([zed.dev/docs/debugger](https://zed.dev/docs/debugger)).
- **netcoredbg-specific:** the binary must match your OS/architecture (no
  official macOS Intel binary in release 3.2.0-1092), must be marked
  executable, and a `dap.<name>.binary` override only takes effect once an
  extension has registered that adapter name — Zed's adapter list comes from
  built-ins and extensions only.

## launchSettings.json — conventions that hold today

`Properties/launchSettings.json` is a `dotnet run` / IDE launch feature, not
something Zed reads. Zed's debugger reads `.zed/debug.json` (and can surface
`.vscode/launch.json`). What this means concretely:

- **Run via tasks:** `dotnet run` applies launch settings — the first profile
  with `commandName: "Project"`, or the one named by
  `--launch-profile`; `--no-launch-profile` skips the file entirely. Profile
  values (`applicationUrl`, `environmentVariables`) override the process
  environment. ([Microsoft Learn: ASP.NET Core environments](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/environments), checked 2026-09-06.)
- **Debug via an adapter (if one exists):** a `dotnet build` + direct-DLL
  launch does **not** consult `launchSettings.json`. You must set
  `ASPNETCORE_URLS` and `ASPNETCORE_ENVIRONMENT` yourself in the debug
  configuration's `env`, or the app runs with `Production` defaults and a
  surprise port.
- **Secrets:** launch profiles commonly contain connection strings and keys.
  `.zed/debug.json` is a plaintext file usually committed to version control.
  Never copy profile secrets into it; use a user-level debug file or shell
  environment instead. This extension's task templates never print
  launch-profile secrets (backlog M2.2), and no debug documentation here will
  either.

## What would change this picture

A netcoredbg-based Zed debug extension published in the extension registry
with an active maintainer would move this to branch (a): this page would then
document companion setup, task conventions, and a verified walkthrough against
the fixture solutions (`fixtures/README.md`, from backlog item M0.3b),
launching and debugging a fixture app. The integration owner re-checks
[zed.dev/extensions?filter=debug-adapters](https://zed.dev/extensions?filter=debug-adapters)
at that point; `qwadrox/zed-netcoredbg` is the most likely candidate to
publish. Until then, this is a documentation-only handoff.

## Why this extension ships no debugger

The backlog is explicit: "Do not add a debugger, application scaffolding,
deployment/cloud tooling, or legacy-framework semantic support in this
roadmap," and "Out of scope: A new debugger or debug adapter"
(`BACKLOG.md`, *Decisions already made* and *Out of scope*). Zed's product
direction for C# Plus is that existing Zed extensions continue to own
debugging. M2.3 is a documentation handoff, not code; nothing in this
extension launches, downloads, or configures a debug adapter. See also
[docs/known-limits.md](known-limits.md) for the editing-only scope of this
release and [docs/dotnet-workflow.md](dotnet-workflow.md) for the tasks that
do exist.
