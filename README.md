# Cgroup Monitor — Linux Control Group v2 Resource Scanner

**cgroup-monitor** is a Linux utility that reads cgroup v2 control files to report per-service CPU usage, memory consumption, and process counts. It walks the `/sys/fs/cgroup/` hierarchy and parses the kernel-exposed accounting files that every systemd slice, container, and process group publishes.

## Why It Matters

Every modern Linux system uses cgroups v2 to enforce resource limits on processes — Docker, systemd, Kubernetes, and Podman all map to the same cgroup filesystem. When a container OOM-kills or a systemd service pegs a CPU, the data is sitting right there in `/sys/fs/cgroup/`. This crate provides a programmatic way to scan those files without shelling out to `systemd-cgtop` or `docker stats`. Engineers building observability dashboards, resource controllers, or fleet monitoring agents need direct cgroup parsing — it's the ground truth for resource consumption on Linux.

## How It Works

Linux cgroup v2 exposes a unified filesystem at `/sys/fs/cgroup/`. Each cgroup directory contains well-defined control files:

| File | Description |
|---|---|
| `cpu.stat` | CPU accounting: `usage_usec`, `user_usec`, `system_usec` |
| `memory.current` | Current memory usage in bytes |
| `memory.max` | Memory limit (bytes or `max` for unlimited) |
| `pids.current` | Number of processes currently in the cgroup |
| `cpu.weight` | CPU scheduler weight (1–10000, default 100) |
| `cpu.max` | CPU bandwidth limit: `$MAX $PERIOD` (e.g., `50000 100000` = 50%) |

The monitor performs a single-pass scan:

1. **Enumerate** `system.slice/*/` — each subdirectory is a systemd service cgroup.
2. **Read** `cpu.stat`, `memory.current`, `memory.max`, `pids.current` for each.
3. **Aggregate** and print a summary with totals.

The scan is `O(n)` where `n` is the number of cgroups — each read is a single `read(2)` syscall on a pseudo-file, so there is no parsing overhead beyond splitting on whitespace. Memory usage is `O(1)` per cgroup (one `CgroupInfo` struct at a time).

## Quick Start

```bash
# Build and run as root (cgroup files require read access)
cargo run --release
```

```rust
use std::path::Path;
use std::fs;

fn read_memory_current(cgroup_name: &str) -> u64 {
    let path = format!("/sys/fs/cgroup/system.slice/{cgroup_name}/memory.current");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}
```

## API

| Struct / Function | Description |
|---|---|
| `CgroupInfo` | Snapshot: `name`, `cpu_usage`, `memory_current`, `memory_max`, `pids_current`. |
| `scan_cgroup(path)` | Read all control files for a single cgroup directory. Returns `Option<CgroupInfo>`. |
| `read_cgroup_file(path)` | Read a cgroup control file as a string (`"<unreadable>"` on error). |

## Architecture Notes

In the SuperInstance fleet, cgroup-monitor feeds the η (evaluation) side of γ + η = C — it observes the actual resource state of running instances. The data it collects flows into the metrics pipeline for capacity planning and overload detection. See [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Linux Kernel Documentation. *cgroup v2*. <https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html>
2. systemd Documentation. *systemd.resource-control(5)*. — Describes `cpu.weight`, `cpu.max`, `memory.max` mapping.
3. Kerrisk, M. (2010). *The Linux Programming Interface*, Ch. 25–28. NoStarch Press.

## License

MIT
