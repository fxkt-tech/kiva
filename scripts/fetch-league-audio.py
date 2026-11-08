#!/usr/bin/env python3
"""Reproduce pinned source soundbanks without installing a League client.

Optional importer dependency: pyzstd. This tool never runs during game startup
or builds. The reviewed range manifest avoids downloading entire client WADs.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import urllib.request

try:
    import pyzstd
except ImportError as error:
    raise SystemExit("Install the optional importer dependency: pip install pyzstd") from error


ROOT = Path(__file__).resolve().parents[1]
USER_AGENT = (
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fetch_segment(segment: dict, cache: Path) -> bytes:
    expected = segment["decoded_sha256"]
    cached = cache / f"{expected}.bin"
    if cached.exists():
        data = cached.read_bytes()
        if len(data) == segment["decoded_size"] and sha256(data) == expected:
            return data
    start = segment["start"]
    length = segment["length"]
    request = urllib.request.Request(segment["url"], headers={
        "User-Agent": USER_AGENT,
        "Range": f"bytes={start}-{start + length - 1}",
    })
    with urllib.request.urlopen(request, timeout=30) as response:
        if response.status != 206:
            raise ValueError("CDN did not honor the required byte range")
        compressed = response.read(length + 1)
    if len(compressed) != length:
        raise ValueError("Unexpected compressed segment length")
    data = pyzstd.decompress(compressed)
    if len(data) != segment["decoded_size"] or sha256(data) != expected:
        raise ValueError("Source segment checksum mismatch")
    cached.write_bytes(data)
    return data


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=ROOT / "assets/audio/riot-sources.json")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    output = args.output.resolve()
    cache = output / "segments"
    cache.mkdir(parents=True, exist_ok=True)
    for source in manifest["files"]:
        target = output / source["file"]
        if target.parent != output:
            raise ValueError("A source bank filename must be a plain filename")
        if target.exists() and sha256(target.read_bytes()) == source["sha256"]:
            print(f"Verified {target.name}")
            continue
        combined = b"".join(fetch_segment(segment, cache) for segment in source["segments"])
        offset = source["offset"]
        data = combined[offset:offset + source["length"]]
        if len(data) != source["length"]:
            raise ValueError("Source entry extends beyond its segments")
        if source["encoding"] == "zstd":
            data = pyzstd.decompress(data)
        elif source["encoding"] != "raw":
            raise ValueError("Unsupported source entry encoding")
        if len(data) != source["size"] or sha256(data) != source["sha256"]:
            raise ValueError(f"Soundbank checksum mismatch: {source['file']}")
        target.write_bytes(data)
        print(f"Downloaded {target.name} ({len(data)} bytes)")


if __name__ == "__main__":
    main()
