# Fixtures (M0.3b)

Buildable .NET fixture solutions for the C# Plus backlog. These are the
projects that gate M2 (everyday .NET workflow — task definitions run against
them) and M3 (Razor semantics — the probe needs real Razor projects). They are
plain directories on disk, are **not** referenced by any solution in this
repository, and contain only `dotnet new` template output plus the small
adjustments documented below.

Acceptance (BACKLOG.md M0.3b): fixtures open and restore deterministically on
.NET 8+, offline after a first restore, with no external-service dependency.

## Layout

| Fixture | Template command | TFM | What it exercises |
| --- | --- | --- | --- |
| `mvc-web/` | `dotnet new mvc -n MvcWeb -f net8.0` | net8.0 | MVC views, controllers, Tag Helpers, `.cshtml` layouts/partials |
| `razor-pages/` | `dotnet new razor -n RazorPages -f net8.0` | net8.0 | Razor Pages, `@page`/`@model`, PageModel code-behind, `dotnet new page` target of the reload case |
| `blazor-webapp/` | `dotnet new blazor -n BlazorWebApp -f net8.0` | net8.0 | Blazor Web App with Server interactivity (template default), components, layouts, scoped CSS |
| `blazor-wasm/` | `dotnet new blazorwasm -n BlazorWasm -f net8.0` | net8.0 | Standalone WebAssembly Blazor, WASM SDK, client-side routing, JS interop entry points |
| `razor-classlib/` | `dotnet new razorclasslib -n RazorClassLib` | net8.0 | Razor Class Library (`Microsoft.NET.Sdk.Razor`), shareable components, `SupportedPlatform` |
| `multi-project/` | `dotnet new sln -n MultiProject --format slnx` | net8.0 | `.slnx` solution referencing `mvc-web`, `razor-pages`, and `razor-classlib` by relative path |
| `reload-case/` | (no project) | — | Project-reload scenario: `add-page.sh` + README |

### Template decisions

- **Blazor WASM standalone:** the `blazorwasm` template exists in the SDK this
  was built with (dotnet 10.0.302, template "Blazor WebAssembly Standalone
  App") and accepts `-f net8.0`, so it was used directly rather than
  synthesizing a standalone app from `dotnet new blazor` interactivity
  options. On older SDKs without `blazorwasm`, the `blazor` template with
  `-int WebAssembly` is the equivalent fallback.
- **Solution format:** `.slnx` via `dotnet new sln --format slnx` (supported
  by this SDK; `dotnet new slnx` is not a separate template). `.slnx` is the
  format the extension registers (`C# Solution File`), so the fixture matches
  the product surface. Note the solution references the three sibling fixture
  projects **by relative path** — do not move fixture directories apart.
- **Razor Class Library TFM:** the `razorclasslib` template offers no `-f`
  option and generated `net10.0` + `Microsoft.AspNetCore.Components.Web`
  10.0.10. Since the backlog guarantees .NET 8 LTS+, the generated `.csproj`
  was retargeted to `net8.0` with `Microsoft.AspNetCore.Components.Web`
  `8.0.30` (latest 8.0.x patch at creation). All other templates accepted
  `-f net8.0` directly; `blazor-wasm`'s template-pinned 8.0.x packages were
  kept as generated.

## Restore / build / verify

```sh
# First restore (one-time, needs network for template-pinned packages)
dotnet restore fixtures/mvc-web/MvcWeb.csproj
dotnet restore fixtures/razor-pages/RazorPages.csproj
dotnet restore fixtures/blazor-webapp/BlazorWebApp.csproj
dotnet restore fixtures/blazor-wasm/BlazorWasm.csproj
dotnet restore fixtures/razor-classlib/RazorClassLib.csproj
dotnet restore fixtures/multi-project/MultiProject.slnx

# Everything else is offline: build without restore
dotnet build fixtures/mvc-web/MvcWeb.csproj --no-restore
# ... or run the same checks over all fixtures at once:
scripts/verify-fixtures.sh
```

`scripts/verify-fixtures.sh` loops over all six targets, restores only if the
restore marker (`obj/project.assets.json`) is missing, then runs
`dotnet build --no-restore`, prints a compact pass/fail table, and exits
non-zero on any failure.

### Offline guarantee

After the first successful restore, all fixtures build with
`--no-restore` and never touch the network — verified by running
`scripts/verify-fixtures.sh` with `HTTP(S)_PROXY` pointed at a dead local
address so any network access would fail hard (2026-09-06, dotnet 10.0.302).
The only packages downloaded are the template-pinned ones listed in the
`.csproj` files above (from nuget.org); nothing else is fetched, and no
fixture connects to a database or external API. Sample data in the templates
is embedded/local (e.g. `blazor-wasm`'s HttpClient targets its own
`BaseAddress`).

## Lifecycle cases

- **Generated state:** after a successful build every fixture has `obj/` and
  `bin/` populated — that populated state *is* the generated-state case. It is
  intentionally **not committed** (`.gitignore` excludes `fixtures/**/bin/`
  and `fixtures/**/obj/`): a fresh clone starts with no generated state, and
  the release smoke's stale/missing-generated-state lifecycle test deletes
  and rebuilds these directories.
- **Project reload:** `fixtures/reload-case/` — run
  `fixtures/reload-case/add-page.sh` while a server/editor session is open on
  `razor-pages`; it adds `Pages/ReloadProbe.cshtml(.cs)` (refusing to
  overwrite, exit non-zero if it already exists) so project reload can be
  observed. See `fixtures/reload-case/README.md` for the full scenario and
  reset steps.

## Hygiene

- No secrets, no connection strings, no external-service dependencies —
  template defaults only (auth `None`, sample content embedded).
- `bin/` and `obj/` output is gitignored; `*.user` files are ignored too.
- Fixtures are not added to any repository solution and must stay that way.
