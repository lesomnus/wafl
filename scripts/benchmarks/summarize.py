#!/usr/bin/env python3
"""Aggregate recorded trials without selecting the fastest run."""
import argparse
import collections
import json
from pathlib import Path
import statistics


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("reports", nargs="+")
    args = parser.parse_args()
    grouped = collections.defaultdict(list)
    for filename in args.reports:
        report = json.loads(Path(filename).read_text())
        for trial in report["trials"]:
            for phase in trial["phases"]:
                key = (trial["gateway"], trial.get("guest_keepalive", False), trial.get("client_keepalive", False),
                       trial["memory_mib_per_guest"], trial["case"], phase["phase"])
                grouped[key].append(phase)
    summary = []
    for (gateway, guest_keepalive, client_keepalive, memory, case, phase), values in sorted(grouped.items()):
        item = {"gateway": gateway, "memory_mib_per_guest": memory,
                "guest_keepalive": guest_keepalive, "client_keepalive": client_keepalive,
                "case": case, "phase": phase, "trials": len(values)}
        metrics = {}

        def aggregate(name, data):
            data = [v for v in data if v is not None]
            if data:
                metrics[name] = {"median": statistics.median(data), "min": min(data), "max": max(data)}

        for group in [None, *values[0]["groups"]]:
            prefix = "total" if group is None else group
            for field in ["cpu_pct", "cpu_ms_request", "pss_mib", "rss_mib", "threads"]:
                aggregate(f"{prefix}.{field}", [(v if group is None else v["groups"][group])[field] for v in values])
        traffic = [v["traffic"] for v in values if v["traffic"]]
        for field in ["achieved_rps", "latency_p50_ms", "latency_p95_ms", "schedule_p95_ms"]:
            aggregate(field, [v[field] for v in traffic])
        for field in ["client_connections_opened", "client_connections_reused"]:
            aggregate(field, [v.get(field) for v in traffic])
        item["requests"] = sum(v["requests"] for v in traffic)
        item["errors"] = sum(len(v["errors"]) for v in traffic)
        item["cgroup_throttled_usec"] = sum(v["cgroup_cpu_end"].get("throttled_usec", 0) -
                                             v["cgroup_cpu_start"].get("throttled_usec", 0) for v in values)
        item["metrics"] = metrics
        summary.append(item)
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
