#!/usr/bin/env python3
"""Fetch and authenticate the pinned upstream amd64 reference in local work space."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
REVISION = "29dd3dc2b13b4123162e5fe132504bb4b15569f1"
SHA256 = "599f6f94ff8c5057241eff0d54c3c74f95c34935b6457b33fe545defc61e9488"
SIGNER = "C2428CD75720FACDCF76B6EA17DE5ECB75A1100E"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True, help="clean pinned runc source checkout")
    parser.add_argument("--directory", type=Path, default=ROOT / "work/reference-runc")
    args = parser.parse_args()
    source = args.source.resolve(strict=True)
    head = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    dirty = subprocess.check_output(["git", "-C", str(source), "status", "--porcelain", "--untracked-files=all"], text=True)
    if head != REVISION or dirty:
        parser.error("source must be the clean pinned runc revision")
    directory = args.directory.resolve()
    directory.mkdir(parents=True, exist_ok=True)
    for name in ("runc.amd64", "runc.amd64.asc"):
        if not (directory / name).exists():
            subprocess.run(["gh", "release", "download", "v1.5.2", "--repo", "opencontainers/runc",
                            "--pattern", name, "--dir", str(directory)], check=True)
    binary = directory / "runc.amd64"
    with binary.open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != SHA256:
            parser.error("reference artifact hash mismatch; refusing execution")
    # The trust anchor is the public keyring at the pinned source revision.
    # Use a disposable keyring, without changing the user's GPG trust/settings.
    with tempfile.TemporaryDirectory(prefix="rune-reference-keys-") as home:
        imported = subprocess.run(["gpg", "--batch", "--homedir", home, "--import",
                                   str(source / "runc.keyring")], capture_output=True)
        if imported.returncode:
            parser.error("could not import pinned reference public keys")
        verified = subprocess.run(["gpg", "--batch", "--homedir", home, "--status-fd", "1", "--verify",
                                   str(directory / "runc.amd64.asc"), str(binary)], capture_output=True, text=True)
        if verified.returncode or f"[GNUPG:] VALIDSIG {SIGNER} " not in verified.stdout:
            parser.error("reference artifact signature did not match the pinned signer")
    binary.chmod(0o755)
    version = subprocess.check_output([str(binary), "--version"], text=True)
    expected = ("runc version 1.5.2", "commit: v1.5.2-0-g29dd3dc", "spec: 1.3.0",
                "go: go1.26.8", "libseccomp: 2.6.1", "libpathrs: 0.2.6")
    if tuple(version.strip().splitlines()) != expected:
        parser.error("reference artifact reported unexpected build provenance")
    provenance = {"repository": "https://github.com/opencontainers/runc", "revision": REVISION,
                  "release": "v1.5.2", "artifact": "runc.amd64", "sha256": SHA256,
                  "signer_fingerprint": SIGNER, "keyring_sha256": hashlib.sha256((source / "runc.keyring").read_bytes()).hexdigest(),
                  "build": list(expected), "source_build_reproduced": False}
    (directory / "provenance.json").write_text(json.dumps(provenance, indent=2, sort_keys=True) + "\n")
    print("Pinned reference artifact hash, signature, revision and build provenance verified.")


if __name__ == "__main__":
    main()
