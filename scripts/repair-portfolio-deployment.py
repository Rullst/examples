"""One explicitly approved recovery: copy only Showcase's admin password to Portfolio."""

import argparse
import copy
import importlib.util
from pathlib import Path
import subprocess
import sys
import time

spec = importlib.util.spec_from_file_location("verify", Path(__file__).with_name("verify-deployment.py"))
verify = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verify)

SOURCE = "rullst-showcase"
TARGET = "rullst-portfolio"
SECRET = "nexus-admin-showcase"
# Tests and image publication succeeded in GitHub run 35104293273. Only the
# subsequent preflight failed because Portfolio had no valid admin password.
IMAGE = "ghcr.io/rullst/portfolio:f1159fc4f04bb7b65dd429441b2affb98cc57d4e"


def azure_write(*args):
    for attempt in range(3):
        result = subprocess.run(
            ["az", "containerapp", *args, "--name", TARGET,
             "--resource-group", "rullst-rg", "--only-show-errors", "--output", "none"],
            capture_output=True, text=True, timeout=180,
        )
        if result.returncode == 0:
            return
        # Retrying the same secret/image update is idempotent. Do not echo CLI
        # output or exceptions: these may contain credentials or configuration.
        throttled = any(marker in result.stderr.lower() for marker in (
            "too many requests", "toomanyrequests", "throttl", "429",
        ))
        verify.require(throttled and attempt < 2, "Portfolio Azure update failed; no secrets logged.")
        print("Azure throttled the Portfolio update; retrying shortly.", flush=True)
        time.sleep(20 * (attempt + 1))


def repair(approved):
    verify.require(approved, "Explicit password-reuse approval is required.")
    source = verify.azure(SOURCE, "show")["properties"]
    _, password = verify.configuration(SOURCE, source)
    target = verify.azure(TARGET, "show")["properties"]
    verify.require(len(target["template"]["containers"]) == 1, "Expected one Portfolio container.")
    container = target["template"]["containers"][0]
    existing = {entry["name"]: entry for entry in container.get("env", [])}
    print("Portfolio admin setting: " + ("present" if "NEXUS_ADMIN_PASSWORD" in existing else "absent"))

    # Check the eventual configuration (including its OWN AI provider) before
    # any write. Never copy Groq keys or usernames from another application.
    candidate = copy.deepcopy(target)
    candidate["template"]["containers"][0]["env"] = [
        entry for entry in container.get("env", []) if entry["name"] != "NEXUS_ADMIN_PASSWORD"
    ] + [{"name": "NEXUS_ADMIN_PASSWORD", "value": password}]
    verify.configuration(TARGET, candidate)

    # Azure uses JSON null, not an empty array, before the first secret exists.
    declared = (target.get("configuration") or {}).get("secrets") or []
    if any(secret["name"] == SECRET for secret in declared):
        current = verify.azure(TARGET, "secret", "list", "--show-values", "--query",
                               f"[?name=='{SECRET}'].value | [0]")
        verify.require(current == password, "Dedicated Portfolio secret exists with a different value; no overwrite performed.")

    azure_write("secret", "set", "--secrets", f"{SECRET}={password}")
    # One revision switches the application image and password reference
    # together, retaining all other environment settings and the username.
    azure_write("update", "--image", IMAGE, "--set-env-vars",
                f"NEXUS_ADMIN_PASSWORD=secretref:{SECRET}", "--container-name", container["name"])
    configured = verify.azure(TARGET, "show")["properties"]
    _, actual = verify.configuration(TARGET, configured)
    verify.require(actual == password, "Portfolio password verification failed.")
    print("Portfolio now references the approved Showcase password as an Azure secret; source apps unchanged.", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reuse-showcase-password", action="store_true")
    args = parser.parse_args()
    try:
        repair(args.reuse_showcase_password)
    except RuntimeError as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
    except Exception as error:
        print(f"::error::Portfolio recovery failed ({type(error).__name__}); no response bodies or credentials logged.", file=sys.stderr)
        sys.exit(1)
