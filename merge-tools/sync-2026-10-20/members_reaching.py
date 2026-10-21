#!/usr/bin/env python3
r"""Which workspace members transitively depend on a given set of packages?

Reads a `cargo metadata` JSON and answers: for every workspace member, does
its dependency closure reach any of the named packages? Those members cannot
be built on this host, so they must be excluded from a `cargo check
--workspace` run.

Usage: python members_reaching.py <metadata.json> <pkg> [<pkg> ...]
"""
import json
import sys

meta = json.load(open(sys.argv[1], encoding="utf-8"))
targets = set(sys.argv[2:])
host = meta.get("target_directory", "")  # unused; host triple is fixed below
HOST = "x86_64-pc-windows-msvc"

# package id -> its dependency ids
deps = {p["id"]: list(p.get("dependencies", [])) for p in meta["packages"]}
by_name = {}
for p in meta["packages"]:
    by_name.setdefault(p["name"], []).append(p["id"])

target_ids = set()
for name in targets:
    target_ids.update(by_name.get(name, []))

# resolve "dependencies" (which are names) to ids via the resolve graph
resolve = meta.get("resolve", {})
node_deps = {}
for node in resolve.get("nodes", []):
    keep = []
    for d in node.get("deps", []):
        kinds = d.get("dep_kinds") or [{}]
        # a dep only applies on this host if at least one of its dep_kinds is
        # target-agnostic (target: null) or names the host platform
        applies = False
        for k in kinds:
            tgt = k.get("target")
            if tgt is None or HOST in tgt:
                applies = True
                break
        if applies:
            keep.append(d["pkg"])
    node_deps[node["id"]] = keep

members = [p for p in meta["packages"] if p.get("source") is None]


def reaches(target: str) -> bool:
    seen, stack = set(), [target]
    while stack:
        n = stack.pop()
        if n in seen:
            continue
        seen.add(n)
        if n in target_ids:
            return True
        stack.extend(node_deps.get(n, []))
    return False


print(f"target packages: {', '.join(sorted(targets))} "
      f"({len(target_ids)} id(s))")
bad = []
for m in members:
    if reaches(m["id"]):
        bad.append(m["name"])
print(f"\n{len(bad)} of {len(members)} workspace members cannot build on this host:")
for n in sorted(bad):
    print(f"   {n}")
print("\n--exclude " + " --exclude ".join(sorted(bad)))
