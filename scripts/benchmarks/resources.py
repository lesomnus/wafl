#!/usr/bin/env python3
"""Linux benchmark: Wasmtime CLI vs WAFL proxy vs direct guest sockets.

Only child server PIDs are charged. Raw samples and errors are preserved in JSON.
Requires Python 3.11+, Wasmtime 51, and a release build of wafl.
Optional Traefik measures backend, gateway, and combined process costs.
"""
import argparse
import asyncio
import base64
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import statistics
import subprocess
import tarfile
import time
import urllib.request
import uuid


def read_optional(filename):
    try:
        return Path(filename).read_text().strip()
    except OSError:
        return None


def percentile(values, fraction):
    values = sorted(values)
    return values[min(len(values) - 1, int((len(values) - 1) * fraction))] if values else None


def snapshot(processes):
    result = {name: 0 for name in ["cpu_seconds", "rss_kib", "pss_kib", "private_kib", "threads"]}
    for process in processes:
        if process.poll() is not None:
            raise RuntimeError(f"server {process.pid} exited: {process.returncode}")
        root = Path(f"/proc/{process.pid}")
        stat = (root / "stat").read_text().rsplit(")", 1)[1].split()
        result["cpu_seconds"] += (int(stat[11]) + int(stat[12])) / os.sysconf("SC_CLK_TCK")
        fields = {}
        for line in (root / "smaps_rollup").read_text().splitlines()[1:]:
            key, value = line.split(":", 1)
            fields[key] = int(value.split()[0])
        result["rss_kib"] += fields["Rss"]
        result["pss_kib"] += fields["Pss"]
        result["private_kib"] += fields["Private_Clean"] + fields["Private_Dirty"]
        for line in (root / "status").read_text().splitlines():
            if line.startswith("Threads:"):
                result["threads"] += int(line.split()[1])
    return result


def ports(count):
    reservations = [socket.socket() for _ in range(count)]
    try:
        for sock in reservations:
            sock.bind(("127.0.0.1", 0))
        return [sock.getsockname()[1] for sock in reservations]
    finally:
        for sock in reservations:
            sock.close()


def request_sync(port, path="/", body=None):
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(f"http://127.0.0.1:{port}{path}", data=data,
                                     headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=120) as response:
        return response.read()


async def request(target, path="/"):
    port, instance, host = target
    reader, writer = await asyncio.open_connection("127.0.0.1", port)
    try:
        route = f"wafl-instance: {instance}\r\n" if instance else ""
        writer.write((f"GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n{route}\r\n").encode())
        await writer.drain()
        response = await reader.read()
        header, body = response.split(b"\r\n\r\n", 1)
        if header.split(b"\r\n")[0] != b"HTTP/1.1 200 OK" or body != b"ok\n":
            raise RuntimeError(f"unexpected response: {response[:200]!r}")
        return header
    finally:
        writer.close()
        await writer.wait_closed()


async def ready(processes, targets):
    deadline = time.monotonic() + 120
    for target in targets:
        while True:
            snapshot(processes)
            try:
                await asyncio.wait_for(request(target), 2)
                break
            except (OSError, asyncio.TimeoutError, RuntimeError):
                if time.monotonic() >= deadline:
                    raise RuntimeError(f"readiness timeout: {target}")
                await asyncio.sleep(0.05)


async def load(targets, seconds, rate, path):
    latencies, lateness, errors = [], [], []
    start = time.monotonic()

    async def one(index, scheduled):
        actual = time.monotonic()
        lateness.append(max(0, actual - scheduled) * 1000)
        try:
            await asyncio.wait_for(request(targets[index % len(targets)], path), 5)
            latencies.append((time.monotonic() - actual) * 1000)
        except Exception as error:
            errors.append(repr(error))

    tasks = []
    for index in range(round(seconds * rate)):
        scheduled = start + index / rate
        await asyncio.sleep(max(0, scheduled - time.monotonic()))
        tasks.append(asyncio.create_task(one(index, scheduled)))
    await asyncio.gather(*tasks)
    await asyncio.sleep(max(0, start + seconds - time.monotonic()))
    elapsed = time.monotonic() - start
    return {"offered_rps": rate, "requests": len(tasks), "ok": len(latencies), "errors": errors,
            "elapsed": elapsed, "achieved_rps": len(latencies) / elapsed,
            "latency_p50_ms": percentile(latencies, .50), "latency_p95_ms": percentile(latencies, .95),
            "schedule_p95_ms": percentile(lateness, .95)}


def grouped_snapshot(groups):
    groups = {name: snapshot(processes) for name, processes in groups.items()}
    total = {key: sum(values[key] for values in groups.values()) for key in next(iter(groups.values()))}
    return {**total, "groups": groups}


def cgroup_cpu():
    text = read_optional("/sys/fs/cgroup/cpu.stat")
    return dict((key, int(value)) for key, value in (line.split() for line in text.splitlines())) if text else {}


def cpu_counters(groups):
    counters = {}
    for name, processes in groups.items():
        total = 0
        for process in processes:
            stat = Path(f"/proc/{process.pid}/stat").read_text().rsplit(")", 1)[1].split()
            total += int(stat[11]) + int(stat[12])
        counters[name] = total / os.sysconf("SC_CLK_TCK")
    return counters


def tcp_connections(ports):
    pairs = set()
    for line in Path("/proc/net/tcp").read_text().splitlines()[1:]:
        fields = line.split()
        if fields[3] == "0A":  # Exclude listening sockets.
            continue
        if any(int(address.rsplit(":", 1)[1], 16) in ports for address in fields[1:3]):
            pairs.add(tuple(sorted(fields[1:3])))
    return pairs


def metrics(samples, elapsed, traffic, cpu):
    return {"cpu_seconds": cpu, "cpu_pct": 100 * cpu / elapsed,
            "pss_mib": statistics.median(s["pss_kib"] for s in samples) / 1024,
            "rss_mib": statistics.median(s["rss_kib"] for s in samples) / 1024,
            "private_mib": statistics.median(s["private_kib"] for s in samples) / 1024,
            "threads": statistics.median(s["threads"] for s in samples),
            "cpu_ms_request": 1000 * cpu / traffic["ok"] if traffic and traffic["ok"] else None}


async def measure(groups, targets, seconds, name, rate=0, path="/", connection_ports=None):
    # Reading smaps_rollup can block on kernel memory accounting. Never let it
    # stall the event loop that schedules the offered request rate.
    sampling_ms = []

    async def take_snapshot():
        before = time.monotonic()
        value = await asyncio.to_thread(grouped_snapshot, groups)
        sampling_ms.append((time.monotonic() - before) * 1000)
        return value

    connections_before = await asyncio.to_thread(tcp_connections, connection_ports) if connection_ports else None
    initial = await take_snapshot()
    cpu_start = cpu_counters(groups)
    start = time.monotonic()
    cgroup_start = cgroup_cpu()
    samples = [{"time": 0, **initial}]
    stop = asyncio.Event()

    async def sampler():
        while not stop.is_set():
            try:
                await asyncio.wait_for(stop.wait(), .5)
            except asyncio.TimeoutError:
                value = await take_snapshot()
                samples.append({"time": time.monotonic() - start, **value})

    sampling = asyncio.create_task(sampler())
    try:
        traffic = await load(targets, seconds, rate, path) if rate else None
        if not rate:
            await asyncio.sleep(seconds)
    finally:
        elapsed = time.monotonic() - start
        cpu_end = cpu_counters(groups)
        cgroup_end = cgroup_cpu()
        stop.set()
        await sampling
    final = await take_snapshot()
    connections_after = await asyncio.to_thread(tcp_connections, connection_ports) if connection_ports else None
    samples.append({"time": elapsed, **final})
    cpu = {name: cpu_end[name] - cpu_start[name] for name in groups}
    group_metrics = {name: metrics([s["groups"][name] for s in samples], elapsed, traffic, cpu[name])
                     for name in groups}
    return {"phase": name, "elapsed": elapsed, **metrics(samples, elapsed, traffic, sum(cpu.values())),
            "groups": group_metrics, "traffic": traffic, "samples": samples,
            "sampling_ms": sampling_ms,
            "connections": None if connections_before is None else {
                "ports": connection_ports, "initial": len(connections_before),
                "final": len(connections_after), "new_tuples": len(connections_after - connections_before)},
            "cpu_start": cpu_start, "cpu_end": cpu_end,
            "cgroup_cpu_start": cgroup_start, "cgroup_cpu_end": cgroup_end}


async def trial(args, case, memory, repeat, bundles, directory):
    folder = directory / f"{case}-{memory}-{repeat}"
    folder.mkdir()
    processes, outputs, commands, instances = [], [], [], []
    groups = {"backend": []}
    environment = {**os.environ, "TOKIO_WORKER_THREADS": str(args.workers), "RAYON_NUM_THREADS": str(args.workers),
                   "GOMAXPROCS": str(args.workers),
                   "BENCH_MEMORY_MIB": str(memory), "BENCH_WORK": str(args.work)}

    def spawn(command, group="backend"):
        output = (folder / f"{len(processes)}.log").open("wb")
        outputs.append(output)
        commands.append(command)
        process = subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL,
                                   stdout=output, stderr=subprocess.STDOUT)
        processes.append(process)
        groups.setdefault(group, []).append(process)
        return process

    start = time.monotonic()
    try:
        chosen = ports(2)
        if case == "cli":
            for index, port in enumerate(chosen):
                spawn([args.wasmtime, "run", "-C", "cache=n,parallel-compilation=n", "-O", "pooling-allocator=n",
                       "-W", "max-memory-size=67108864,max-memories=1,max-tables=8,max-table-elements=65536,max-instances=32",
                       "-S", "inherit-network=y,tcp=y,udp=n,allow-ip-name-lookup=y,max-resources=1024",
                       "--env", f"PORT={port}", "--env", f"BENCH_MEMORY_MIB={memory}",
                       "--env", f"BENCH_WORK={args.work}", args.guest])
            targets = [(port, None, "bench.local") for port in chosen]
        else:
            ingress, control = chosen
            config = {"listen": f"127.0.0.1:{ingress}", "control": f"127.0.0.1:{control}",
                      "directory": str(folder / "cache")}
            if case == "direct":
                config["direct"] = {"listen": "127.0.0.1", "advertise": "127.0.0.1",
                                    "directory": str(folder / "endpoints"), "first": 20000, "last": 20999}
            config_file = folder / "runner.json"
            config_file.write_text(json.dumps(config, indent=2) + "\n")
            spawn([args.wafl, "serve", "runner", "--config", str(config_file)])
            # Wait for the management listener; readiness requests are not timed.
            for _ in range(2400):
                snapshot(processes)
                try:
                    connection = socket.create_connection(("127.0.0.1", control), timeout=.1)
                    connection.close()
                    break
                except OSError:
                    await asyncio.sleep(.05)
            else:
                raise RuntimeError("runner management readiness timeout")
            targets = []
            for index in range(2):
                added = json.loads(await asyncio.to_thread(request_sync, control, "/wafl.InstanceService/Add", {
                    "tenant": {"id": base64.b64encode(uuid.UUID("00000000-0000-8000-8001-000000000001").bytes).decode()},
                    "alias": f"bench-{index}",
                    "manifest": {"bundle": {"url": bundles[index].as_uri()}, "port": 8080,
                                 "idleMs": "0" if case == "direct" else "3600000",
                                 "memoryBytes": "67108864", "startupMs": "120000",
                                 "env": {"BENCH_MEMORY_MIB": str(memory), "BENCH_WORK": str(args.work)}}}))
                instance = str(uuid.UUID(bytes=base64.b64decode(added["id"])))
                instances.append(added)
                if case == "direct":
                    assert added["direct"] and added["phase"] == "PHASE_RUNNING"
                    targets.append((int(added["endpoint"]["address"].rsplit(":", 1)[1]), None, "bench.local"))
                else:
                    targets.append((ingress, instance, "bench.local"))
        await ready(processes, targets)
        connection_ports = sorted({port for port, _, _ in targets}) if args.connections else None
        if args.gateway:
            gateway_port = ports(1)[0]
            dynamic = folder / "routes"
            dynamic.mkdir()
            lines = []
            for index, (port, instance, _) in enumerate(targets):
                lines.extend([f"[http.routers.app{index}]", f'rule = "Host(`bench-{index}.local`)"',
                              'entryPoints = ["web"]', f'service = "app{index}"'])
                if instance:
                    lines.extend([f'middlewares = ["route{index}"]',
                                  f"[http.middlewares.route{index}.headers.customRequestHeaders]",
                                  f'"wafl-instance" = "{instance}"'])
                lines.extend([f"[[http.services.app{index}.loadBalancer.servers]]", f'url = "http://127.0.0.1:{port}"'])
            (dynamic / "apps.toml").write_text("\n".join(lines) + "\n")
            spawn([args.traefik, f"--entrypoints.web.address=127.0.0.1:{gateway_port}",
                   f"--providers.file.directory={dynamic}", "--log.level=ERROR"], "gateway")
            targets = [(gateway_port, None, f"bench-{index}.local") for index in range(2)]
            await ready(processes, targets)
        startup = time.monotonic() - start
        await asyncio.sleep(2)
        phases = [await measure(groups, targets, args.seconds, "idle", connection_ports=connection_ports)]
        for name, rate, path in [("http", args.rps, "/"), ("work", args.work_rps, "/work")]:
            warmup = await load(targets, 2, rate, path)
            if warmup["errors"]:
                raise RuntimeError(f"warmup errors: {warmup['errors'][:3]}")
            phases.append(await measure(groups, targets, args.seconds, name, rate, path, connection_ports))
        result = {"case": case, "memory_mib_per_guest": memory, "repeat": repeat,
                  "startup_seconds": startup, "pids": [p.pid for p in processes], "commands": commands,
                  "instances": instances, "gateway": args.gateway, "phases": phases}
        print(json.dumps({"case": case, "memory": memory, "repeat": repeat,
                          "phases": [{k: phase[k] for k in ["phase", "pss_mib", "rss_mib", "cpu_pct", "cpu_ms_request"]}
                                     for phase in phases]}), flush=True)
        return result
    finally:
        # Each trial is isolated: stop the gateway first and wait for it before
        # killing backends. No stale routes/pools survive into the next trial.
        for process in reversed(processes):
            if process.poll() is None:
                process.terminate()
            try:
                await asyncio.to_thread(process.wait, timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                await asyncio.to_thread(process.wait)
        for output in outputs:
            output.close()


async def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["wasmtime", "wafl", "guest", "output"]:
        parser.add_argument(f"--{name}", required=True)
    parser.add_argument("--traefik", help="Traefik binary, required with --gateway")
    parser.add_argument("--gateway", action="store_true", help="Put a Traefik gateway in front of each case")
    parser.add_argument("--connections", action="store_true", help="Diagnostic: observe new backend TCP tuples via /proc/net/tcp")
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--seconds", type=float, default=10)
    parser.add_argument("--rps", type=int, default=500)
    parser.add_argument("--work-rps", type=int, default=200)
    parser.add_argument("--work", type=int, default=200_000)
    parser.add_argument("--workers", type=int, default=2)
    parser.add_argument("--memory", type=int, nargs="+", default=[0])
    parser.add_argument("--cases", nargs="+", choices=["cli", "proxy", "direct"], default=["cli", "proxy", "direct"])
    args = parser.parse_args()
    if args.gateway and not args.traefik:
        parser.error("--gateway requires --traefik")
    if min(args.seconds, args.repeat, args.rps, args.work_rps, args.workers) <= 0 or min(args.memory) < 0:
        parser.error("durations, rates, repeats and workers must be positive; memory must be nonnegative")
    if args.traefik:
        args.traefik = str(Path(args.traefik).resolve())
    for name in ["wasmtime", "wafl", "guest", "output"]:
        setattr(args, name, str(Path(getattr(args, name)).resolve()))
    directory = Path(args.output).with_suffix(".files")
    directory.mkdir(parents=True)
    bundles = []
    for index, guest in enumerate([args.guest, args.guest]):
        bundle = directory / f"bundle-{index}.tar.gz"
        if index == 1:
            bundles.append(bundles[0])
            continue
        with tarfile.open(bundle, "w:gz") as archive:
            archive.add(guest, arcname="main.wasm")
            data = tarfile.TarInfo("data")
            data.type = tarfile.DIRTYPE
            data.mode = 0o755
            archive.addfile(data)
        bundles.append(bundle)
    report = {"arguments": vars(args), "date_utc": datetime.now(timezone.utc).isoformat(),
              "git_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
              "platform": platform.platform(),
              "cpu": next((line.split(":", 1)[1].strip() for line in (read_optional("/proc/cpuinfo") or "").splitlines() if line.startswith("model name")), None),
              "cpu_max": read_optional("/sys/fs/cgroup/cpu.max"), "memory_max": read_optional("/sys/fs/cgroup/memory.max"),
              "wasmtime_version": subprocess.check_output([args.wasmtime, "--version"], text=True).strip(),
              "binary_sha256": {name: hashlib.sha256(Path(getattr(args, name)).read_bytes()).hexdigest()
                                for name in ["wasmtime", "wafl"] + (["traefik"] if args.gateway else [])},
              "guest_sha256": hashlib.sha256(Path(args.guest).read_bytes()).hexdigest(),
              "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), "trials": []}
    if args.gateway:
        report["traefik_version"] = subprocess.check_output([args.traefik, "version"], text=True).strip()
    for repeat in range(args.repeat):
        # Rotate host order to reduce systematic warm-cache/time-of-run bias.
        cases = args.cases[repeat % len(args.cases):] + args.cases[:repeat % len(args.cases)]
        for memory in args.memory:
            for case in cases:
                report["trials"].append(await trial(args, case, memory, repeat, bundles, directory))
                Path(args.output).write_text(json.dumps(report, indent=2) + "\n")
    if any(phase["traffic"] and phase["traffic"]["errors"] for trial_result in report["trials"] for phase in trial_result["phases"]):
        raise RuntimeError("benchmark completed with request failures; see JSON")


if __name__ == "__main__":
    asyncio.run(main())
