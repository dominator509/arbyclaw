#!/usr/bin/env python3
import os
import re
import json
import sys

# Elite secrets scanning regexes
SECRETS_REGEXES = {
    "AWS Access Key ID": r"(?i)AKIA[0-9A-Z]{16}",
    "AWS Secret Access Key": r"(?i)[A-Za-z0-9/+=]{40}",
    "RSA Private Key": r"-----BEGIN RSA PRIVATE KEY-----",
    "EC Private Key": r"-----BEGIN EC PRIVATE KEY-----",
    "PGP Private Key": r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
    "Generic API Key": r"(?i)(api[_-]?key|secret|token|password)[\s]*[:=][\s]*['\"][A-Za-z0-9\-_]{16,}['\"]",
    "Ethereum Private Key": r"0x[0-9a-fA-F]{64}",
    "Mnemonic Phrase": r"(\b[a-z]{3,}\b ){11}\b[a-z]{3,}\b"  # 12-word mnemonic approximation
}

IGNORE_DIRS = {".git", "target", "security-audit", "node_modules", ".github"}
IGNORE_FILES = {"Cargo.lock", "package-lock.json", "secrets_scanner.py"}

def scan_repo(root_dir="."):
    findings = []

    for dirpath, dirnames, filenames in os.walk(root_dir):
        # Filter ignored directories
        dirnames[:] = [d for d in dirnames if d not in IGNORE_DIRS]

        for file in filenames:
            if file in IGNORE_FILES:
                continue

            filepath = os.path.join(dirpath, file)
            try:
                with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()

                    for name, regex in SECRETS_REGEXES.items():
                        matches = re.finditer(regex, content)
                        for match in matches:
                            # Verify context to reduce false positives
                            snippet = content[max(0, match.start() - 20):min(len(content), match.end() + 20)]
                            # Basic entropy/false positive check for random strings would go here in a real scanner

                            findings.append({
                                "file": filepath,
                                "type": name,
                                "snippet": snippet.strip().replace('\n', '\\n')
                            })
            except Exception as e:
                pass

    return findings

if __name__ == "__main__":
    print("Starting elite secrets scan...")
    findings = scan_repo()

    report_path = "security-audit/phase-1-recon-threat-modeling/secrets_scan_report.json"
    with open(report_path, "w") as f:
        json.dump({"total_findings": len(findings), "findings": findings}, f, indent=2)

    print(f"Secrets scan complete. {len(findings)} potential secrets found.")
    print(f"Report saved to {report_path}")
