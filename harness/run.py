#!/usr/bin/env python3
"""Golden-file snapshot harness for the extension's tree-sitter queries.

Runs `tree-sitter parse` and `tree-sitter query` over the corpus
for each configured language, normalizes the CLI output, and either compares
against committed goldens (`--check`, the default) or regenerates them
(`--update`).

Golden output is tree-sitter CLI version-sensitive: the normalized `parse` and
`query` output differs between CLI releases, so the goldens are only valid for
the exact CLI version pinned in `.github/workflows/ci.yml` (0.27.0). Bump both
together, regenerating with `--update` in the same commit.

Usage:
    python3 harness/run.py               # check all languages
    python3 harness/run.py --update      # regenerate goldens
    python3 harness/run.py --language csharp
    python3 harness/run.py --prep        # only prepare grammar workspaces
"""

from __future__ import annotations

import argparse
import difflib
import glob
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
GOLDEN_ROOT = REPO_ROOT / "harness" / "golden"


@dataclass(frozen=True)
class PinnedGrammar:
    repo: str
    commit: str
    workspace: str
    subdir: str = ""
    lang_name: str = ""


@dataclass(frozen=True)
class Language:
    name: str
    grammar_dir: str
    corpus_glob: str
    query_glob: str
    suffixes: tuple[str, ...]
    pinned: PinnedGrammar | None = None


PINNED_GRAMMARS = {
    "c_sharp": PinnedGrammar(
        repo="https://github.com/tree-sitter/tree-sitter-c-sharp",
        commit="485f0bae0274ac9114797fc10db6f7034e4086e3",
        workspace="harness/workspaces/c_sharp",
        lang_name="c_sharp",
    ),
    "razor": PinnedGrammar(
        repo="https://github.com/joeizang/tree-sitter-razor",
        commit="4ede6ff6e5c85630e8232eb9f52aca952f327ce7",
        workspace="harness/workspaces/razor",
        lang_name="razor",
    ),
    "xml": PinnedGrammar(
        repo="https://github.com/tree-sitter-grammars/tree-sitter-xml",
        commit="863dbc381f44f6c136a399e684383b977bb2beaa",
        workspace="harness/workspaces/xml",
        subdir="xml",
        lang_name="xml",
    ),
}

LANGUAGES: dict[str, Language] = {
    "razor": Language(
        name="razor",
        grammar_dir=PINNED_GRAMMARS["razor"].workspace,
        corpus_glob="corpus/razor/**",
        query_glob="languages/razor/*.scm",
        suffixes=(".cshtml", ".razor"),
        pinned=PINNED_GRAMMARS["razor"],
    ),
    "csharp": Language(
        name="csharp",
        grammar_dir=PINNED_GRAMMARS["c_sharp"].workspace,
        corpus_glob="corpus/csharp/**",
        query_glob="languages/csharp/*.scm",
        suffixes=(".cs",),
        pinned=PINNED_GRAMMARS["c_sharp"],
    ),
    "msbuild": Language(
        name="msbuild",
        grammar_dir=PINNED_GRAMMARS["xml"].workspace + "/" + PINNED_GRAMMARS["xml"].subdir,
        corpus_glob="corpus/msbuild/**",
        query_glob="languages/msbuild/*.scm",
        suffixes=(".csproj", ".props", ".targets", ".slnx"),
        pinned=PINNED_GRAMMARS["xml"],
    ),
    # csproj/slnx share the msbuild corpus: their .scm files are symlinks
    # to languages/msbuild (Zed resolves each language's queries from its
    # own directory only), but each language still gets its own snapshot
    # coverage of the corpus files it owns (see docs/project-files.md).
    "csproj": Language(
        name="csproj",
        grammar_dir=PINNED_GRAMMARS["xml"].workspace + "/" + PINNED_GRAMMARS["xml"].subdir,
        corpus_glob="corpus/msbuild/**",
        query_glob="languages/csproj/*.scm",
        suffixes=(".csproj",),
        pinned=PINNED_GRAMMARS["xml"],
    ),
    "slnx": Language(
        name="slnx",
        grammar_dir=PINNED_GRAMMARS["xml"].workspace + "/" + PINNED_GRAMMARS["xml"].subdir,
        corpus_glob="corpus/msbuild/**",
        query_glob="languages/slnx/*.scm",
        suffixes=(".slnx",),
        pinned=PINNED_GRAMMARS["xml"],
    ),
}

ANSI_RE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
TIMING_RE = re.compile(r"^[^(\s].*:\s*[0-9]+(?:\.[0-9]+)?\s*ms\b.*$")
SUMMARY_RE = re.compile(r"^[0-9]+ (?:matches|files)[^\n]*$")


def note(msg: str) -> None:
    print(msg)


def fail(msg: str) -> None:
    print(msg, file=sys.stderr)


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ, NO_COLOR="1", TERM="dumb", CLICOLOR="0")
    return subprocess.run(
        cmd,
        cwd=str(cwd),
        env=env,
        capture_output=True,
        text=True,
        errors="replace",
        check=False,
    )


def normalize(text: str, corpus_abs: Path, grammar_abs: Path) -> str:
    text = ANSI_RE.sub("", text)
    corpus_rel = corpus_abs.relative_to(REPO_ROOT).as_posix()
    text = text.replace(str(corpus_abs), corpus_rel)
    text = text.replace(str(REPO_ROOT) + os.sep, "<repo>/").replace(str(REPO_ROOT), "<repo>")
    text = text.replace(str(grammar_abs), "<grammar>")
    lines = [SUMMARY_RE.sub("", TIMING_RE.sub("", line).rstrip()) for line in text.splitlines()]
    while lines and not lines[-1]:
        lines.pop()
    return "\n".join(lines) + "\n"


def ensure_tree_sitter() -> None:
    probe = run(["tree-sitter", "--version"], cwd=REPO_ROOT)
    if probe.returncode != 0:
        fail("error: tree-sitter CLI not found; install tree-sitter-cli@0.27")
        sys.exit(2)


def ensure_workspace(language: Language) -> Path:
    if language.pinned is None:
        return REPO_ROOT / language.grammar_dir
    pinned = language.pinned
    ws = REPO_ROOT / pinned.workspace
    if not (ws / ".git").exists():
        note(f"[prep] {language.name}: cloning {pinned.repo} into {pinned.workspace}")
        ws.parent.mkdir(parents=True, exist_ok=True)
        if ws.exists():
            subprocess.run(["rm", "-rf", str(ws)], check=True)
        subprocess.run(["git", "init", "--quiet", str(ws)], check=True)
        subprocess.run(["git", "-C", str(ws), "remote", "add", "origin", pinned.repo], check=True)
        fetched = subprocess.run(
            ["git", "-C", str(ws), "fetch", "--quiet", "--depth", "1", "origin", pinned.commit],
            capture_output=True,
            text=True,
        )
        if fetched.returncode != 0:
            fail(f"error: failed to fetch {pinned.repo} @ {pinned.commit}: {fetched.stderr.strip()}")
            sys.exit(2)
        subprocess.run(["git", "-C", str(ws), "checkout", "--quiet", "FETCH_HEAD"], check=True)
    head = subprocess.run(
        ["git", "-C", str(ws), "rev-parse", "HEAD"], capture_output=True, text=True
    )
    if head.returncode != 0 or head.stdout.strip() != pinned.commit:
        fail(f"error: {pinned.workspace} is not at pinned commit {pinned.commit}")
        sys.exit(2)
    grammar_dir = ws / pinned.subdir if pinned.subdir else ws
    src = grammar_dir / "src" / "parser.c"
    if not src.exists():
        note(f"[prep] {language.name}: running tree-sitter generate in {pinned.workspace}")
        generated = run(["tree-sitter", "generate"], cwd=grammar_dir)
        if generated.returncode != 0 or not src.exists():
            fail(f"error: tree-sitter generate failed for {pinned.workspace}: {generated.stderr.strip()}")
            sys.exit(2)
    else:
        note(
            f"[prep] {language.name}: using committed parser in {pinned.workspace} "
            "(generate skipped; regenerating can produce different tables)"
        )
    config_json = grammar_dir / "tree-sitter.json"
    config_content = (
        json.dumps(
            {
                "grammars": [
                    {
                        "name": pinned.lang_name,
                        "path": ".",
                        "scope": f"source.{pinned.lang_name}",
                        "file-types": [s.lstrip(".") for s in language.suffixes],
                    }
                ],
                "metadata": {
                    "version": "0.0.0",
                    "license": "MIT",
                    "description": (
                        "Generated by harness/run.py for snapshot testing; "
                        "not part of the upstream grammar repository."
                    ),
                    "links": {"repository": pinned.repo},
                },
            },
            indent=2,
        )
        + "\n"
    )
    if not config_json.exists() or config_json.read_text(encoding="utf-8") != config_content:
        note(f"[prep] {language.name}: writing tree-sitter.json into {pinned.workspace}")
        config_json.write_text(config_content, encoding="utf-8")
    return grammar_dir


def collect_corpus_files(language: Language) -> list[Path]:
    pattern = str(REPO_ROOT / language.corpus_glob)
    files = []
    for hit in glob.glob(pattern, recursive=True):
        path = Path(hit)
        if path.is_file() and path.suffix in language.suffixes:
            files.append(path)
    return sorted(files)


def collect_query_files(language: Language) -> list[Path]:
    pattern = str(REPO_ROOT / language.query_glob)
    files = [Path(hit) for hit in glob.glob(pattern, recursive=True) if Path(hit).is_file()]
    return sorted(files)


def snapshot_parse(language: Language, grammar_dir: Path, corpus_file: Path) -> subprocess.CompletedProcess[str]:
    return run(["tree-sitter", "parse", str(corpus_file)], cwd=grammar_dir)


def snapshot_query(
    language: Language, grammar_dir: Path, query_file: Path, corpus_file: Path
) -> subprocess.CompletedProcess[str]:
    return run(["tree-sitter", "query", str(query_file), str(corpus_file)], cwd=grammar_dir)


def golden_path_for(language: str, corpus_file: Path, suffix: str) -> Path:
    rel = corpus_file.relative_to(REPO_ROOT).as_posix()
    return GOLDEN_ROOT / language / (rel + suffix)


def process_language(language: Language, update: bool) -> bool:
    ok = True
    grammar_dir = ensure_workspace(language)
    corpus_files = collect_corpus_files(language)
    if not corpus_files:
        fail(f"error: {language.name}: no corpus files matched {language.corpus_glob}")
        return False
    query_files = collect_query_files(language)
    if not query_files:
        note(
            f"[skip] {language.name}: no query files matched {language.query_glob} "
            "(query files may not exist yet; parse snapshots only)"
        )

    generated: set[Path] = set()
    for corpus_file in corpus_files:
        proc = snapshot_parse(language, grammar_dir, corpus_file)
        if not proc.stdout.strip():
            fail(f"error: {language.name}: tree-sitter parse failed for {corpus_file.relative_to(REPO_ROOT)}")
            fail(proc.stderr.strip())
            ok = False
            continue
        if proc.returncode != 0:
            note(
                f"[note] {language.name}: parse reported errors for "
                f"{corpus_file.relative_to(REPO_ROOT)} (expected for pathological inputs)"
            )
        content = normalize(proc.stdout, corpus_file, grammar_dir)
        target = golden_path_for(language.name, corpus_file, ".parse.txt")
        generated.add(target)
        ok = write_or_compare(target, content, update) and ok

        for query_file in query_files:
            qname = query_file.stem
            proc = snapshot_query(language, grammar_dir, query_file, corpus_file)
            if proc.returncode != 0:
                fail(
                    f"error: {language.name}: query {query_file.name} failed on "
                    f"{corpus_file.relative_to(REPO_ROOT)}"
                )
                fail(proc.stderr.strip())
                ok = False
                continue
            content = normalize(proc.stdout, corpus_file, grammar_dir)
            target = golden_path_for(language.name, corpus_file, f"__{qname}.txt")
            generated.add(target)
            ok = write_or_compare(target, content, update) and ok

    if update:
        prune_stale_goldens(language, generated, query_files)
    return ok


def write_or_compare(target: Path, content: str, update: bool) -> bool:
    rel = target.relative_to(REPO_ROOT).as_posix()
    if update:
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists() and target.read_text(encoding="utf-8") == content:
            return True
        target.write_text(content, encoding="utf-8")
        note(f"[update] {rel}")
        return True
    if not target.exists():
        fail(f"MISMATCH  {rel}: golden file missing (run with --update to generate)")
        return False
    expected = target.read_text(encoding="utf-8")
    if expected == content:
        return True
    diff = difflib.unified_diff(
        expected.splitlines(keepends=True),
        content.splitlines(keepends=True),
        fromfile=f"a/{rel}",
        tofile=f"b/{rel}",
    )
    fail("".join(diff).rstrip("\n"))
    fail(f"MISMATCH  {rel}")
    return False


def prune_stale_goldens(language: Language, generated: set[Path], query_files: list[Path]) -> None:
    lang_golden_dir = GOLDEN_ROOT / language.name
    if not lang_golden_dir.exists():
        return
    keep = {str(p.relative_to(REPO_ROOT)) for p in generated}
    for path in sorted(lang_golden_dir.rglob("*.txt")):
        rel = str(path.relative_to(REPO_ROOT))
        if rel in keep:
            continue
        if not query_files and "__" in path.name:
            continue
        path.unlink()
        note(f"[prune] {rel}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Tree-sitter golden snapshot harness")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--update", action="store_true", help="regenerate golden files")
    mode.add_argument(
        "--check", action="store_true", help="compare against golden files (the default)"
    )
    parser.add_argument("--language", choices=sorted(LANGUAGES), help="process one language only")
    parser.add_argument("--prep", action="store_true", help="only prepare grammar workspaces")
    args = parser.parse_args()

    ensure_tree_sitter()
    if args.prep:
        for language in LANGUAGES.values():
            if args.language is None or language.name == args.language:
                ensure_workspace(language)
        return

    selected = [LANGUAGES[args.language]] if args.language else list(LANGUAGES.values())
    all_ok = True
    for language in selected:
        note(f"=== {language.name} ===")
        all_ok = process_language(language, update=args.update) and all_ok
    if not all_ok:
        fail("FAILED: snapshot mismatches found (run with --update to regenerate)")
        sys.exit(1)
    note("OK: all snapshots match" if not args.update else "OK: goldens written")


if __name__ == "__main__":
    main()
