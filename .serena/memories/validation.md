# ArbyClaw Validation

## Required Full Local Sequence

Run after substantive repo changes. In this repo, prefix shell commands with `rtk`; for PowerShell built-ins, use `rtk powershell -NoProfile -Command "..."`

```text
rtk python3 scripts/generate_structure_manifest.py
rtk python3 scripts/validate_structure.py
rtk python3 -m py_compile scripts/validate_structure.py scripts/generate_structure_manifest.py scripts/validate_deployment_host_runtime.py
rtk cargo fmt --check
rtk cargo check --workspace
rtk cargo test --workspace
rtk cargo clippy --workspace --all-targets -- -D warnings
```

## Patch Rule

- Patch only real compile/test/lint failures with the smallest safe change.
- Regenerate `STRUCTURE_MANIFEST.md` after structure/source/doc inventory changes.
- Update `PRODUCTION_GAP_TRACKER.md` only for true current gap status, not to manufacture new gates.

## Completion Language

- Report exact gates run.
- Do not claim production readiness unless every roadmap/tracker blocker is actually closed and externally/prod-validated as required.
