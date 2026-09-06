# The everyday .NET workflow (M2.1 + M2.2)

Status: **M2.2 deliverable; M2.1 documentation half.** This document covers
language-server selection, the task templates shipped for `.csproj` and
`.slnx` files, how task scoping works in multi-project repositories, and the
remediation paths for the failure modes M2.1 names. The diagnostics messages
themselves live in `src/language_servers/roslyn.rs` and are owned by the M2.1
diagnostics work; the remediation matrix below is written to match their
message style (failure + one concrete next action; files stay editable) and
the roadmap owner reconciles exact strings at integration.

## Choosing a language server

**Roslyn is the recommended default server for C#**, and it is the required
backend for Razor semantics: when the release that attaches Roslyn to Razor
ships (M3.4), Razor completion, hover, and diagnostics come from Roslyn via
the `aspnetcorerazor` language id (see `docs/razor-contract.md` for the
compatibility contract). Selecting Roslyn now means a later Razor-semantics
upgrade changes nothing about your server configuration.

OmniSharp and `csharp-ls` remain supported **C#-only alternatives** with
different semantic expectations. Neither serves Razor; neither will. Their
diagnostic and completion behavior is theirs, not Roslyn's — do not expect
the two to agree with Roslyn (or each other) on diagnostics, code actions, or
project-load edge cases.

### Selecting per worktree

Server attachment is a manifest fact; choosing between servers is a user-side
setting. Put it in the worktree's `.zed/settings.json` so the choice follows
the repository:

```json
{
  "languages": {
    "CSharp": {
      "language_servers": ["!omnisharp", "!csharp-ls", "roslyn"]
    }
  }
}
```

Server ids are the keys from `extension.toml`: `omnisharp`, `roslyn`,
`csharp-ls`. A `!` prefix disables a server. To flip a single worktree to
OmniSharp: `["!roslyn", "!csharp-ls", "omnisharp"]`.

Server-specific settings (`lsp.roslyn.*`, `lsp.omnisharp.*`,
`lsp.csharp-ls.*`) are configured alongside, in the same file. Existing
settings carry over unchanged from the upstream C# extension — see
`docs/migration.md`.

## Task templates

The extension ships task templates in `languages/csproj/tasks.json` and
`languages/slnx/tasks.json`. They appear in Zed's task UI
(`task: spawn`) **only when you invoke them** — nothing in the task schema
can run on file open, and nothing in this extension auto-runs restore, build,
or any other command. Opening a project file starts no task; opening a Razor
file does not even start a language server (M0.4).

All templates share the same terminal behavior: they run in the current
terminal pane (`use_new_terminal: false`), disallow concurrent runs
(`allow_concurrent_runs: false` — a second click while one runs is refused,
so builds and publishes cannot double-fire), always reveal the terminal, and
hide it on success.

### Project tasks (`.csproj`)

Available when a `.csproj` editor is focused. `$ZED_FILE` resolves to the
focused file's path relative to the worktree root.

| Label | Command | Notes |
| --- | --- | --- |
| Restore Current Project | `dotnet restore $ZED_FILE` | |
| Build Current Project | `dotnet build $ZED_FILE` | |
| Test Current Project | `dotnet test $ZED_FILE` | Meaningful for test projects and solutions containing them; on a non-test project it runs zero tests. |
| Run Current Project | `dotnet run --project $ZED_FILE` | Only meaningful for runnable projects (`OutputType Exe`, e.g. web apps). On a class library it stops with "Ensure you have a runnable project type… OutputType 'Library'" — that is the correct outcome, not a task defect. Long-running; stop it with the terminal's interrupt. |
| Watch Current Project | `dotnet watch --project $ZED_FILE` | Build + run with hot reload. Runnable projects only. Long-running. |
| Clean Current Project | `dotnet clean $ZED_FILE` | |
| Publish Current Project | `dotnet publish $ZED_FILE` | Release output under `bin/Release/…/publish/`. |
| Format Current Project | `dotnet format $ZED_FILE` | Formats in place. Add `--verify-no-changes` in a personal task copy to make it check-only. |
| EF: Create Migration (requires dotnet-ef) | `dotnet ef migrations add MigrationName --project $ZED_FILE` | **Requires the `dotnet-ef` tool** (`dotnet tool install --global dotnet-ef`) and the target project referencing `Microsoft.EntityFrameworkCore.Design`. Zed has no per-run input prompt: the template ships the literal placeholder `MigrationName`. Edit it before running — duplicate the task into your user `tasks.json` with the real name, or run the command in the terminal. Running it unedited creates a migration literally named `MigrationName` (rename it, or `dotnet ef migrations remove` and redo). |

### Solution tasks (`.slnx`)

Available when a `.slnx` editor is focused. Every command scopes to the whole
solution.

| Label | Command | Notes |
| --- | --- | --- |
| Restore Current Solution | `dotnet restore $ZED_FILE` | |
| Build Current Solution | `dotnet build $ZED_FILE` | |
| Test Current Solution | `dotnet test $ZED_FILE` | Runs tests in every test project in the solution. |
| Clean Current Solution | `dotnet clean $ZED_FILE` | |
| Publish Current Solution | `dotnet publish $ZED_FILE` | Publishes every runnable project. |
| Format Current Solution | `dotnet format $ZED_FILE` | Solution-scoped formatting. |

## Multi-project selection

`$ZED_FILE` is the path of the **currently focused file**, relative to the
worktree root, and tasks run from the worktree root. Which template set is
offered follows which file is focused:

- Focus a **`.csproj`** → the project tasks appear and scope to that project,
  whatever else the worktree contains.
- Focus a **`.slnx`** → the solution tasks appear and scope to the whole
  solution.

For a **multi-project repository** the recommended workflow:

1. Keep the `.slnx` open (its tab need only be focused once) and use the
   solution tasks for restore/build/test/publish — they cover every project
   in one pass and cannot accidentally hit the wrong project.
2. When a command must target one project (run, watch, EF migrations), focus
   that project's `.csproj` first so `$ZED_FILE` points at it, then spawn the
   task.

The extension does not scan the worktree for project files or offer
alternatives — no automatic selection, no guessing. Focused file decides.

## Safety properties

- **User-invoked only.** Task templates are inert definitions; Zed runs one
  when you spawn it from the task UI. No task is triggered by opening,
  saving, or editing a file, and the extension code adds no other invocation
  path.
- **No secrets surfaced.** No task reads, prints, or transforms
  `launchSettings.json`, user-secrets, or connection strings, and no task
  accepts an environment or connection argument. One honest nuance: `dotnet
  run` and `dotnet watch` (the SDK itself) consult the project's
  `launchSettings.json` to pick the hosting environment and URLs and print
  the line `Using launch settings from …/launchSettings.json…` — that is
  SDK behavior, visible identically from any terminal. The file's contents,
  profile variables, and any connection strings inside it are not printed
  by these tasks.
- **No concurrency hazards.** Build, test, publish, run, and watch templates
  all set `allow_concurrent_runs: false`, so Zed refuses a second instance
  while one is running.

## Remediation matrix

M2.1 failure modes and how to get unstuck. What you see follows the
contract in `docs/razor-contract.md` ("Unsupported environment" state): the
error names the failure and one concrete next action, and **files stay fully
editable** — syntax highlighting, snippets, and structural editing never
depend on the server. Exact message strings are finalized by the M2.1
diagnostics work; the shape is fixed here.

| Failure | What you see | Remediation |
| --- | --- | --- |
| Missing .NET SDK | An extension error naming the missing SDK and one next action, e.g. "Roslyn could not start: no .NET SDK found. Install the .NET SDK (8.0+) from https://dotnet.microsoft.com/download and reopen." Files stay editable. | Install the .NET SDK 8 LTS or newer, verify `dotnet --version` in a terminal, reopen the worktree. |
| Invalid `global.json` | An error naming the conflict, e.g. "Roslyn could not load any project in this worktree (SDK 8.0.100 found, `global.json` requires 9.0.x). Files stay fully editable. Fix: update the SDK or `global.json`." | Align `global.json`'s `sdk.version`/`rollForward` with an installed SDK (`dotnet --list-sdks`), or install the required SDK. |
| Project-load failure | An error naming the load outcome and next step, e.g. "Roslyn loaded 0 of 3 projects (see Roslyn output for details). Files stay fully editable. Fix: run *Restore Current Solution* and check `dotnet build` output." | Run the Restore/Build task templates in the terminal to see the real MSBuild error (bad package reference, missing target framework, malformed project file); fix the project and reopen. |
| Incompatible Roslyn components | An error naming the mismatch, e.g. "Roslyn could not serve this worktree: server/components mismatch (version X). Files stay fully editable. Fix: reload the worktree so C# Plus re-resolves a compatible server." | Reload the worktree (or reinstall the extension) so the pinned server package is re-downloaded; clear the extension's cached server directory if the error persists. Razor semantics additionally require the compatible components per `docs/razor-contract.md`. |
| Disabled Roslyn (user setting) | No Roslyn session and Razor semantics unavailable; C# still served by whatever remains enabled. With Roslyn disabled by `language_servers: ["!roslyn"]`, Razor files keep editing support and explain that semantics require Roslyn. | Re-enable Roslyn: remove the `!roslyn` entry from `languages.CSharp.language_servers` in the worktree's `.zed/settings.json` and reload. |
| Offline NuGet | Server-side errors naming offline restore (e.g. "unable to load the service index for https://api.nuget.org/v3/index.json"), files editable. | Restore once online (`dotnet restore` against a worktree with network, or a configured mirror); the M0.3b fixtures restore offline only after a first online restore. Then reload the worktree. |

Two invariants hold in every row: syntax editing is always preserved, and
the extension never silently switches language servers.

## Validation record

Every template was validated against throwaway projects under `/tmp/taskval`
(MVC app `mvc`, class library `classlib`, xUnit project `tests`, three-project
solution `TaskVal.slnx`) with .NET SDK 10.0.302 on macOS arm64, using the
exact command shape shipped above, run from a directory playing the role of
the worktree root:

| Template | Evidence |
| --- | --- |
| Restore / Build (project, solution) | Executed; restore up-to-date, build 0 errors. |
| Test (project) | Executed against `tests`: 1/1 passed. |
| Test (solution) | Executed against `TaskVal.slnx`: 1/1 passed across the solution. |
| Run (project) | Booted on `mvc`: "Now listening on: http://localhost:5162", "Application started"; killed after ~20 s (long-running by design). |
| Watch (project) | Booted on `mvc`: "Hot reload enabled", "Loaded 1 project(s)", "Waiting for changes", app listening; killed after ~20 s. |
| Clean (project, solution) | Executed; exit 0. |
| Publish (project) | Executed on `classlib`: artifacts under `bin/Release/…/publish/`. |
| Publish (solution) | Executed on `TaskVal.slnx`: published both `mvc` and `classlib` to Release publish directories. |
| Format (project, solution) | Executed; exit 0. (Note: `dotnet format` does not accept `--nologo`; the template deliberately passes no extra flags.) |
| EF: Create Migration | Two-part proof. The command dispatches through the real `dotnet-ef` CLI: with the tool absent or the target lacking `Microsoft.EntityFrameworkCore.Design` it stops with exactly the documented prerequisite error. End-to-end on an EF-equipped scratch project (`dotnet-ef` 10.0.9 + Design/Sqlite packages + a `DbContext`), the exact template command created `Migrations/<timestamp>_MigrationName.cs` and the model snapshot. |

The final acceptance validation — every task run from Zed's task UI against
the M0.3b fixture solutions — is section 2 of `docs/release-smoke.md` and is
run per release once `fixtures/` lands (see `fixtures/README.md`). The
validation above substitutes throwaway projects for the fixtures until then,
recorded as `release-smoke.md` instructs.
