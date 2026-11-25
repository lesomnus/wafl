#!/usr/bin/env python3
"""Select whole repetition blocks, preserving provenance and checking load quality."""
import argparse
import copy
import json
import math
from pathlib import Path


def select(plan, directory):
    reports, trials, identities = {}, [], None
    configurations = None
    seen = set()
    for repeat, block in enumerate(plan["blocks"]):
        filename, original_repeat = block["report"], block["repeat"]
        if (filename, original_repeat) in seen:
            raise ValueError("a repetition cannot be selected twice")
        seen.add((filename, original_repeat))
        if filename not in reports:
            reports[filename] = json.loads((directory / filename).read_text())
        report = reports[filename]
        identity = {key: report[key] for key in ["guest_sha256", "binary_sha256", "script_sha256",
                                                "platform", "cpu", "cpu_max", "memory_max"]}
        arguments = report["arguments"]
        identity["workload"] = {key: arguments[key] for key in ["gateway", "seconds", "rps", "work_rps",
                                                               "work", "workers", "connections"]}
        if identities is None:
            identities = identity
        if identity != identities:
            raise ValueError("selected blocks use different binaries, environment or measurement method")
        rows = [t for t in report["trials"] if t["repeat"] == original_repeat]
        cases = {(t["case"], t["guest_keepalive"], t["client_keepalive"], t["memory_mib_per_guest"])
                 for t in rows}
        expected = {(case, guest == "on", client == "on", memory)
                    for case in arguments["cases"] for guest in arguments["guest_keepalive"]
                    for client in arguments["client_keepalive"] for memory in arguments["memory"]}
        if not rows or len(cases) != len(rows) or cases != expected:
            raise ValueError("select complete repetition blocks with each configuration exactly once")
        if configurations is None:
            configurations = cases
        if cases != configurations:
            raise ValueError("selected repetitions have different configurations")
        for original in rows:
            for phase in original["phases"]:
                traffic = phase["traffic"]
                if not traffic:
                    continue
                if traffic["errors"] or traffic["ok"] != traffic["requests"]:
                    raise ValueError(f"request failures in {filename}, repeat {original_repeat}")
                schedule = traffic["schedule_p95_ms"]
                fraction = traffic["achieved_rps"] / traffic["offered_rps"]
                quality = plan["quality"]
                if (not math.isfinite(schedule) or not math.isfinite(fraction)
                        or schedule > quality["schedule_p95_ms"]
                        or fraction < quality["rate_fraction"]):
                    raise ValueError(f"load quality failed: {filename}, repeat {original_repeat}, "
                                     f"{original['case']} {phase['phase']}: schedule={schedule:.3f} ms, "
                                     f"rate fraction={fraction:.6f}")
            trial = copy.deepcopy(original)
            trial["source"] = {"report": filename, "repeat": original_repeat}
            trial["repeat"] = repeat
            trials.append(trial)
    if not trials:
        raise ValueError("no repetition selected")
    return {**identities, "selection": plan,
            "source_reports": {name: {k: v for k, v in report.items() if k != "trials"}
                               for name, report in reports.items()}, "trials": trials}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan", type=Path)
    args = parser.parse_args()
    print(json.dumps(select(json.loads(args.plan.read_text()), args.plan.parent), indent=2))


if __name__ == "__main__":
    main()
