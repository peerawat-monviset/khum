use std::fs;
use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Instant;
use crate::state::AppState;
use crate::handler::send_response;

unsafe extern "C" {
    fn sysconf(name: std::os::raw::c_int) -> std::os::raw::c_long;
}

pub fn handle_metrics(stream: &mut TcpStream, state: Arc<AppState>) {
    let mem_current = read_self_rss_bytes().unwrap_or(0);

    let mem_limit = read_cgroup_metric("/sys/fs/cgroup/memory.max")
        .or_else(|_| read_cgroup_metric("/sys/fs/cgroup/memory/memory.limit_in_bytes"))
        .unwrap_or(0);

    let mut cpu_pct = 0.0;
    if let Ok(current_cpu_usec) = read_cpu_usec() {
        let mut tracker = state.cpu_tracker.lock().unwrap();
        let elapsed_wall = tracker.last_instant.elapsed();
        let elapsed_wall_usec = elapsed_wall.as_micros() as u64;

        if elapsed_wall_usec > 0 {
            let elapsed_cpu_usec = current_cpu_usec.saturating_sub(tracker.last_cpu_usec);
            cpu_pct = (elapsed_cpu_usec as f64 / elapsed_wall_usec as f64) * 100.0;
        }

        tracker.last_cpu_usec = current_cpu_usec;
        tracker.last_instant = Instant::now();
    }

    let json = format!(
        "{{\"mem_current_mb\":{:.2},\"mem_limit_mb\":{:.2},\"cpu_percent\":{:.2}}}",
        (mem_current as f64) / 1024.0 / 1024.0,
        (mem_limit as f64) / 1024.0 / 1024.0,
        cpu_pct
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );

    send_response(stream, &headers, Some(json.as_bytes()));
}

pub fn handle_sysinfo(stream: &mut TcpStream) {
    let os = read_line_matching("/etc/os-release", "PRETTY_NAME=")
        .map(|s| s.replace("PRETTY_NAME=", "").replace("\"", ""))
        .unwrap_or_else(|_| "Unknown OS".to_string());

    let kernel = fs::read_to_string("/proc/version")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<&str>>().join(" "))
        .unwrap_or_else(|_| "Unknown Kernel".to_string());

    let hostname = fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown Host".to_string());

    let hypervisor = fs::read_to_string("/sys/hypervisor/type")
        .map(|s| s.trim().to_string())
        .or_else(|_| fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "KVM/Hypervisor".to_string());

    let cpu_model = read_line_matching("/proc/cpuinfo", "model name")
        .map(|s| s.split(':').nth(1).unwrap_or("").trim().to_string())
        .unwrap_or_else(|_| "Unknown CPU".to_string());

    let cpu_speed = read_line_matching("/proc/cpuinfo", "cpu MHz")
        .map(|s| s.split(':').nth(1).unwrap_or("").trim().to_string() + " MHz")
        .unwrap_or_else(|_| "Unknown Speed".to_string());

    let cpu_cores = fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.lines().filter(|line| line.starts_with("processor")).count())
        .unwrap_or(1);

    let mem_total = read_line_matching("/proc/meminfo", "MemTotal:")
        .map(|s| s.split_whitespace().nth(1).unwrap_or("").to_string() + " KB")
        .unwrap_or_else(|_| "Unknown Mem".to_string());

    let uptime = fs::read_to_string("/proc/uptime")
        .map(|s| {
            if let Some(sec_str) = s.split_whitespace().next() {
                if let Ok(sec) = sec_str.parse::<f64>() {
                    let h = (sec / 3600.0) as u32;
                    let m = ((sec % 3600.0) / 60.0) as u32;
                    return format!("{}h {}m", h, m);
                }
            }
            "Unknown".to_string()
        })
        .unwrap_or_else(|_| "Unknown".to_string());

    let loadavg = fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<&str>>().join(" "))
        .unwrap_or_else(|_| "Unknown".to_string());

    // Deep Hardware Topology Specs
    let l1_data = read_sys_file("/sys/devices/system/cpu/cpu0/cache/index0/size");
    let l1_inst = read_sys_file("/sys/devices/system/cpu/cpu0/cache/index1/size");
    let l2 = read_sys_file("/sys/devices/system/cpu/cpu0/cache/index2/size");
    let l3 = read_sys_file("/sys/devices/system/cpu/cpu0/cache/index3/size");
    let cache_line = read_sys_file("/sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size")
        .parse::<u32>()
        .unwrap_or(64);
    
    let numa_nodes = get_numa_nodes();
    let page_size = unsafe { sysconf(30) }; // _SC_PAGESIZE is 30 on Linux

    let json = format!(
        "{{\"os\":\"{}\",\"kernel\":\"{}\",\"hostname\":\"{}\",\"hypervisor\":\"{}\",\
          \"cpu_model\":\"{}\",\"cpu_speed\":\"{}\",\"cpu_cores\":{},\
          \"cpu_flags\":\"{}\",\"l1_data\":\"{}\",\"l1_inst\":\"{}\",\
          \"l2\":\"{}\",\"l3\":\"{}\",\"cache_line\":{},\"numa_nodes\":{},\
          \"page_size\":{},\"mem_total\":\"{}\",\"uptime\":\"{}\",\"loadavg\":\"{}\"}}",
        os, kernel, hostname, hypervisor,
        cpu_model, cpu_speed, cpu_cores,
        get_simd_flags(), l1_data, l1_inst,
        l2, l3, cache_line, numa_nodes,
        page_size, mem_total, uptime, loadavg
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );

    send_response(stream, &headers, Some(json.as_bytes()));
}

pub fn read_cgroup_metric(path: &str) -> Result<u64, io::Error> {
    let content = fs::read_to_string(path)?;
    let val = content.trim().parse::<u64>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(val)
}

pub fn read_cpu_usec() -> Result<u64, io::Error> {
    if let Ok(content) = fs::read_to_string("/sys/fs/cgroup/cpu.stat") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "usage_usec" {
                if let Ok(val) = parts[1].parse::<u64>() {
                    return Ok(val);
                }
            }
        }
    }
    if let Ok(content) = fs::read_to_string("/sys/fs/cgroup/cpuacct/cpuacct.usage") {
        if let Ok(ns) = content.trim().parse::<u64>() {
            return Ok(ns / 1000);
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "CPU cgroup files not found"))
}

fn read_line_matching(path: &str, prefix: &str) -> Result<String, io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with(prefix) {
            return Ok(line);
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "Prefix not found"))
}

fn read_self_rss_bytes() -> Result<u64, io::Error> {
    let file = fs::File::open("/proc/self/status")?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("VmRSS:") {
            let mut split = line.split_whitespace();
            let _prefix = split.next();
            if let Some(val_str) = split.next() {
                if let Ok(kb) = val_str.parse::<u64>() {
                    return Ok(kb * 1024);
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "VmRSS not found"))
}

fn read_sys_file(path: &str) -> String {
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "N/A".to_string())
}

fn get_simd_flags() -> String {
    let mut flags = Vec::new();
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("flags") {
                let parts = line.split(':').nth(1).unwrap_or("").split_whitespace();
                let targets = ["sse", "sse2", "sse3", "ssse3", "sse4_1", "sse4_2", "avx", "avx2", "avx512f", "aes", "sha_ni", "hypervisor"];
                for part in parts {
                    if targets.contains(&part) {
                        flags.push(part.to_string());
                    }
                }
                break;
            }
        }
    }
    flags.join(" ")
}

fn get_numa_nodes() -> u32 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir("/sys/devices/system/node") {
        for entry in entries {
            if let Ok(entry) = entry {
                if entry.file_name().to_string_lossy().starts_with("node") {
                    count += 1;
                }
            }
        }
    }
    if count == 0 { 1 } else { count }
}
