#!/usr/bin/env python3
"""Reproduce the local backend patch from the checksum-pinned source archive."""
import argparse
import difflib
import hashlib
from pathlib import Path, PurePosixPath
import tarfile

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE_SHA = '2c28770cf8fb3acca0d168a10f5d9f8333d304af0b2ff5654029822d222d19f0'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, help='libcontainer 0.7.0 crate; defaults to the local Cargo cache')
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--write', action='store_true')
    mode.add_argument('--check', action='store_true')
    args = parser.parse_args()
    candidates = sorted((Path.home() / '.cargo/registry/cache').glob('*/libcontainer-0.7.0.crate'))
    archive = args.archive or (candidates[0] if candidates else None)
    if archive is None:
        parser.error('supply --archive with the pinned upstream crate archive')
    with archive.open('rb') as stream:
        checksum = hashlib.file_digest(stream, 'sha256').hexdigest()
    if checksum != ARCHIVE_SHA:
        parser.error('upstream archive checksum differs from pinned provenance')
    original = {}
    with tarfile.open(archive) as package:
        for member in package.getmembers():
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or path.parts[0] != 'libcontainer-0.7.0':
                parser.error('unexpected upstream archive path')
            relative = Path(*path.parts[1:])
            if member.isfile() and (relative.suffix == '.rs' or str(relative) == 'Cargo.toml'):
                original[relative] = package.extractfile(member).read().decode()
    backend = ROOT / 'vendor/libcontainer'
    paths = set(original) | set(path.relative_to(backend) for path in backend.rglob('*.rs')) | {Path('Cargo.toml')}
    hunks = []
    for path in sorted(paths):
        local = backend / path
        if local.is_symlink():
            parser.error('backend source must not be a symlink')
        before = original.get(path, '')
        after = local.read_text() if local.exists() else ''
        hunks.extend(difflib.unified_diff(before.splitlines(keepends=True), after.splitlines(keepends=True),
            fromfile=f'a/crates/libcontainer/{path}' if path in original else '/dev/null',
            tofile=f'b/crates/libcontainer/{path}' if local.exists() else '/dev/null'))
    # Unified diff accepts an empty context line; omit its whitespace-only marker.
    patch = ''.join('\n' if line == ' \n' else line for line in hunks)
    destination = ROOT / 'docs/libcontainer.patch'
    if args.write:
        destination.write_text(patch)
    elif destination.read_text() != patch:
        raise SystemExit('backend patch is stale; run scripts/backend-patch.py --write')
    print(f'Backend patch matches {len(paths)} Rust/TOML source files against the pinned archive')


if __name__ == '__main__':
    main()
