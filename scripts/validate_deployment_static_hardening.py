#!/usr/bin/env python3
"""Validate non-secret static deployment hardening for example artifacts.

This local gate inspects committed example deployment files and can run the
local arb-agent config/status path. It does not build or push images, install
or mutate services, open network listeners, load secrets, call exchanges/RPCs,
or claim production readiness.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import shutil
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
EXAMPLE_CONTAINERFILE = ROOT / "deployment/container/Containerfile.example"
PRODUCTION_CONTAINERFILE = ROOT / "deployment/container/Containerfile.production"
SYSTEMD_UNIT = ROOT / "deployment/systemd/arb-agent.service.example"
CONFIG_EXAMPLE = ROOT / "config.example.toml"

SECRET_ASSIGNMENT = re.compile(
    r"(?i)(api[_-]?key|secret|private[_-]?key|seed[_-]?phrase|mnemonic|token)\s*[:=]\s*['\"]?[A-Za-z0-9_/+=.-]{12,}"
)
CONFIG_SMOKE_TIMEOUT_SECONDS = 120


def read(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8")


def require(condition: bool, failures: list[str], code: str) -> None:
    if not condition:
        failures.append(code)


def validate_container(text: str) -> dict[str, Any]:
    failures: list[str] = []
    runtime_from = next(
        (line.strip() for line in text.splitlines() if line.strip().lower().startswith("from ") and " as runtime" in line.lower()),
        "",
    )
    require("distroless" in runtime_from.lower(), failures, "container runtime is not distroless")
    require("nonroot" in runtime_from.lower(), failures, "container runtime is not nonroot")
    require("expose" not in text.lower(), failures, "container declares exposed ports")
    require("env " not in text.lower(), failures, "container embeds environment values")
    require(not SECRET_ASSIGNMENT.search(text), failures, "container contains secret-like assignment")
    require('ENTRYPOINT ["/usr/local/bin/arb-agent"]' in text, failures, "container entrypoint is not arb-agent")
    require('CMD ["--help"]' in text, failures, "container default command is not help/status-safe")
    return {
        "runtime_from": runtime_from,
        "distroless_runtime": "distroless" in runtime_from.lower(),
        "nonroot_runtime": "nonroot" in runtime_from.lower(),
        "ports_exposed": "expose" in text.lower(),
        "env_values_embedded": "env " in text.lower(),
        "secret_like_assignment": SECRET_ASSIGNMENT.search(text) is not None,
        "failures": failures,
    }


def validate_systemd(text: str) -> dict[str, Any]:
    failures: list[str] = []
    lower = text.lower()
    checks = {
        "non_root_user": "user=arbagent" in lower and "group=arbagent" in lower,
        "config_file_reference": "--config /etc/arb-agent/config.toml" in text,
        "no_new_privileges": "nonewprivileges=true" in lower,
        "private_tmp": "privatetmp=true" in lower,
        "protect_system_strict": "protectsystem=strict" in lower,
        "protect_home": "protecthome=true" in lower,
        "bounded_write_paths": "readwritepaths=/var/lib/arb-agent /var/log/arb-agent" in lower,
        "empty_capability_bounding_set": "capabilityboundingset=" in lower,
        "empty_ambient_capabilities": "ambientcapabilities=" in lower,
        "memory_deny_write_execute": "memorydenywriteexecute=true" in lower,
        "native_syscall_arch": "systemcallarchitectures=native" in lower,
    }
    for key, passed in checks.items():
        require(passed, failures, f"systemd check failed: {key}")
    require(not SECRET_ASSIGNMENT.search(text), failures, "systemd unit contains secret-like assignment")
    require("environment=" not in lower, failures, "systemd unit embeds environment values")
    return checks | {
        "secret_like_assignment": SECRET_ASSIGNMENT.search(text) is not None,
        "environment_values_embedded": "environment=" in lower,
        "failures": failures,
    }


def validate_config(text: str) -> dict[str, Any]:
    failures: list[str] = []
    lower = text.lower()
    observe_mode = 'mode = "observe"' in lower
    paper_mode = 'mode = "paper"' in lower
    require(observe_mode or paper_mode, failures, "example config is not observe or paper mode")
    require("live_execution_enabled = false" in lower, failures, "live execution is not disabled")
    require("allow_withdrawals = false" in lower, failures, "withdrawals are not disabled")
    require("kill_switch_enabled = true" in lower, failures, "kill switch is not enabled")
    require(not SECRET_ASSIGNMENT.search(text), failures, "example config contains secret-like assignment")
    return {
        "observe_or_paper_mode": observe_mode or paper_mode,
        "observe_mode": observe_mode,
        "paper_mode": paper_mode,
        "live_execution_disabled": "live_execution_enabled = false" in lower,
        "withdrawals_disabled": "allow_withdrawals = false" in lower,
        "kill_switch_enabled": "kill_switch_enabled = true" in lower,
        "secret_like_assignment": SECRET_ASSIGNMENT.search(text) is not None,
        "failures": failures,
    }


def run_config_smoke(agent_bin: pathlib.Path | None) -> dict[str, Any]:
    if agent_bin is not None:
        command = [str(agent_bin), "--config", str(CONFIG_EXAMPLE)]
    else:
        cargo = shutil.which("cargo")
        if cargo is None:
            raise RuntimeError("cargo unavailable and --agent-bin was not provided")
        command = [cargo, "run", "-p", "arb-agent", "--", "--config", str(CONFIG_EXAMPLE)]
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT,
            check=False,
            encoding="utf-8",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=CONFIG_SMOKE_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired:
        return {
            "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
            "returncode": 124,
            "stdout_line_count": 1,
            "timeout_seconds": CONFIG_SMOKE_TIMEOUT_SECONDS,
            "timeout_expired": True,
            "config_loaded": False,
            "observe_or_paper_mode": False,
            "live_execution_disabled": False,
            "secret_like_output": False,
            "passed": False,
        }
    output = completed.stdout
    lower_output = output.lower()
    secret_like_output = SECRET_ASSIGNMENT.search(output) is not None
    return {
        "command_kind": "agent-bin" if agent_bin is not None else "cargo-run",
        "returncode": completed.returncode,
        "stdout_line_count": len(output.splitlines()),
        "timeout_seconds": CONFIG_SMOKE_TIMEOUT_SECONDS,
        "timeout_expired": False,
        "config_loaded": "config: loaded and validated" in lower_output,
        "observe_or_paper_mode": "mode: observe" in lower_output or "mode: paper" in lower_output,
        "live_execution_disabled": "live-intent: false" in lower_output,
        "secret_like_output": secret_like_output,
        "passed": completed.returncode == 0
        and "config: loaded and validated" in lower_output
        and ("mode: observe" in lower_output or "mode: paper" in lower_output)
        and "live-intent: false" in lower_output
        and not secret_like_output,
    }


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    missing = [
        str(path.relative_to(ROOT))
        for path in [EXAMPLE_CONTAINERFILE, PRODUCTION_CONTAINERFILE, SYSTEMD_UNIT, CONFIG_EXAMPLE]
        if not path.exists()
    ]
    if missing:
        raise RuntimeError(f"missing required deployment artifacts: {', '.join(missing)}")

    example_container = validate_container(read(EXAMPLE_CONTAINERFILE))
    production_container = validate_container(read(PRODUCTION_CONTAINERFILE))
    systemd = validate_systemd(read(SYSTEMD_UNIT))
    config = validate_config(read(CONFIG_EXAMPLE))
    config_smoke = run_config_smoke(args.agent_bin) if args.run_config_smoke else None
    failures = (
        example_container["failures"]
        + production_container["failures"]
        + systemd["failures"]
        + config["failures"]
    )
    if config_smoke is not None and not config_smoke["passed"]:
        failures.append("config smoke failed")

    return {
        "schema": "arbyclaw.deployment_static_hardening.v1",
        "container": example_container,
        "example_container": example_container,
        "production_container": production_container,
        "systemd": systemd,
        "config": config,
        "config_smoke": config_smoke,
        "config_smoke_requested": args.run_config_smoke,
        "bounded_timeouts": {"config_smoke_seconds": CONFIG_SMOKE_TIMEOUT_SECONDS},
        "passed": not failures,
        "failures": failures,
        "service_actions_performed": False,
        "network_listeners_started": False,
        "external_calls_performed": False,
        "secrets_loaded": False,
        "live_execution_enabled": False,
        "production_readiness_claimed": False,
    }


def print_text(report: dict[str, Any]) -> None:
    print("deployment static hardening validation")
    print(f"passed: {str(report['passed']).lower()}")
    print(f"container-distroless-runtime: {str(report['container']['distroless_runtime']).lower()}")
    print(f"container-nonroot-runtime: {str(report['container']['nonroot_runtime']).lower()}")
    print(f"container-ports-exposed: {str(report['container']['ports_exposed']).lower()}")
    print(
        "production-container-distroless-runtime: "
        f"{str(report['production_container']['distroless_runtime']).lower()}"
    )
    print(
        "production-container-nonroot-runtime: "
        f"{str(report['production_container']['nonroot_runtime']).lower()}"
    )
    print(
        "production-container-ports-exposed: "
        f"{str(report['production_container']['ports_exposed']).lower()}"
    )
    print(f"systemd-non-root-user: {str(report['systemd']['non_root_user']).lower()}")
    print(f"systemd-protect-system-strict: {str(report['systemd']['protect_system_strict']).lower()}")
    print(f"systemd-bounded-write-paths: {str(report['systemd']['bounded_write_paths']).lower()}")
    print(f"config-observe-or-paper-mode: {str(report['config']['observe_or_paper_mode']).lower()}")
    print(f"config-live-execution-disabled: {str(report['config']['live_execution_disabled']).lower()}")
    print(f"config-smoke-timeout-seconds: {report['bounded_timeouts']['config_smoke_seconds']}")
    if report["config_smoke"] is not None:
        smoke = report["config_smoke"]
        print(f"config-smoke-passed: {str(smoke['passed']).lower()}")
        print(f"config-smoke-timeout-expired: {str(smoke['timeout_expired']).lower()}")
        print(f"config-smoke-config-loaded: {str(smoke['config_loaded']).lower()}")
        print(f"config-smoke-observe-or-paper-mode: {str(smoke['observe_or_paper_mode']).lower()}")
        print(f"config-smoke-live-execution-disabled: {str(smoke['live_execution_disabled']).lower()}")
    print("service-actions-performed: false")
    print("network-listeners-started: false")
    print("external-calls-performed: false")
    print("secrets-loaded: false")
    print("live-execution-enabled: false")
    print("production-readiness-claimed: false")
    for failure in report["failures"]:
        print(f"failure: {failure}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="emit JSON report")
    parser.add_argument("--run-config-smoke", action="store_true", help="run arb-agent config/status smoke")
    parser.add_argument("--agent-bin", type=pathlib.Path, help="optional prebuilt arb-agent binary")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print_text(report)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
