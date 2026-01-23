use std::fs;
use std::process::Command;
use std::thread;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};

// ============================================================================
// ================================ CONFIG ====================================
// ============================================================================

// Display Settings
const PROGRESSIVE_DISPLAY: bool = true;     // Show info as it loads (fastfetch style)
const USE_COLOR_OUTPUT: bool = true;        // Enable colored output
const ENABLE_GPU_DETECTION: bool = true;    // Detect GPUs (can be slow)

// Info to Display (set to false to hide)
const SHOW_OS: bool = true;
const SHOW_KERNEL: bool = true;
const SHOW_UPTIME: bool = true;
const SHOW_SHELL: bool = true;
const SHOW_DE: bool = true;                 // Desktop Environment
const SHOW_WM: bool = true;                 // Window Manager
const SHOW_TERMINAL: bool = true;
const SHOW_CPU: bool = true;
const SHOW_MEMORY: bool = true;
const SHOW_GPU: bool = true;
const SHOW_DISK: bool = true;
const SHOW_LOCALE: bool = false;

// ============================================================================
// ============================= END CONFIG ===================================
// ============================================================================

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const MAGENTA: &str = "\x1b[95m";

struct SystemInfo {
    user: Option<String>,
    hostname: Option<String>,
    os_name: Option<String>,
    kernel: Option<String>,
    uptime: Option<String>,
    shell: Option<String>,
    de: Option<String>,
    wm: Option<String>,
    terminal: Option<String>,
    cpu: Option<String>,
    memory: Option<String>,
    gpus: Option<Vec<String>>,
    disk: Option<String>,
    locale: Option<String>,
}

impl SystemInfo {
    fn new() -> Self {
        SystemInfo {
            user: None,
            hostname: None,
            os_name: None,
            kernel: None,
            uptime: None,
            shell: None,
            de: None,
            wm: None,
            terminal: None,
            cpu: None,
            memory: None,
            gpus: None,
            disk: None,
            locale: None,
        }
    }
}

fn main() {
    let info = Arc::new(Mutex::new(SystemInfo::new()));
    let logo_lines = Arc::new(Mutex::new(Vec::new()));

    if PROGRESSIVE_DISPLAY {
        let info_clone = Arc::clone(&info);
        let logo_clone = Arc::clone(&logo_lines);
        
        let display_handle = thread::spawn(move || {
            progressive_display(info_clone, logo_clone);
        });

        // Launch all data gathering threads
        let mut handles = vec![
            {
                let info = Arc::clone(&info);
                thread::spawn(move || {
                    let user = get_user();
                    let hostname = get_hostname();
                    let os_name = get_os_name();
                    let mut guard = info.lock().unwrap();
                    guard.user = Some(user);
                    guard.hostname = Some(hostname);
                    guard.os_name = Some(os_name.clone());
                    drop(guard);
                    
                    let mut logo_guard = logo_lines.lock().unwrap();
                    *logo_guard = get_os_logo(&os_name);
                })
            },
        ];

        if SHOW_KERNEL {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().kernel = Some(get_kernel());
            }));
        }

        if SHOW_UPTIME {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().uptime = Some(get_uptime());
            }));
        }

        if SHOW_SHELL || SHOW_DE || SHOW_WM || SHOW_TERMINAL {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                let shell = if SHOW_SHELL { Some(get_shell()) } else { None };
                let de = if SHOW_DE { Some(get_desktop_environment()) } else { None };
                let wm = if SHOW_WM { Some(get_window_manager()) } else { None };
                let terminal = if SHOW_TERMINAL { Some(get_terminal()) } else { None };
                let mut guard = info.lock().unwrap();
                guard.shell = shell;
                guard.de = de;
                guard.wm = wm;
                guard.terminal = terminal;
            }));
        }

        if SHOW_CPU {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().cpu = Some(get_cpu());
            }));
        }

        if SHOW_MEMORY {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().memory = Some(get_memory());
            }));
        }

        if SHOW_GPU && ENABLE_GPU_DETECTION {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().gpus = Some(get_all_gpus());
            }));
        }

        if SHOW_DISK {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().disk = Some(get_disk_usage());
            }));
        }

        if SHOW_LOCALE {
            let info = Arc::clone(&info);
            handles.push(thread::spawn(move || {
                info.lock().unwrap().locale = Some(get_locale());
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }

        let _ = display_handle.join();
    } else {
        // Fast mode - no progressive display
        let user = get_user();
        let hostname = get_hostname();
        let os_name = get_os_name();
        let kernel = if SHOW_KERNEL { get_kernel() } else { String::new() };
        let uptime = if SHOW_UPTIME { get_uptime() } else { String::new() };
        let shell = if SHOW_SHELL { get_shell() } else { String::new() };
        let de = if SHOW_DE { get_desktop_environment() } else { String::new() };
        let wm = if SHOW_WM { get_window_manager() } else { String::new() };
        let terminal = if SHOW_TERMINAL { get_terminal() } else { String::new() };
        let cpu = if SHOW_CPU { get_cpu() } else { String::new() };
        let memory = if SHOW_MEMORY { get_memory() } else { String::new() };
        let gpus = if SHOW_GPU && ENABLE_GPU_DETECTION { get_all_gpus() } else { vec![] };
        let disk = if SHOW_DISK { get_disk_usage() } else { String::new() };
        let locale = if SHOW_LOCALE { get_locale() } else { String::new() };

        let logo = get_os_logo(&os_name);
        let info_lines = format_info_full(&user, &hostname, &os_name, &kernel, &uptime, &shell, 
            &de, &wm, &terminal, &cpu, &memory, &gpus, &disk, &locale);

        display_side_by_side(&logo, &info_lines);
    }
}

fn progressive_display(info: Arc<Mutex<SystemInfo>>, logo_lines: Arc<Mutex<Vec<String>>>) {
    let mut last_line_count = 0;
    let required_fields = 3; // user, hostname, os_name are always required
    
    loop {
        thread::sleep(std::time::Duration::from_millis(30));
        
        let info_guard = info.lock().unwrap();
        let logo_guard = logo_lines.lock().unwrap();
        
        let info_lines = format_info_progressive(&info_guard);
        
        if last_line_count > 0 {
            print!("\x1b[{}A\x1b[J", last_line_count);
        }
        
        let line_count = display_side_by_side(&logo_guard, &info_lines);
        last_line_count = line_count;
        io::stdout().flush().unwrap();
        
        // Check if all enabled fields are loaded
        let mut all_loaded = info_guard.user.is_some() && 
                             info_guard.hostname.is_some() && 
                             info_guard.os_name.is_some();
        
        if SHOW_KERNEL { all_loaded &= info_guard.kernel.is_some(); }
        if SHOW_UPTIME { all_loaded &= info_guard.uptime.is_some(); }
        if SHOW_SHELL { all_loaded &= info_guard.shell.is_some(); }
        if SHOW_DE { all_loaded &= info_guard.de.is_some(); }
        if SHOW_WM { all_loaded &= info_guard.wm.is_some(); }
        if SHOW_TERMINAL { all_loaded &= info_guard.terminal.is_some(); }
        if SHOW_CPU { all_loaded &= info_guard.cpu.is_some(); }
        if SHOW_MEMORY { all_loaded &= info_guard.memory.is_some(); }
        if SHOW_DISK { all_loaded &= info_guard.disk.is_some(); }
        if SHOW_LOCALE { all_loaded &= info_guard.locale.is_some(); }
        if SHOW_GPU && ENABLE_GPU_DETECTION { all_loaded &= info_guard.gpus.is_some(); }
        
        if all_loaded {
            break;
        }
    }
}

fn format_info_progressive(info: &SystemInfo) -> Vec<String> {
    let mut lines = Vec::new();
    
    if let (Some(user), Some(hostname)) = (&info.user, &info.hostname) {
        lines.push(format!(
            "{}{}{}",
            colorize(&format!("{}", user), BOLD),
            colorize("@", RESET),
            colorize(&hostname, BOLD)
        ));
        lines.push("─".repeat(user.len() + hostname.len() + 1));
    }
    
    if SHOW_OS {
        if let Some(os_name) = &info.os_name {
            lines.push(format!("{}: {}", colorize("OS", CYAN), os_name));
        }
    }
    
    if SHOW_KERNEL {
        if let Some(kernel) = &info.kernel {
            lines.push(format!("{}: {}", colorize("Kernel", CYAN), kernel));
        }
    }
    
    if SHOW_UPTIME {
        if let Some(uptime) = &info.uptime {
            lines.push(format!("{}: {}", colorize("Uptime", CYAN), uptime));
        }
    }
    
    if SHOW_SHELL {
        if let Some(shell) = &info.shell {
            lines.push(format!("{}: {}", colorize("Shell", CYAN), shell));
        }
    }
    
    if SHOW_DE {
        if let Some(de) = &info.de {
            if de != "Unknown" {
                lines.push(format!("{}: {}", colorize("DE", CYAN), de));
            }
        }
    }
    
    if SHOW_WM {
        if let Some(wm) = &info.wm {
            if wm != "Unknown" {
                lines.push(format!("{}: {}", colorize("WM", CYAN), wm));
            }
        }
    }
    
    if SHOW_TERMINAL {
        if let Some(terminal) = &info.terminal {
            if terminal != "Unknown" {
                lines.push(format!("{}: {}", colorize("Terminal", CYAN), terminal));
            }
        }
    }
    
    if SHOW_CPU {
        if let Some(cpu) = &info.cpu {
            lines.push(format!("{}: {}", colorize("CPU", GREEN), cpu));
        }
    }
    
    if SHOW_MEMORY {
        if let Some(memory) = &info.memory {
            lines.push(format!("{}: {}", colorize("Memory", YELLOW), memory));
        }
    }
    
    if SHOW_GPU {
        if let Some(gpus) = &info.gpus {
            for (i, gpu) in gpus.iter().enumerate() {
                if i == 0 {
                    lines.push(format!("{}: {}", colorize("GPU", MAGENTA), gpu));
                } else {
                    lines.push(format!("    {}", gpu));
                }
            }
        }
    }
    
    if SHOW_DISK {
        if let Some(disk) = &info.disk {
            lines.push(format!("{}: {}", colorize("Disk", BLUE), disk));
        }
    }
    
    if SHOW_LOCALE {
        if let Some(locale) = &info.locale {
            if locale != "Unknown" {
                lines.push(format!("{}: {}", colorize("Locale", CYAN), locale));
            }
        }
    }
    
    lines
}

fn format_info_full(user: &str, hostname: &str, os_name: &str, kernel: &str, uptime: &str, 
    shell: &str, de: &str, wm: &str, terminal: &str, cpu: &str, memory: &str, 
    gpus: &[String], disk: &str, locale: &str) -> Vec<String> {
    let mut lines = Vec::new();
    
    lines.push(format!(
        "{}{}{}",
        colorize(user, BOLD),
        colorize("@", RESET),
        colorize(hostname, BOLD)
    ));
    lines.push("─".repeat(user.len() + hostname.len() + 1));
    
    if SHOW_OS { lines.push(format!("{}: {}", colorize("OS", CYAN), os_name)); }
    if SHOW_KERNEL { lines.push(format!("{}: {}", colorize("Kernel", CYAN), kernel)); }
    if SHOW_UPTIME { lines.push(format!("{}: {}", colorize("Uptime", CYAN), uptime)); }
    if SHOW_SHELL { lines.push(format!("{}: {}", colorize("Shell", CYAN), shell)); }
    if SHOW_DE && de != "Unknown" { lines.push(format!("{}: {}", colorize("DE", CYAN), de)); }
    if SHOW_WM && wm != "Unknown" { lines.push(format!("{}: {}", colorize("WM", CYAN), wm)); }
    if SHOW_TERMINAL && terminal != "Unknown" { lines.push(format!("{}: {}", colorize("Terminal", CYAN), terminal)); }
    if SHOW_CPU { lines.push(format!("{}: {}", colorize("CPU", GREEN), cpu)); }
    if SHOW_MEMORY { lines.push(format!("{}: {}", colorize("Memory", YELLOW), memory)); }
    
    if SHOW_GPU {
        for (i, gpu) in gpus.iter().enumerate() {
            if i == 0 {
                lines.push(format!("{}: {}", colorize("GPU", MAGENTA), gpu));
            } else {
                lines.push(format!("    {}", gpu));
            }
        }
    }
    
    if SHOW_DISK { lines.push(format!("{}: {}", colorize("Disk", BLUE), disk)); }
    if SHOW_LOCALE && locale != "Unknown" { lines.push(format!("{}: {}", colorize("Locale", CYAN), locale)); }
    
    lines
}

#[inline(always)]
fn colorize(text: &str, color: &str) -> String {
    if USE_COLOR_OUTPUT {
        format!("{}{}{}", color, text, RESET)
    } else {
        text.to_string()
    }
}

#[inline(always)]
fn get_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
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
    "Unknown".to_string()
}

#[inline(always)]
fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_desktop_environment() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| {
            if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
                "GNOME".to_string()
            } else if std::env::var("KDE_FULL_SESSION").is_ok() {
                "KDE".to_string()
            } else if std::env::var("MATE_DESKTOP_SESSION_ID").is_ok() {
                "MATE".to_string()
            } else {
                "Unknown".to_string()
            }
        })
}

fn get_window_manager() -> String {
    // Fast WM detection via processes
    let wms = ["hyprland", "sway", "i3", "bspwm", "awesome", "dwm", "openbox", "xmonad", "qtile"];
    for wm in &wms {
        if let Ok(output) = Command::new("pgrep").arg("-x").arg(wm).output() {
            if output.status.success() && !output.stdout.is_empty() {
                return wm.to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn get_terminal() -> String {
    std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("TERMINAL"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_cpu() -> String {
    if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
        let mut model = None;
        let mut cores = 0;

        for line in contents.lines() {
            if model.is_none() && line.starts_with("model name") {
                model = line.split(':').nth(1).map(|s| {
                    s.trim()
                        .replace("(R)", "")
                        .replace("(TM)", "")
                        .replace("  ", " ")
                        .trim()
                        .to_string()
                });
            } else if line.starts_with("processor") {
                cores += 1;
            }
            if model.is_some() && cores > 0 && line.is_empty() {
                break;
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
    "Unknown".to_string()
}

fn get_all_gpus() -> Vec<String> {
    let mut gpus = Vec::new();

    // NVIDIA via nvidia-smi
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

    // lspci for all GPUs
    if let Ok(output) = Command::new("lspci").output() {
        if output.status.success() {
            if let Ok(lspci_output) = String::from_utf8(output.stdout) {
                for line in lspci_output.lines() {
                    if line.contains("VGA compatible controller") || line.contains("3D controller") {
                        if let Some(gpu_info) = line.split(':').nth(2) {
                            let gpu_name = gpu_info.trim();
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

    if gpus.is_empty() {
        gpus.push("No GPU detected".to_string());
    }

    gpus
}

fn get_disk_usage() -> String {
    if let Ok(output) = Command::new("df").args(&["-h", "/"]).output() {
        if let Ok(df_output) = String::from_utf8(output.stdout) {
            if let Some(line) = df_output.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    return format!("{} / {} ({})", parts[2], parts[1], parts[4]);
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_locale() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "Unknown".to_string())
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

fn display_side_by_side(logo: &[String], info: &[String]) -> usize {
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
    
    max_lines
}
