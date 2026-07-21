#!/usr/bin/env python3
"""Claude Code Stop hook: read the last assistant reply aloud via siren.

Configured in .claude/settings.json. Reads the Stop payload from stdin,
pulls the final assistant text turn from the transcript, cleans it for
speech, and hands it to `siren speak` (backgrounded, non-blocking).

Env:
  SIREN_BIN        path to the siren binary (auto-detected otherwise)
  SIREN_SPEAK_ARGS extra args for `siren speak`, e.g. "--local" or "--voice Orus"
  SIREN_MAX_CHARS  spoken-text cap (default 600)
  SIREN_DRY_RUN    if set, print the spoken text to stderr instead of speaking
"""
import json
import os
import re
import subprocess
import sys

CANDIDATES = [
    os.environ.get("SIREN_BIN"),
    "/Users/ms/Development/siren/target/release/siren",
    "/Users/ms/Development/siren/target/debug/siren",
    "siren",
]


def siren_bin():
    for path in CANDIDATES:
        if path and (path == "siren" or os.path.exists(path)):
            return path
    return "siren"


def last_assistant_text(transcript_path):
    text = None
    try:
        with open(transcript_path, encoding="utf-8") as handle:
            for line in handle:
                line = line.strip()
                if not line:
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if entry.get("type") != "assistant":
                    continue
                parts = entry.get("message", {}).get("content", [])
                chunks = [
                    p.get("text", "")
                    for p in parts
                    if isinstance(p, dict) and p.get("type") == "text"
                ]
                joined = " ".join(c for c in chunks if c).strip()
                if joined:
                    text = joined
    except OSError:
        return None
    return text


def clean(text):
    text = re.sub(r"```.*?```", " ", text, flags=re.DOTALL)
    text = re.sub(r"`[^`]*`", " ", text)
    text = re.sub(r"!\[.*?\]\(.*?\)", " ", text)
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)
    text = re.sub(r"https?://\S+", " ", text)
    text = re.sub(r"[#>*_~|`]", " ", text)
    text = re.sub(r"\s+", " ", text).strip()
    cap = int(os.environ.get("SIREN_MAX_CHARS", "600"))
    if len(text) > cap:
        text = text[:cap].rsplit(" ", 1)[0] + " …"
    return text


def main():
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return
    transcript = payload.get("transcript_path")
    if not transcript:
        return
    text = last_assistant_text(transcript)
    if not text:
        return
    spoken = clean(text)
    if not spoken:
        return
    if os.environ.get("SIREN_DRY_RUN"):
        sys.stderr.write(spoken + "\n")
        return
    extra = os.environ.get("SIREN_SPEAK_ARGS", "").split()
    try:
        subprocess.Popen(
            [siren_bin(), "speak", *extra, spoken],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
        )
    except FileNotFoundError:
        pass


if __name__ == "__main__":
    main()
