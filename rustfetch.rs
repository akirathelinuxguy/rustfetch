use std::fs;
use std::process::Command;
use std::thread;
use std::sync::Arc;

// ==== CONFIGURATION ====
const ENABLE_GPU_DETECTION: bool = true;
const USE_COLOR_OUTPUT: bool = true;

// ==== COLOR CODES ====
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const MAGENTA: &str = "\x1b[95m";

fn main() {
    // Parallel data collection - ALL info gathered simultaneously
    let hostname_handle = thread::spawn(|| get_hostname());
    let os_handle = thread::spawn(|| get_os_name());
    let kernel_handle = thread::spawn(|| get_kernel());
    let uptime_handle = thread::spawn(|| get_uptime());
    let shell_handle = thread::spawn(|| get_shell());
    let cpu_handle = thread::spawn(|| get_cpu());
    let memory_handle = thread::spawn(|| get_memory());
    let gpu_handle = thread::spawn(|| {
        if ENABLE_GPU_DETECTION {
            get_all_gpus()
        } else {
            vec!["GPU detection disabled".to_string()]
        }
    });

    // Wait for all threads and collect results
    let hostname = hostname_handle.join().unwrap();
    let os_name = os_handle.join().unwrap();
    let kernel = kernel_handle.join().unwrap();
    let uptime = uptime_handle.join().unwrap();
    let shell = shell_handle.join().unwrap();
    let cpu = cpu_handle.join().unwrap();
    let memory = memory_handle.join().unwrap();
    let gpus = gpu_handle.join().unwrap();

    let logo = get_os_logo(&os_name);
    let info = format_info(&hostname, &os_name, &kernel, &uptime, &shell, &cpu, &memory, &gpus);

    display_side_by_side(&logo, &info);
}

#[inline(always)]
fn colorize(text: &str, color: &str) -> String {
    if USE_COLOR_OUTPUT {
        format!("{}{}{}", color, text, RESET)
    } else {
        text.to_string()
    }
}

fn get_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .or_else(|_| {
            Command::new("hostname")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, ""))
        })
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_os_name() -> String {
    // Fast path: read os-release once
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if let Some(name) = line.strip_prefix("PRETTY_NAME=") {
                return name.trim_matches('"').to_string();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            if let Ok(version) = String::from_utf8(output.stdout) {
                return format!("macOS {}", version.trim());
            }
        }
        return "macOS".to_string();
    }

    #[cfg(target_os = "freebsd")]
    return "FreeBSD".to_string();

    #[cfg(target_os = "openbsd")]
    return "OpenBSD".to_string();

    #[cfg(target_os = "netbsd")]
    return "NetBSD".to_string();

    "Unknown OS".to_string()
}

fn get_kernel() -> String {
    Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_uptime() -> String {
    // Fast path: parse /proc/uptime directly
    if let Ok(contents) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_str) = contents.split_whitespace().next() {
            if let Ok(secs) = uptime_str.parse::<f64>() {
                let days = (secs / 86400.0) as u64;
                let hours = ((secs % 86400.0) / 3600.0) as u64;
                let mins = ((secs % 3600.0) / 60.0) as u64;

                return match (days, hours) {
                    (d, h) if d > 0 => format!("{}d {}h {}m", d, h, mins),
                    (_, h) if h > 0 => format!("{}h {}m", h, mins),
                    _ => format!("{}m", mins),
                };
            }
        }
    }

    Command::new("uptime")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[inline(always)]
fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_cpu() -> String {
    // Read /proc/cpuinfo ONCE and parse everything
    if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        let mut model = None;
        let mut cores = 0;

        for line in contents.lines() {
            if model.is_none() && line.starts_with("model name") {
                model = line.split(':').nth(1).map(|s| s.trim().to_string());
            } else if line.starts_with("processor") {
                cores += 1;
            }
        }

        if let Some(m) = model {
            return format!("{} ({} cores)", m, cores);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let cpu_name = Command::new("sysctl")
            .args(&["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let cores = Command::new("sysctl")
            .args(&["-n", "hw.ncpu"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        if let (Some(name), Some(c)) = (cpu_name, cores) {
            return format!("{} ({} cores)", name, c);
        }
    }

    "Unknown CPU".to_string()
}

fn get_memory() -> String {
    // Fast path: parse /proc/meminfo once
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        let mut total = None;
        let mut available = None;

        for line in contents.lines() {
            if total.is_none() && line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if available.is_none() && line.starts_with("MemAvailable:") {
                available = line.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            }
            
            if total.is_some() && available.is_some() {
                break;
            }
        }

        if let (Some(t), Some(a)) = (total, available) {
            let used = t - a;
            return format!(
                "{:.1} GiB / {:.1} GiB",
                used as f64 / 1048576.0,
                t as f64 / 1048576.0
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {
            if let Ok(mem_str) = String::from_utf8(output.stdout) {
                if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                    return format!("{:.1} GiB", mem_bytes as f64 / 1073741824.0);
                }
            }
        }
    }

    "Unknown".to_string()
}

fn get_all_gpus() -> Vec<String> {
    let mut gpus = Vec::new();

    // Method 1: NVIDIA via nvidia-smi (fastest for NVIDIA)
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=gpu_name", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(nvidia_output) = String::from_utf8(output.stdout) {
                gpus.extend(
                    nvidia_output
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .map(|line| format!("NVIDIA {}", line))
                );
            }
        }
    }

    // Method 2: lspci for all GPUs
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            if let Ok(lspci_output) = String::from_utf8(output.stdout) {
                for line in lspci_output.lines() {
                    if line.contains("VGA compatible controller") || line.contains("3D controller") {
                        if let Some(gpu_info) = line.split(':').nth(2) {
                            let gpu_name = gpu_info.trim();
                            // Skip if nvidia-smi already found it
                            let is_duplicate = gpus.iter().any(|g| {
                                gpu_name.contains(&g.replace("NVIDIA ", ""))
                            });
                            if !is_duplicate {
                                gpus.push(gpu_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 3: macOS GPU detection
    #[cfg(target_os = "macos")]
    {
        if gpus.is_empty() {
            if let Ok(output) = Command::new("system_profiler").arg("SPDisplaysDataType").output() {
                if let Ok(gpu_info) = String::from_utf8(output.stdout) {
                    for line in gpu_info.lines() {
                        if let Some(model) = line.strip_prefix("      Chipset Model:") {
                            gpus.push(model.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    if gpus.is_empty() {
        gpus.push("No GPU detected".to_string());
    }

    gpus
}

fn get_os_logo(os_name: &str) -> Vec<String> {
    let os_lower = os_name.to_lowercase();

    if os_lower.contains("arch") || os_lower.contains("cachy") {
        vec![
            "      /\\      ",
            "     /  \\     ",
            "    /\\   \\    ",
            "   /  \\   \\   ",
            "  /    \\   \\  ",
            " /______\\___\\ ",
        ]
    } else if os_lower.contains("ubuntu") {
        vec![
            "         _     ",
            "     ---(_)    ",
            " _/  ---  \\    ",
            "(_) |   |      ",
            "  \\  --- _/    ",
            "     ---(_)    ",
        ]
    } else if os_lower.contains("debian") {
        vec![
            "  _____  ",
            " /  __ \\ ",
            "|  /    |",
            "|  \\___- ",
            " -_      ",
            "   --_   ",
        ]
    } else if os_lower.contains("fedora") {
        vec![
            "      _____    ",
            "     /   __)\\  ",
            "     |  /  \\ \\ ",
            "  ___|  |__/ / ",
            " / (_    _)_/  ",
            "/ /  |  |      ",
        ]
    } else if os_lower.contains("macos") || os_lower.contains("darwin") {
        vec![
            "       .:'     ",
            "    __ :'__    ",
            " .'`  `-'  ``. ",
            ":          .-' ",
            ":         :    ",
            " :         `-; ",
        ]
    } else {
        vec![
            "   ______   ",
            "  /      \\  ",
            " |  ◉  ◉  | ",
            " |    >   | ",
            " |  \\___/ | ",
            "  \\______/  ",
        ]
    }
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

fn format_info(
    hostname: &str,
    os_name: &str,
    kernel: &str,
    uptime: &str,
    shell: &str,
    cpu: &str,
    memory: &str,
    gpus: &[String],
) -> Vec<String> {
    let mut info = Vec::with_capacity(10);

    info.push(format!(
        "{}{}{}",
        colorize(&format!("{}", hostname), BOLD),
        colorize("@", RESET),
        colorize("rustfetch", BOLD)
    ));
    info.push("─".repeat(hostname.len() + 10));
    info.push(format!("{}: {}", colorize("OS", CYAN), os_name));
    info.push(format!("{}: {}", colorize("Kernel", CYAN), kernel));
    info.push(format!("{}: {}", colorize("Uptime", CYAN), uptime));
    info.push(format!("{}: {}", colorize("Shell", CYAN), shell));
    info.push(format!("{}: {}", colorize("CPU", GREEN), cpu));
    info.push(format!("{}: {}", colorize("Memory", YELLOW), memory));

    for (i, gpu) in gpus.iter().enumerate() {
        if i == 0 {
            info.push(format!("{}: {}", colorize("GPU", MAGENTA), gpu));
        } else {
            info.push(format!("    {}", gpu));
        }
    }

    info
}

fn display_side_by_side(logo: &[String], info: &[String]) {
    let logo_width = logo.iter().map(|s| s.len()).max().unwrap_or(0);
    let max_lines = logo.len().max(info.len());

    for i in 0..max_lines {
        let logo_line = if i < logo.len() {
            format!("{:width$}", colorize(&logo[i], BLUE), width = logo_width + 10)
        } else {
            " ".repeat(logo_width + 4)
        };

        let info_line = if i < info.len() { &info[i] } else { "" };

        println!("{}  {}", logo_line, info_line);
    }
}
