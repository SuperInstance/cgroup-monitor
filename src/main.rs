use std::fs;
use std::path::Path;

fn read_cgroup_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| "<unreadable>".into())
}

struct CgroupInfo {
    name: String,
    cpu_usage: String,
    memory_current: String,
    memory_max: String,
    pids_current: String,
}

fn scan_cgroup(cgroup_dir: &Path) -> Option<CgroupInfo> {
    if !cgroup_dir.exists() {
        return None;
    }
    Some(CgroupInfo {
        name: cgroup_dir.file_name()?.to_str()?.to_string(),
        cpu_usage: read_cgroup_file(&cgroup_dir.join("cpu.stat")),
        memory_current: read_cgroup_file(&cgroup_dir.join("memory.current")),
        memory_max: read_cgroup_file(&cgroup_dir.join("memory.max")),
        pids_current: read_cgroup_file(&cgroup_dir.join("pids.current")),
    })
}

fn main() {
    let cgroup_root = Path::new("/sys/fs/cgroup");
    if !cgroup_root.exists() {
        eprintln!("cgroup v2 not found at {}", cgroup_root.display());
        return;
    }

    let subtree = cgroup_root.join("system.slice");
    println!("Scanning cgroups under {}...\n", subtree.display());

    let mut total_groups = 0;
    if let Ok(entries) = fs::read_dir(&subtree) {
        for entry in entries.flatten() {
            if let Some(info) = scan_cgroup(&entry.path()) {
                total_groups += 1;
                println!("📦 {}", info.name);
                println!("   memory.current: {}", info.memory_current.trim());
                println!("   memory.max:     {}", info.memory_max.trim());
                println!("   pids.current:   {}", info.pids_current.trim());
                println!();
            }
        }
    }

    println!("Total cgroups scanned: {total_groups}");

    // Read root-level stats
    let cpu_weight = read_cgroup_file(&cgroup_root.join("cpu.weight"));
    let cpu_max = read_cgroup_file(&cgroup_root.join("cpu.max"));
    println!("\nRoot cgroup:");
    println!("  cpu.weight: {}", cpu_weight.trim());
    println!("  cpu.max:    {}", cpu_max.trim());
}
