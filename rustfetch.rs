use std::fs;
use std::process::Command;

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
    let hostname = get_hostname();
    let os_name = get_os_name();
    let kernel = get_kernel();
    let uptime = get_uptime();
    let shell = get_shell();
    let cpu = get_cpu();
    let memory = get_memory();
    let gpus = if ENABLE_GPU_DETECTION {
        get_all_gpus()
    } else {
        vec!["GPU detection disabled".to_string()]
    };

    let logo = get_os_logo(&os_name);
    let info = format_info(&hostname, &os_name, &kernel, &uptime, &shell, &cpu, &memory, &gpus);

    display_side_by_side(&logo, &info);
}

fn colorize(text: &str, color: &str) -> String {
    if USE_COLOR_OUTPUT {
        format!("{}{}{}", color, text, RESET)
    } else {
        text.to_string()
    }
}

fn get_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| {
            Command::new("hostname")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .trim()
        .to_string()
}

fn get_os_name() -> String {
    if let Ok(contents) = fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .split('=')
                    .nth(1)
                    .unwrap_or("Unknown Linux")
                    .trim_matches('"')
                    .to_string();
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
    if let Ok(contents) = fs::read_to_string("/proc/uptime") {
        if let Some(uptime_str) = contents.split_whitespace().next() {
            if let Ok(uptime_secs) = uptime_str.parse::<f64>() {
                let days = (uptime_secs / 86400.0) as u64;
                let hours = ((uptime_secs % 86400.0) / 3600.0) as u64;
                let mins = ((uptime_secs % 3600.0) / 60.0) as u64;

                if days > 0 {
                    return format!("{}d {}h {}m", days, hours, mins);
                } else if hours > 0 {
                    return format!("{}h {}m", hours, mins);
                } else {
                    return format!("{}m", mins);
                }
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

fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.split('/').last().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_cpu() -> String {
    if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        let mut model = String::new();
        let mut cores = 0;

        for line in contents.lines() {
            if line.starts_with("model name") {
                model = line
                    .split(':')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if line.starts_with("processor") {
                cores += 1;
            }
        }

        if !model.is_empty() {
            return format!("{} ({} cores)", model, cores);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
        {
            if let Ok(cpu_name) = String::from_utf8(output.stdout) {
                if let Ok(cores_output) = Command::new("sysctl")
                    .arg("-n")
                    .arg("hw.ncpu")
                    .output()
                {
                    if let Ok(cores_str) = String::from_utf8(cores_output.stdout) {
                        return format!("{} ({} cores)", cpu_name.trim(), cores_str.trim());
                    }
                }
                return cpu_name.trim().to_string();
            }
        }
    }

    "Unknown CPU".to_string()
}

fn get_memory() -> String {
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        let mut total = 0;
        let mut available = 0;

        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }

        if total > 0 {
            let used = total - available;
            return format!(
                "{:.1} GiB / {:.1} GiB",
                used as f64 / 1024.0 / 1024.0,
                total as f64 / 1024.0 / 1024.0
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
        {
            if let Ok(mem_str) = String::from_utf8(output.stdout) {
                if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                    return format!("{:.1} GiB", mem_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
                }
            }
        }
    }

    "Unknown".to_string()
}

fn get_all_gpus() -> Vec<String> {
    let mut gpus = Vec::new();

    // Method 1: NVIDIA GPUs via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .arg("--query-gpu=gpu_name")
        .arg("--format=csv,noheader")
        .output()
    {
        if output.status.success() {
            if let Ok(nvidia_output) = String::from_utf8(output.stdout) {
                for line in nvidia_output.lines() {
                    let gpu_name = line.trim();
                    if !gpu_name.is_empty() {
                        gpus.push(format!("NVIDIA {}", gpu_name));
                    }
                }
            }
        }
    }

    // Method 2: lspci for all GPUs (AMD, Intel, and NVIDIA as fallback)
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            if let Ok(lspci_output) = String::from_utf8(output.stdout) {
                for line in lspci_output.lines() {
                    if line.contains("VGA compatible controller") || line.contains("3D controller") {
                        if let Some(gpu_info) = line.split(':').nth(2) {
                            let gpu_name = gpu_info.trim();
                            // Avoid duplicates if nvidia-smi already found NVIDIA GPUs
                            if !gpus.iter().any(|g| gpu_name.contains(&g.replace("NVIDIA ", ""))) {
                                gpus.push(gpu_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 3: /sys/class/drm for additional GPU info on Linux
    if gpus.is_empty() {
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("card") && !name_str.contains('-') {
                        let vendor_path = path.join("device/vendor");
                        let device_path = path.join("device/device");
                        
                        if let (Ok(vendor), Ok(device)) = (
                            fs::read_to_string(&vendor_path),
                            fs::read_to_string(&device_path)
                        ) {
                            let vendor_id = vendor.trim();
                            let device_id = device.trim();
                            gpus.push(format!("GPU (Vendor: {}, Device: {})", vendor_id, device_id));
                        }
                    }
                }
            }
        }
    }

    // Method 4: macOS GPU detection
    #[cfg(target_os = "macos")]
    {
        if gpus.is_empty() {
            if let Ok(output) = Command::new("system_profiler")
                .arg("SPDisplaysDataType")
                .output()
            {
                if let Ok(gpu_info) = String::from_utf8(output.stdout) {
                    for line in gpu_info.lines() {
                        if line.contains("Chipset Model:") {
                            if let Some(model) = line.split(':').nth(1) {
                                gpus.push(model.trim().to_string());
                            }
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
            "      /\\      ".to_string(),
            "     /  \\     ".to_string(),
            "    /\\   \\    ".to_string(),
            "   /  \\   \\   ".to_string(),
            "  /    \\   \\  ".to_string(),
            " /______\\___\\ ".to_string(),
        ]
    } else if os_lower.contains("ubuntu") {
        vec![
            "         _     ".to_string(),
            "     ---(_)    ".to_string(),
            " _/  ---  \\    ".to_string(),
            "(_) |   |      ".to_string(),
            "  \\  --- _/    ".to_string(),
            "     ---(_)    ".to_string(),
        ]
    } else if os_lower.contains("debian") {
        vec![
            "  _____  ".to_string(),
            " /  __ \\ ".to_string(),
            "|  /    |".to_string(),
            "|  \\___- ".to_string(),
            " -_      ".to_string(),
            "   --_   ".to_string(),
        ]
    } else if os_lower.contains("fedora") {
        vec![
            "      _____    ".to_string(),
            "     /   __)\\  ".to_string(),
            "     |  /  \\ \\ ".to_string(),
            "  ___|  |__/ / ".to_string(),
            " / (_    _)_/  ".to_string(),
            "/ /  |  |      ".to_string(),
        ]
    } else if os_lower.contains("macos") || os_lower.contains("darwin") {
        vec![
            "       .:'     ".to_string(),
            "    __ :'__    ".to_string(),
            " .'`  `-'  ``. ".to_string(),
            ":          .-' ".to_string(),
            ":         :    ".to_string(),
            " :         `-; ".to_string(),
        ]
    } else {
        vec![
            "   ______   ".to_string(),
            "  /      \\  ".to_string(),
            " |  ◉  ◉  | ".to_string(),
            " |    >   | ".to_string(),
            " |  \\___/ | ".to_string(),
            "  \\______/  ".to_string(),
        ]
    }
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
    let mut info = Vec::new();

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
