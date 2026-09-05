# Release smoke (automated, per release)

The "Automated, per release" bucket of `docs/quality-gates.md`. Run before
every publishing PR. Two of the three buckets live here; the third (manual
checklist) is `docs/manual-checklist.md`.

## 1. Server download and startup smoke, per supported runtime identifier

For each rid the extension maps (`win-x64`, `win-arm64`, `linux-x64`,
`linux-arm64`, `osx-x64`, `osx-arm64`), on a machine (or container) of that
platform:

1. Install the extension from the release commit (dev-mode install is
   acceptable).
2. Open a `.cs` file in a fresh worktree with the Roslyn server selected.
3. Expect: the extension resolves the NuGet package
   `roslyn-language-server.<rid>`, downloads it on first run, launches
   `Microsoft.CodeAnalysis.LanguageServer` with `--stdio --autoLoadProjects`,
   and the server completes `initialize`.
4. Repeat once for OmniSharp (GitHub release asset path) and once for
   csharp-ls (requires `dotnet` on PATH; verify the missing-`dotnet` error
   message appears when it is absent).
5. Offline pass: with the package already cached in the extension work
   directory, disconnect the network and reopen a `.cs` file — startup must
   succeed from cache.

Record: date, OS/arch, server, version resolved, pass/fail.

## 2. Task definitions against the M0.3b fixtures

> Status: pending M0.3b (fixture solutions). Until the fixtures land, run
> this section against any throwaway project and record that substitution.

For each task template shipped in `languages/csproj/tasks.json` and
`languages/slnx/tasks.json`:

1. Open the corresponding fixture file (`fixtures/**`, M0.3b).
2. Run every task from Zed's task UI; expect the exact command in
   `tasks.json` to execute in the terminal tab and to succeed against the
   fixture (or fail with a clear, non-hanging error where intentionally
   invalid).
3. Verify no task runs automatically on open, and that no task surfaces
   connection strings or launch-profile secrets.

## 3. Editing-support invariants (cheap, every release)

- Open a `.razor` and a `.cshtml` file: the language resolves to **Razor**
  (not HTML, not CSharp).
- Confirm no language server starts for those buffers (the manifest attaches
  none in this release).
- Confirm ordinary `.cs` still opens as CSharp with the previously selected
  server.
