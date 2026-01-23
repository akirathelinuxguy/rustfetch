use std::fs;
use std::io::{self, BufRead};
use std::process::Command;

// ==== CONFIGURATION ====
const ENABLE_GPU_DETECTION: bool = true;
const USE_COLOR_OUTPUT: bool = true;
const SHOW_SHELL: bool = true;
const SHOW_KERNEL: bool = true;

// ANSI color codes
mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD_BLUE: &str = "\x1b[1;34m";
    pub const BOLD_GREEN: &str = "\x1b[1;32m";
    pub const BOLD_RED: &str = "\x1b[1;31m";
    pub const BOLD_YELLOW: &str = "\x1b[1;33m";
}

fn color_text(text: &str, color_code: &str) -> String {
    if USE_COLOR_OUTPUT {
        format!("{}{}{}", color_code, text, colors::RESET)
    } else {
        text.to_string()
    }
}

fn get_os_icon(os: &str) -> &str {
    match os.to_lowercase().as_str() {
        s if s.contains("macos") || s.contains("mac os") => "",
        s if s.contains("ubuntu") => "♕",
        s if s.contains("debian") => "♦",
        s if s.contains("fedora") => "🦋",
        s if s.contains("arch") => "🌀",
        s if s.contains("pop") => "🚀",
        s if s.contains("cachy") => "🌰",
        s if s.contains("pika") => "🐭",
        s if s.contains("elementary") => "🍎",
        s if s.contains("manjaro") => "🌄",
        s if s.contains("kali") => "🔪",
        s if s.contains("suse") => "🦎",
        s if s.contains("centos") => "🩸",
        s if s.contains("rocky") => "🪨",
        s if s.contains("alpine") => "🏔️",
        s if s.contains("mint") => "🌿",
        s if s.contains("linux") => "🐧",
        _ => "",
    }
}

fn main() {
    println!("\n{}\n", color_text("=== rustfetch ===", colors::BOLD_BLUE));

    display_info("Hostname", &get_hostname());
    
    let os_info = get_os_info();
    let os_icon = get_os_icon(&os_info);
    println!(
        "{}: {} {}",
        color_text("OS", colors::BOLD_GREEN),
        os_icon,
        os_info
    );

    if SHOW_KERNEL {
        display_info("Kernel", &get_kernel_version());
    }

    if SHOW_SHELL {
        display_info("Shell", &get_shell());
    }

    let (cpu_model, cores, threads) = get_cpu_info();
    display_info("CPU", &cpu_model);
    println!(
        "{}: {} cores / {} threads",
        color_text("Cores/Threads", colors::BOLD_GREEN),
        cores,
        threads
    );

    display_info("Memory", &get_memory_info());
    display_info("Uptime", &get_uptime());

    if ENABLE_GPU_DETECTION {
        display_info("GPU", &get_gpu_info());
    }

    println!();
}

fn display_info(label: &str, value: &str) {
    println!("{}: {}", color_text(label, colors::BOLD_GREEN), value);
}

fn get_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .or_else(|_| {
            Command::new("hostname")
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        })
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_os_info() -> String {
    #[cfg(target_os = "macos")]
    {
        Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|out| {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                Some(format!("macOS {}", version))
            })
            .unwrap_or_else(|| "macOS".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        fs::File::open("/etc/os-release")
            .ok()
            .and_then(|file| {
                io::BufReader::new(file)
                    .lines()
                    .find_map(|line| {
                        line.ok().and_then(|l| {
                            if l.starts_with("PRETTY_NAME=") {
                                Some(l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
                            } else {
                                None
                            }
                        })
                    })
            })
            .unwrap_or_else(|| "Linux".to_string())
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        Command::new("uname")
            .args(&["-sr"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_else(|| "BSD".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    {
        "Unknown OS".to_string()
    }
}

fn get_kernel_version() -> String {
    Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.split('/').last().map(String::from))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_cpu_info() -> (String, usize, usize) {
    #[cfg(target_os = "macos")]
    {
        let model = Command::new("sysctl")
            .args(&["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cores = Command::new("sysctl")
            .args(&["-n", "hw.physicalcpu"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
            .unwrap_or(1);

        let threads = Command::new("sysctl")
            .args(&["-n", "hw.logicalcpu"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
            .unwrap_or(cores);

        (model, cores, threads)
    }

    #[cfg(target_os = "linux")]
    {
        let mut model = "Unknown CPU".to_string();
        let mut physical_cores = 0;
        let mut thread_count = 0;

        if let Ok(file) = fs::File::open("/proc/cpuinfo") {
            let reader = io::BufReader::new(file);
            let mut core_ids = std::collections::HashSet::new();

            for line in reader.lines().filter_map(Result::ok) {
                if line.starts_with("model name") && model == "Unknown CPU" {
                    model = line.split(':').nth(1)
                        .map(|s| s.trim().to_string())
                        .unwrap_or(model);
                } else if line.starts_with("processor") {
                    thread_count += 1;
                } else if line.starts_with("core id") {
                    if let Some(id) = line.split(':').nth(1).and_then(|s| s.trim().parse::<usize>().ok()) {
                        core_ids.insert(id);
                    }
                }
            }

            physical_cores = if core_ids.is_empty() { thread_count } else { core_ids.len() };
        }

        (model, physical_cores.max(1), thread_count.max(1))
    }

    #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        let model = Command::new("sysctl")
            .args(&["-n", "hw.model"])
            .output()
            .ok()
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cores = Command::new("sysctl")
            .args(&["-n", "hw.ncpu"])
            .output()
            .ok()
            .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
            .unwrap_or(1);

        (model, cores, cores)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    {
        ("Unknown CPU".to_string(), 1, 1)
    }
}

fn get_memory_info() -> String {
    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .map(|bytes| format!("{:.2} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0))
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|contents| {
                contents.lines()
                    .find(|line| line.starts_with("MemTotal:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|kb| kb.parse::<u64>().ok())
                            .map(|kb| format!("{:.2} GB", kb as f64 / 1024.0 / 1024.0))
                    })
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    {
        "Unknown".to_string()
    }
}

fn get_uptime() -> String {
    #[cfg(target_os = "linux")]
    {
        fs::read_to_string("/proc/uptime")
            .ok()
            .and_then(|contents| {
                contents.split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|secs| format_duration(secs as u64))
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("sysctl")
            .args(&["-n", "kern.boottime"])
            .output()
            .ok()
            .and_then(|out| {
                let output = String::from_utf8_lossy(&out.stdout);
                output.split_whitespace()
                    .nth(3)
                    .and_then(|s| s.trim_matches(',').parse::<u64>().ok())
                    .and_then(|boot_time| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|now| format_duration(now.as_secs() - boot_time))
                    })
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "Unknown".to_string()
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn get_gpu_info() -> String {
    #[cfg(target_os = "macos")]
    {
        Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
            .ok()
            .and_then(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .find(|line| line.trim().starts_with("Chipset Model:"))
                    .map(|line| {
                        line.trim()
                            .trim_start_matches("Chipset Model:")
                            .trim()
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "Not detected".to_string())
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
    {
        Command::new("lspci")
            .output()
            .ok()
            .and_then(|out| {
                String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .find(|line| {
                        let lower = line.to_lowercase();
                        lower.contains("vga") || lower.contains("3d controller")
                    })
                    .map(|line| {
                        line.split(':')
                            .skip(2)
                            .collect::<Vec<_>>()
                            .join(":")
                            .trim()
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "Not detected".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
    {
        "Not detected".to_string()
    }
}
