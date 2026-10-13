#!/usr/bin/env python3
"""Inventory pinned source declarations; never infer semantic conformance."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs/parity/inventory.json"
BASELINES = {
    "runc": {"repository": "https://github.com/opencontainers/runc", "tag": "v1.5.2",
             "revision": "29dd3dc2b13b4123162e5fe132504bb4b15569f1"},
    "runtime-spec": {"repository": "https://github.com/opencontainers/runtime-spec", "tag": "v1.3.0",
                     "revision": "92249139eea7161e13745abd4cb6d0ea02a3227a"},
}
PARTIAL_COMMANDS = {"create", "start", "state", "kill", "delete", "exec", "pause", "resume"}
MISSING_OPTIONS = {"create:pidfd-socket", "create:no-new-keyring",
                   "exec:pidfd-socket", "exec:ignore-paused"}
KINDS = {"command", "option", "schema-property", "go-json-field", "spec-section",
         "normative-line", "integration-case", "unit-case", "runc-doc-section"}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def inventory(paths):
    entries, sources = [], {}

    def read(project, path):
        relative = path.relative_to(paths[project]).as_posix()
        data = path.read_bytes()
        sources[f"{project}/{relative}"] = digest(data)
        return relative, data.decode()

    def add(project, path, kind, locator, issue, status="untested"):
        entries.append({"id": f"{project}:{path}:{kind}:{locator}", "kind": kind,
                        "source": f"{project}/{path}", "locator": str(locator),
                        "issue": issue, "status": status, "evidence": []})

    # Only registered commands count. The extractor is intentionally tied to the
    # pinned Go declaration layout; count guards below expose unexpected changes.
    main_path, main = read("runc", paths["runc"] / "main.go")
    registration = re.search(r"app.Commands = \[\]cli.Command\{(.*?)\n\t\}", main, re.S)
    if registration is None:
        raise ValueError("cannot find registered commands")
    variables = re.findall(r"\b(\w+Command)\b", registration[1])
    if len(variables) != 17 or len(set(variables)) != 17:
        raise ValueError("unexpected pinned command set")
    files = list(sorted(paths["runc"].glob("*.go")))
    for variable in variables:
        declarations = [(p, p.read_text()) for p in files
                        if re.search(rf"var {variable} = cli.Command\{{", p.read_text())]
        if len(declarations) != 1:
            raise ValueError(f"ambiguous command {variable}")
        path, content = declarations[0]
        relative, _ = read("runc", path)
        start = content.index(f"var {variable} = cli.Command{{")
        # resume shares pause.go; stop at the next top-level declaration.
        block = re.split(r"\n(?:var|func) ", content[start:], maxsplit=1)[0]
        name = re.search(r'Name:\s*"([^"]+)"', block)[1]
        status = "partial" if name in PARTIAL_COMMANDS else "missing"
        add("runc", relative, "command", name, 12, status)
        for flag in re.findall(r'cli\.\w+Flag\{\s*Name:\s*"([^"]+)"', block):
            locator = name + ":" + flag
            option_status = "missing" if status == "missing" or locator in MISSING_OPTIONS else "untested"
            if locator == "exec:tty, t":
                option_status = "partial"  # Direct TTY exec needs a process file.
            add("runc", relative, "option", locator, 12, option_status)
    global_block = main.split("app.Flags = []cli.Flag{", 1)[1].split("app.Commands", 1)[0]
    for flag in re.findall(r'cli\.\w+Flag\{\s*Name:\s*"([^"]+)"', global_block):
        add("runc", main_path, "option", "global:" + flag, 12,
            "missing" if flag == "rootless" else "untested")
    # Framework-provided flags are not explicit declarations in main.go.
    for flag in ("global:help, h", "global:version, v", "command:help, h"):
        add("runc", main_path, "option", flag, 12)

    def walk_schema(relative, node, pointer=""):
        if isinstance(node, dict):
            for key, value in sorted(node.items()):
                escaped = key.replace("~", "~0").replace("/", "~1")
                child = pointer + "/" + escaped
                if key == "properties":
                    for prop in sorted(value):
                        prop = prop.replace("~", "~0").replace("/", "~1")
                        add("runtime-spec", relative, "schema-property", child + "/" + prop, 14)
                walk_schema(relative, value, child)
        elif isinstance(node, list):
            for index, value in enumerate(node):
                walk_schema(relative, value, pointer + "/" + str(index))

    for path in sorted((paths["runtime-spec"] / "schema").glob("*.json")):
        relative, content = read("runtime-spec", path)
        walk_schema(relative, json.loads(content))
    for path in sorted((paths["runtime-spec"] / "specs-go").rglob("*.go")):
        relative, content = read("runtime-spec", path)
        for line, text in enumerate(content.splitlines(), 1):
            if re.search(r'`json:"[^"-]', text):
                add("runtime-spec", relative, "go-json-field", f"L{line}", 14)
    for project, pattern, kind in (("runtime-spec", "*.md", "spec-section"),
                                   ("runc", "docs/*.md", "runc-doc-section")):
        for path in sorted(paths[project].glob(pattern)):
            relative, content = read(project, path)
            for line, text in enumerate(content.splitlines(), 1):
                if re.match(r"^#{1,6} ", text):
                    add(project, relative, kind, f"L{line}", 11 if project == "runtime-spec" else 13)
                if project == "runtime-spec" and re.search(r"\b(MUST|SHALL|SHOULD|REQUIRED|MAY|RECOMMENDED)\b", text):
                    add(project, relative, "normative-line", f"L{line}", 11)
    for path in sorted((paths["runc"] / "tests/integration").glob("*.bats")):
        relative, content = read("runc", path)
        for line, text in enumerate(content.splitlines(), 1):
            if re.match(r"\s*@test\s+", text):
                add("runc", relative, "integration-case", f"L{line}", 13)
    for path in sorted(paths["runc"].rglob("*_test.go")):
        relative, content = read("runc", path)
        for match in re.finditer(r"(?m)^func ((?:Test|Fuzz)\w+)\(", content):
            add("runc", relative, "unit-case", match[1], 13)
    entries.sort(key=lambda entry: entry["id"])
    return {"format": 1, "baselines": BASELINES,
            "scope": "Source inventory floor; semantic cases and cross-products require refinement. Not parity proof.",
            "counts": dict(sorted(Counter(e["kind"] for e in entries).items())),
            "sources": dict(sorted(sources.items())), "entries": entries}


def check(data):
    if data["format"] != 1 or data["baselines"] != BASELINES:
        raise ValueError("baseline changed: requires explicit baseline review and regeneration")
    entries = data["entries"]
    if len({e["id"] for e in entries}) != len(entries):
        raise ValueError("duplicate inventory IDs")
    if data["counts"] != dict(Counter(e["kind"] for e in entries)):
        raise ValueError("inventory counts differ")
    if set(data["counts"]) != KINDS or data["counts"]["command"] != 17:
        raise ValueError("incomplete inventory kinds or commands")
    for value in data["sources"].values():
        if not re.fullmatch(r"[0-9a-f]{64}", value):
            raise ValueError("invalid source digest")
    for entry in entries:
        if entry["source"] not in data["sources"] or entry["issue"] not in {11, 12, 13, 14}:
            raise ValueError("entry lacks source or gap issue")
        if entry["status"] not in {"partial", "missing", "untested", "verified"}:
            raise ValueError("invalid support status")
        if entry["status"] == "verified" and not entry["evidence"]:
            raise ValueError("verified entry lacks evidence")
        for evidence in entry["evidence"]:
            if not re.fullmatch(r"[0-9a-f]{40}", evidence["rune_revision"]):
                raise ValueError("evidence needs exact rune revision")
            path = (ROOT / evidence["record"]).resolve()
            if not path.is_relative_to(ROOT) or not path.is_file():
                raise ValueError("evidence record missing or outside repository")
    print("parity inventory integrity passed: " + json.dumps(data["counts"], sort_keys=True))
    print("Source declarations are not semantic coverage; no parity claim.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runc", type=Path)
    parser.add_argument("--spec", type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if bool(args.runc) != bool(args.spec) or (args.write and not args.runc):
        parser.error("provide both --runc and --spec; --write requires both")
    if args.runc:
        paths = {"runc": args.runc.resolve(), "runtime-spec": args.spec.resolve()}
        for project, path in paths.items():
            revision = subprocess.check_output(["git", "-C", str(path), "rev-parse", "HEAD"], text=True).strip()
            dirty = subprocess.check_output(["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"], text=True)
            if revision != BASELINES[project]["revision"] or dirty:
                raise ValueError(f"{project} must be a clean checkout of the pinned revision")
        data = inventory(paths)
        check(data)
        if args.write:
            OUTPUT.parent.mkdir(parents=True, exist_ok=True)
            OUTPUT.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
        else:
            # Compare declarations/digests while preserving separately reviewed evidence.
            saved = json.loads(OUTPUT.read_text())
            check(saved)
            statuses = {e["id"]: e["status"] for e in data["entries"]}
            for entry in saved["entries"]:
                entry["evidence"] = []
                entry["status"] = statuses.get(entry["id"])
            if data != saved:
                raise ValueError("source inventory differs from committed inventory")
    check(json.loads(OUTPUT.read_text()))


if __name__ == "__main__":
    main()
