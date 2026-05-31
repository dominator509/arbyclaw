#!/usr/bin/env python3
"""Generate STRUCTURE_MANIFEST.md for the repository."""

from __future__ import annotations

import hashlib
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", "target", "__pycache__"}
MANIFEST = "STRUCTURE_MANIFEST.md"


def included_files() -> list[pathlib.Path]:
    files: list[pathlib.Path] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT)
        if relative.as_posix() == MANIFEST:
            continue
        if any(part in SKIP_DIRS for part in relative.parts):
            continue
        files.append(path)
    return sorted(files, key=lambda path: path.relative_to(ROOT).as_posix())


def main() -> int:
    rows = []
    for path in included_files():
        relative = path.relative_to(ROOT).as_posix()
        data = path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        rows.append(f"| `{relative}` | {len(data)} | `{digest}` |")

    lines = [
        "# STRUCTURE_MANIFEST.md",
        "",
        "Generated during Phase 26 audit/runtime crash, concurrency, filesystem, disk-full, stale-lock, lifecycle-concurrency, state-permission, graceful-shutdown, backup-restore, restart-recovery, recovery-disposition, CLI-status, container-validation, and incomplete-recovery fail-closed validation checkpoint after governance reconciliation and validation.",
        "",
        "Note: this manifest intentionally excludes `STRUCTURE_MANIFEST.md` itself to avoid self-referential hash drift.",
        "",
        "## File Manifest",
        "",
        "| Path | Bytes | SHA-256 |",
        "|---|---:|---|",
        *rows,
    ]
    (ROOT / MANIFEST).write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
