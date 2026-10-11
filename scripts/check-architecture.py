#!/usr/bin/env python3
from pathlib import Path
import tomllib
root = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((root / "Cargo.toml").read_text())
assert manifest["workspace"]["lints"]["rust"]["unsafe_code"] == "forbid"
core = tomllib.loads((root / "crates/rune-core/Cargo.toml").read_text())
assert not core.get("dependencies"), "core must have no external dependencies"
for data in [manifest, core]:
    assert data["lints"]["workspace"] is True
print("architecture checks passed (first-party crates; vendored backend is a separate boundary)")
