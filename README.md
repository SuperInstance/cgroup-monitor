# cgroup-monitor

A Linux **cgroup v2 resource monitor** that scans `/sys/fs/cgroup/` to report per-cgroup CPU usage, memory consumption, memory limits, and process counts. Designed for container observability, system health monitoring, and resource debugging.

## Why It Matters

cgroup v2 is the unified resource control system in modern Linux kernels (5.x+). It is the backbone of:

- **Container resource limits** — Docker, Podman, and Kubernetes all use cgroups
- **systemd service isolation** — each `.service` unit has its own cgroup
- **OOM prevention** — `memory.max` prevents runaway processes from crashing the system
- **Fair CPU scheduling** — `cpu.weight` controls CPU time distribution

Monitoring cgroup usage is essential for:

- **Capacity planning** — identifying which services consume the most resources
- **Anomaly detection** — spotting memory leaks or CPU spikes early
- **Cost attribution** — per-tenant resource accounting in multi-tenant systems
- **SRE dashboards** — real-time observability of service-level resource consumption

## How It Works

### cgroup v2 Hierarchy

cgroup v2 uses a single unified hierarchy mounted at `/sys/fs/cgroup/`. Each cgroup directory contains control files:

| File | Description |
|------|-------------|
| `memory.current` | Current memory usage in bytes |
| `memory.max` | Memory limit (bytes, or "max" for unlimited) |
| `cpu.stat` | CPU usage statistics (usec, user, system) |
| `pids.current` | Current process count |
| `cpu.weight` | CPU scheduler weight (1–10000) |
| `cpu.max` | CPU bandwidth limit (quota period) |

### Scanner Architecture

The monitor performs a directory scan of `system.slice/` (where systemd places services):

```text
/sys/fs/cgroup/
├── cpu.stat          ← root cgroup stats
├── cpu.weight
├── cpu.max
└── system.slice/
    ├── service-a.service/
    │   ├── memory.current
    │   ├── memory.max
    │   ├── cpu.stat
    │   └── pids.current
    ├── service-b.service/
    │   └── ...
    └── ...
```

For each child cgroup, it reads the four key files and aggregates them into a `CgroupInfo` struct.

### Memory Pressure Model

The effective memory pressure of a cgroup is:

$$P = \frac{\text{memory.current}}{\text{memory.max}}$$

When $P \to 1$, the kernel's OOM killer activates for that cgroup. This monitor reports raw values, allowing downstream tools to compute pressure.

### Complexity Analysis

| Operation | Time | Space |
|-----------|------|-------|
| `read_cgroup_file(path)` | O(1) | O(1) |
| `scan_cgroup(dir)` | O(1) (4 file reads) | O(1) |
| Full scan of system.slice | O(n) where n = cgroups | O(n) |
| Root cgroup stats | O(1) (2 reads) | O(1) |

Each file read is a single `fs::read_to_string` — no parsing overhead beyond trimming whitespace.

## Quick Start

```bash
# Run on any Linux system with cgroup v2
cargo run
```

Sample output:

```
Scanning cgroups under /sys/fs/cgroup/system.slice/...

📦 docker.service
   memory.current: 134217728
   memory.max:     536870912
   pids.current:   47

📦 nginx.service
   memory.current: 8388608
   memory.max:     67108864
   pids.current:   3

Total cgroups scanned: 2

Root cgroup:
  cpu.weight: 100
  cpu.max:    max 100000
```

## API

| Struct / Function | Description |
|-------------------|-------------|
| `CgroupInfo` | Per-cgroup: name, cpu_usage, memory_current, memory_max, pids_current |
| `scan_cgroup(path) → Option<CgroupInfo>` | Scan a single cgroup directory |
| `read_cgroup_file(path) → String` | Read a cgroup control file (graceful fallback) |

## Architecture Notes

The **γ + η = C** link: the filesystem scanner (γ) extracts raw resource counters from cgroup control files, while the kernel's cgroup subsystem (η) maintains those files as authoritative resource accounts. Together they conserve the observability invariant C — the reported values are point-in-time snapshots of the kernel's own resource accounting. The scanner is intentionally read-only and never modifies cgroup state.

## References

- Kernel.org: *cgroup v2 — Linux Kernel Documentation.* <https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html>
- systemd documentation: *Resource Control.* <https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html>
- Tejun Heo (2017). *cgroup v2: The Unified Hierarchy.* LWN.net.
- Kerrisk, M. (2024). *The Linux Programming Interface,* 2nd ed. (cgroups chapter.)
- Kubernetes: *Understanding cgroup v2.* SIG-Node documentation.

## License

MIT
