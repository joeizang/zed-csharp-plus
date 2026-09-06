# Project-reload case

A scripted scenario for observing project reload when a file is added to a
fixture while a server/editor session is running. It lives at the top level of
`fixtures/` (not inside the multi-project fixture) so it is discoverable next
to the other fixture lifecycle cases.

## Scenario

1. Open `fixtures/razor-pages/` (or the repo root) in Zed with C# Plus, and
   start `dotnet watch run` in `fixtures/razor-pages/` in a terminal.
2. Run `fixtures/reload-case/add-page.sh`.
3. The script adds `ReloadProbe.cshtml` + `ReloadProbe.cshtml.cs` to
   `fixtures/razor-pages/Pages/` using `dotnet new page`.

Expected observations:

- `dotnet watch` detects the new files, rebuilds, and restarts the app; the
  page is served at `/ReloadProbe`.
- The editor's project system picks up the new page without any `.csproj`
  edit — SDK-style projects glob `Pages/**` automatically, which is exactly
  why this case needs no project-file change.
- With Roslyn attached, completion/navigation inside the new page works after
  the reload (project-load events fire for the added files).

## Script properties

- Refuses to overwrite: exits non-zero (1) if either target file already
  exists, so re-running never clobbers anything.
- Deterministic: fixed page name `ReloadProbe`, fixed namespace
  `RazorPages.Pages`, targets only the `razor-pages` fixture.

## Reset

Delete the two generated files to return the fixture to its committed state:

```sh
rm fixtures/razor-pages/Pages/ReloadProbe.cshtml \
   fixtures/razor-pages/Pages/ReloadProbe.cshtml.cs
```

Requires the .NET SDK on PATH (same requirement as the fixtures themselves).
