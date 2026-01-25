use std::{
    fs, 
    process::Command, 
    thread, 
    sync::{Arc, Mutex}, 
    io::{self, Write}, 
    collections::{HashMap, HashSet},
    time::Duration,
};

// ╔══════════════════════════════════════════════════════════════════════════╗
// ║                            CONFIGURATION                                  ║
// ╚══════════════════════════════════════════════════════════════════════════╝

const PROGRESSIVE_DISPLAY: bool = true;
const USE_COLOR: bool = true;
const CACHE_ENABLED: bool = true;
const CACHE_FILE: &str = "/tmp/rustfetch_cache";

// Display toggles
const SHOW_USER_HOST: bool = true;
const SHOW_OS: bool = true;
const SHOW_KERNEL: bool = true;
const SHOW_UPTIME: bool = true;
const SHOW_PACKAGES: bool = true;
const SHOW_SHELL: bool = true;
const SHOW_DE: bool = true;
const SHOW_WM: bool = true;
const SHOW_WM_THEME: bool = true;
const SHOW_TERMINAL: bool = true;
const SHOW_CPU: bool = true;
const SHOW_GPU: bool = true;
const SHOW_MEMORY: bool = true;
const SHOW_SWAP: bool = true;
const SHOW_DISK: bool = true;
const SHOW_DISK_DETAILED: bool = true;
const SHOW_LOCALE: bool = false;
const SHOW_LOCAL_IP: bool = true;
const SHOW_PUBLIC_IP: bool = false;
const SHOW_BATTERY: bool = true;
const SHOW_COLORS: bool = true;

// Color constants
const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_CYAN: &str = "\x1b[96m";
const C_GREEN: &str = "\x1b[92m";
const C_YELLOW: &str = "\x1b[93m";
const C_BLUE: &str = "\x1b[94m";
const C_MAGENTA: &str = "\x1b[95m";
const C_RED: &str = "\x1b[91m";

// ╔══════════════════════════════════════════════════════════════════════════╗
// ║                           END CONFIGURATION                               ║
// ╚══════════════════════════════════════════════════════════════════════════╝

#[derive(Default)]
struct Info {
    user: Option<String>,
    hostname: Option<String>,
    os: Option<String>,
    kernel: Option<String>,
    uptime: Option<String>,
    packages: Option<String>,
    shell: Option<String>,
    de: Option<String>,
    wm: Option<String>,
    wm_theme: Option<String>,
    terminal: Option<String>,
    cpu: Option<String>,
    gpu: Option<Vec<String>>,
    memory: Option<String>,
    swap: Option<String>,
    disk: Option<String>,
    disk_detailed: Option<Vec<String>>,
    locale: Option<String>,
    local_ip: Option<String>,
    public_ip: Option<String>,
    battery: Option<String>,
}

struct Cache {
    data: HashMap<String, String>,
}

impl Cache {
    fn load() -> Self {
        if !CACHE_ENABLED {
            return Cache {
                data: HashMap::new(),
            };
        }
        
        let data = fs::read_to_string(CACHE_FILE)
            .ok()
            .map(|content| {
                content
                    .lines()
                    .filter_map(|line| {
                        line.split_once('=')
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        Cache { data }
    }
    
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    
    fn set(&mut self, key: &str, value: String) {
        self.data.insert(key.to_string(), value);
    }
    
    fn save(&self) {
        if !CACHE_ENABLED {
            return;
        }
        
        let content: String = self
            .data
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        
        let _ = fs::write(CACHE_FILE, content);
    }
}

fn main() {
    let cache = Arc::new(Mutex::new(Cache::load()));
    let info = Arc::new(Mutex::new(Info::default()));
    let logo = Arc::new(Mutex::new(Vec::new()));

    if PROGRESSIVE_DISPLAY {
        progressive_display(Arc::clone(&cache), Arc::clone(&info), Arc::clone(&logo));
    } else {
        static_display(Arc::clone(&cache), Arc::clone(&info), Arc::clone(&logo));
    }
}

fn progressive_display(
    cache: Arc<Mutex<Cache>>,
    info: Arc<Mutex<Info>>,
    logo: Arc<Mutex<Vec<String>>>,
) {
    let info_clone = Arc::clone(&info);
    let logo_clone = Arc::clone(&logo);
    
    let display_thread = thread::spawn(move || {
        let mut last_lines = 0;
        loop {
            thread::sleep(Duration::from_millis(20));
            
            let info_guard = info_clone.lock().unwrap();
            let logo_guard = logo_clone.lock().unwrap();
            
            if last_lines > 0 {
                print!("\x1b[{}A\x1b[J", last_lines);
            }
            
            last_lines = display_info(&info_guard, &logo_guard);
            io::stdout().flush().unwrap();
            
            if check_loaded(&info_guard) {
                break;
            }
        }
    });

    let mut handles = vec![spawn_basic_info(&info, &cache, &logo)];
    
    if SHOW_KERNEL {
        handles.push(spawn_kernel(&info, &cache));
    }
    if SHOW_UPTIME {
        handles.push(spawn_uptime(&info));
    }
    if SHOW_PACKAGES {
        handles.push(spawn_packages(&info, &cache));
    }
    if SHOW_SHELL || SHOW_DE || SHOW_WM || SHOW_WM_THEME || SHOW_TERMINAL {
        handles.push(spawn_shell_environment(&info, &cache));
    }
    if SHOW_CPU {
        handles.push(spawn_cpu(&info, &cache));
    }
    if SHOW_GPU {
        handles.push(spawn_gpu(&info, &cache));
    }
    if SHOW_MEMORY || SHOW_SWAP {
        handles.push(spawn_memory(&info));
    }
    if SHOW_DISK {
        handles.push(spawn_disk(&info));
    }
    if SHOW_DISK_DETAILED {
        handles.push(spawn_disk_detailed(&info));
    }
    if SHOW_LOCALE {
        handles.push(spawn_locale(&info));
    }
    if SHOW_LOCAL_IP {
        handles.push(spawn_local_ip(&info));
    }
    if SHOW_PUBLIC_IP {
        handles.push(spawn_public_ip(&info));
    }
    if SHOW_BATTERY {
        handles.push(spawn_battery(&info));
    }

    for handle in handles {
        let _ = handle.join();
    }
    
    cache.lock().unwrap().save();
    let _ = display_thread.join();
}

fn static_display(
    cache: Arc<Mutex<Cache>>,
    info: Arc<Mutex<Info>>,
    logo: Arc<Mutex<Vec<String>>>,
) {
    let cache_guard = cache.lock().unwrap();
    
    let user = get_user();
    let host = get_cached_static(&cache_guard, "host", get_hostname);
    let os = get_cached_static(&cache_guard, "os", get_os);
    
    *logo.lock().unwrap() = get_logo(&os);
    
    let mut info_guard = info.lock().unwrap();
    info_guard.user = Some(user);
    info_guard.hostname = Some(host);
    info_guard.os = Some(os);
    
    if SHOW_KERNEL {
        info_guard.kernel = Some(get_cached_static(&cache_guard, "kernel", get_kernel));
    }
    if SHOW_UPTIME {
        info_guard.uptime = Some(get_uptime());
    }
    if SHOW_PACKAGES {
        info_guard.packages = Some(get_cached_static(&cache_guard, "pkgs", get_packages));
    }
    if SHOW_SHELL {
        info_guard.shell = Some(get_shell());
    }
    if SHOW_DE {
        info_guard.de = Some(get_de());
    }
    if SHOW_WM {
        info_guard.wm = Some(get_cached_static(&cache_guard, "wm", get_wm));
    }
    if SHOW_WM_THEME {
        info_guard.wm_theme = Some(get_wm_theme());
    }
    if SHOW_TERMINAL {
        info_guard.terminal = Some(get_terminal());
    }
    if SHOW_CPU {
        info_guard.cpu = Some(get_cached_static(&cache_guard, "cpu", get_cpu));
    }
    if SHOW_GPU {
        info_guard.gpu = Some(get_cached_vec_static(&cache_guard, "gpu", get_gpu));
    }
    if SHOW_MEMORY || SHOW_SWAP {
        let (mem, swap) = get_memory_swap();
        if SHOW_MEMORY {
            info_guard.memory = Some(mem);
        }
        if SHOW_SWAP {
            info_guard.swap = Some(swap);
        }
    }
    if SHOW_DISK {
        info_guard.disk = Some(get_disk());
    }
    if SHOW_DISK_DETAILED {
        info_guard.disk_detailed = Some(get_lsblk());
    }
    if SHOW_LOCALE {
        info_guard.locale = Some(get_locale());
    }
    if SHOW_LOCAL_IP {
        info_guard.local_ip = Some(get_local_ip());
    }
    if SHOW_PUBLIC_IP {
        info_guard.public_ip = Some(get_public_ip());
    }
    if SHOW_BATTERY {
        info_guard.battery = Some(get_battery());
    }
    
    drop(info_guard);
    drop(cache_guard);
    
    cache.lock().unwrap().save();
    display_info(&info.lock().unwrap(), &logo.lock().unwrap());
}

// Thread spawning helpers
fn spawn_basic_info(
    info: &Arc<Mutex<Info>>,
    cache: &Arc<Mutex<Cache>>,
    logo: &Arc<Mutex<Vec<String>>>,
) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    let logo = Arc::clone(logo);
    
    thread::spawn(move || {
        let user = get_user();
        let host = get_cached(&cache, "host", get_hostname);
        let os = get_cached(&cache, "os", get_os);
        
        let mut info_guard = info.lock().unwrap();
        info_guard.user = Some(user);
        info_guard.hostname = Some(host);
        info_guard.os = Some(os.clone());
        drop(info_guard);
        
        *logo.lock().unwrap() = get_logo(&os);
    })
}

fn spawn_kernel(info: &Arc<Mutex<Info>>, cache: &Arc<Mutex<Cache>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    thread::spawn(move || {
        info.lock().unwrap().kernel = Some(get_cached(&cache, "kernel", get_kernel));
    })
}

fn spawn_uptime(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().uptime = Some(get_uptime());
    })
}

fn spawn_packages(info: &Arc<Mutex<Info>>, cache: &Arc<Mutex<Cache>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    thread::spawn(move || {
        info.lock().unwrap().packages = Some(get_cached(&cache, "pkgs", get_packages));
    })
}

fn spawn_shell_environment(
    info: &Arc<Mutex<Info>>,
    cache: &Arc<Mutex<Cache>>,
) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    thread::spawn(move || {
        let mut info_guard = info.lock().unwrap();
        if SHOW_SHELL {
            info_guard.shell = Some(get_shell());
        }
        if SHOW_DE {
            info_guard.de = Some(get_de());
        }
        if SHOW_WM {
            info_guard.wm = Some(get_cached(&cache, "wm", get_wm));
        }
        if SHOW_WM_THEME {
            info_guard.wm_theme = Some(get_wm_theme());
        }
        if SHOW_TERMINAL {
            info_guard.terminal = Some(get_terminal());
        }
    })
}

fn spawn_cpu(info: &Arc<Mutex<Info>>, cache: &Arc<Mutex<Cache>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    thread::spawn(move || {
        info.lock().unwrap().cpu = Some(get_cached(&cache, "cpu", get_cpu));
    })
}

fn spawn_gpu(info: &Arc<Mutex<Info>>, cache: &Arc<Mutex<Cache>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    let cache = Arc::clone(cache);
    thread::spawn(move || {
        info.lock().unwrap().gpu = Some(get_cached_vec(&cache, "gpu", get_gpu));
    })
}

fn spawn_memory(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        let (mem, swap) = get_memory_swap();
        let mut info_guard = info.lock().unwrap();
        if SHOW_MEMORY {
            info_guard.memory = Some(mem);
        }
        if SHOW_SWAP {
            info_guard.swap = Some(swap);
        }
    })
}

fn spawn_disk(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().disk = Some(get_disk());
    })
}

fn spawn_disk_detailed(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().disk_detailed = Some(get_lsblk());
    })
}

fn spawn_locale(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().locale = Some(get_locale());
    })
}

fn spawn_local_ip(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().local_ip = Some(get_local_ip());
    })
}

fn spawn_public_ip(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().public_ip = Some(get_public_ip());
    })
}

fn spawn_battery(info: &Arc<Mutex<Info>>) -> thread::JoinHandle<()> {
    let info = Arc::clone(info);
    thread::spawn(move || {
        info.lock().unwrap().battery = Some(get_battery());
    })
}

fn check_loaded(info: &Info) -> bool {
    info.user.is_some()
        && info.hostname.is_some()
        && info.os.is_some()
        && (!SHOW_KERNEL || info.kernel.is_some())
        && (!SHOW_UPTIME || info.uptime.is_some())
        && (!SHOW_PACKAGES || info.packages.is_some())
        && (!SHOW_SHELL || info.shell.is_some())
        && (!SHOW_DE || info.de.is_some())
        && (!SHOW_WM || info.wm.is_some())
        && (!SHOW_WM_THEME || info.wm_theme.is_some())
        && (!SHOW_TERMINAL || info.terminal.is_some())
        && (!SHOW_CPU || info.cpu.is_some())
        && (!SHOW_GPU || info.gpu.is_some())
        && (!SHOW_MEMORY || info.memory.is_some())
        && (!SHOW_SWAP || info.swap.is_some())
        && (!SHOW_DISK || info.disk.is_some())
        && (!SHOW_DISK_DETAILED || info.disk_detailed.is_some())
        && (!SHOW_LOCALE || info.locale.is_some())
        && (!SHOW_LOCAL_IP || info.local_ip.is_some())
        && (!SHOW_PUBLIC_IP || info.public_ip.is_some())
        && (!SHOW_BATTERY || info.battery.is_some())
}

fn display_info(info: &Info, logo: &[String]) -> usize {
    let mut lines = Vec::new();
    
    if let (Some(user), Some(host)) = (&info.user, &info.hostname) {
        lines.push(format!(
            "{}{}{}",
            colorize(user, C_BOLD),
            colorize("@", C_RESET),
            colorize(host, C_BOLD)
        ));
        lines.push("─".repeat(user.len() + host.len() + 1));
    }
    
    add_info_line(&mut lines, SHOW_OS, &info.os, "OS", C_CYAN);
    add_info_line(&mut lines, SHOW_KERNEL, &info.kernel, "Kernel", C_CYAN);
    add_info_line(&mut lines, SHOW_UPTIME, &info.uptime, "Uptime", C_CYAN);
    add_info_line(&mut lines, SHOW_PACKAGES, &info.packages, "Packages", C_CYAN);
    add_info_line(&mut lines, SHOW_SHELL, &info.shell, "Shell", C_CYAN);
    add_filtered_line(&mut lines, SHOW_DE, &info.de, "DE", C_CYAN);
    add_filtered_line(&mut lines, SHOW_WM, &info.wm, "WM", C_CYAN);
    add_filtered_line(&mut lines, SHOW_WM_THEME, &info.wm_theme, "Theme", C_CYAN);
    add_filtered_line(&mut lines, SHOW_TERMINAL, &info.terminal, "Terminal", C_CYAN);
    add_info_line(&mut lines, SHOW_CPU, &info.cpu, "CPU", C_GREEN);
    
    if SHOW_GPU {
        if let Some(gpus) = &info.gpu {
            for (idx, gpu) in gpus.iter().enumerate() {
                if idx == 0 {
                    lines.push(format!("{}: {}", colorize("GPU", C_MAGENTA), gpu));
                } else {
                    lines.push(format!("    {}", gpu));
                }
            }
        }
    }
    
    add_info_line(&mut lines, SHOW_MEMORY, &info.memory, "Memory", C_YELLOW);
    
    if SHOW_SWAP {
        if let Some(swap) = &info.swap {
            if swap != "0 B" {
                lines.push(format!("{}: {}", colorize("Swap", C_YELLOW), swap));
            }
        }
    }
    
    add_info_line(&mut lines, SHOW_DISK, &info.disk, "Disk (/)", C_BLUE);
    
    if SHOW_DISK_DETAILED {
        if let Some(disks) = &info.disk_detailed {
            for (idx, disk) in disks.iter().enumerate() {
                if idx == 0 {
                    lines.push(format!("{}: {}", colorize("Disks", C_BLUE), disk));
                } else {
                    lines.push(format!("       {}", disk));
                }
            }
        }
    }
    
    add_filtered_line(&mut lines, SHOW_LOCALE, &info.locale, "Locale", C_CYAN);
    add_filtered_line(&mut lines, SHOW_LOCAL_IP, &info.local_ip, "Local IP", C_GREEN);
    add_filtered_line(&mut lines, SHOW_PUBLIC_IP, &info.public_ip, "Public IP", C_GREEN);
    add_filtered_line(&mut lines, SHOW_BATTERY, &info.battery, "Battery", C_RED);
    
    if SHOW_COLORS {
        lines.push(String::new());
        lines.push(format!(
            "{}███{}███{}███{}███{}███{}███{}███{}███{}",
            "\x1b[40m", "\x1b[41m", "\x1b[42m", "\x1b[43m",
            "\x1b[44m", "\x1b[45m", "\x1b[46m", "\x1b[47m", C_RESET
        ));
    }
    
    let logo_width = logo.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let max_lines = logo.len().max(lines.len());
    
    for idx in 0..max_lines {
        let logo_line = if idx < logo.len() {
            format!("{:width$}", colorize(&logo[idx], C_BLUE), width = logo_width + 10)
        } else {
            " ".repeat(logo_width + 4)
        };
        
        let info_line = if idx < lines.len() {
            &lines[idx]
        } else {
            ""
        };
        
        println!("{}  {}", logo_line, info_line);
    }
    
    max_lines
}

fn add_info_line(lines: &mut Vec<String>, show: bool, value: &Option<String>, label: &str, color: &str) {
    if show {
        if let Some(val) = value {
            lines.push(format!("{}: {}", colorize(label, color), val));
        }
    }
}

fn add_filtered_line(lines: &mut Vec<String>, show: bool, value: &Option<String>, label: &str, color: &str) {
    if show {
        if let Some(val) = value {
            if val != "Unknown" {
                lines.push(format!("{}: {}", colorize(label, color), val));
            }
        }
    }
}

#[inline(always)]
fn colorize(text: &str, color: &str) -> String {
    if USE_COLOR {
        format!("{}{}{}", color, text, C_RESET)
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
        .or_else(|_| run_command("hostname"))
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}

fn get_os() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find_map(|line| line.strip_prefix("PRETTY_NAME="))
                .map(|s| s.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Unknown OS".to_string())
}

fn get_kernel() -> String {
    run_command("uname -r").unwrap_or_else(|_| "Unknown".to_string())
}

fn get_uptime() -> String {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|content| {
            content
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|seconds| {
                    let days = (seconds / 86400.0) as u64;
                    let hours = ((seconds % 86400.0) / 3600.0) as u64;
                    let minutes = ((seconds % 3600.0) / 60.0) as u64;
                    
                    match (days, hours) {
                        (d, h) if d > 0 => format!("{}d {}h {}m", d, h, minutes),
                        (_, h) if h > 0 => format!("{}h {}m", h, minutes),
                        _ => format!("{}m", minutes),
                    }
                })
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_packages() -> String {
    let count = try_count_packages("pacman", &["-Qq"])
        .or_else(|| try_count_packages("dpkg", &["-l"]).map(|c| c.saturating_sub(5)))
        .or_else(|| try_count_packages("rpm", &["-qa"]))
        .unwrap_or(0);
    
    if count > 0 {
        count.to_string()
    } else {
        "Unknown".to_string()
    }
}

fn try_count_packages(cmd: &str, args: &[&str]) -> Option<usize> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.lines().count())
}

#[inline(always)]
fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(String::from))
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_de() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| {
            if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
                "GNOME".to_string()
            } else if std::env::var("KDE_FULL_SESSION").is_ok() {
                "KDE".to_string()
            } else {
                "Unknown".to_string()
            }
        })
}

fn get_wm() -> String {
    let window_managers = [
        "hyprland", "sway", "i3", "bspwm", "awesome", "dwm", "openbox", "xmonad",
    ];
    
    for wm in &window_managers {
        if run_command(&format!("pgrep -x {}", wm)).is_ok() {
            return wm.to_string();
        }
    }
    
    "Unknown".to_string()
}

fn get_wm_theme() -> String {
    std::env::var("GTK_THEME").ok().or_else(|| {
        let home = std::env::var("HOME").ok()?;
        let settings_path = format!("{}/.config/gtk-3.0/settings.ini", home);
        
        fs::read_to_string(settings_path)
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find_map(|line| line.strip_prefix("gtk-theme-name=").map(String::from))
            })
    })
    .unwrap_or_else(|| "Unknown".to_string())
}

fn get_terminal() -> String {
    std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("TERMINAL"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_cpu() -> String {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            let mut model_name = None;
            let mut core_count = 0;
            
            for line in content.lines() {
                if model_name.is_none() && line.starts_with("model name") {
                    model_name = line.split(':').nth(1).map(|s| {
                        s.trim()
                            .replace("(R)", "")
                            .replace("(TM)", "")
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ")
                    });
                }
                if line.starts_with("processor") {
                    core_count += 1;
                }
            }
            
            model_name.map(|name| format!("{} ({} cores)", name, core_count))
        })
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_gpu() -> Vec<String> {
    let mut gpus = Vec::new();
    let mut seen_names = HashSet::new();
    
    // Method 1: NVIDIA GPUs via nvidia-smi
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=gpu_name", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines().filter(|l| !l.is_empty()) {
                    let gpu = line.trim().to_string();
                    if seen_names.insert(gpu.to_lowercase()) {
                        gpus.push(format!("NVIDIA {}", gpu));
                    }
                }
            }
        }
    }
    
    // Method 2: lspci for ALL GPUs
    if let Ok(output) = Command::new("lspci").output() {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                if line.contains("VGA compatible controller")
                    || line.contains("3D controller")
                    || line.contains("Display controller")
                {
                    if let Some(after_colon) = line.split(": ").last() {
                        let mut gpu_name = after_colon.trim().to_string();
                        
                        // Remove revision info
                        if let Some(rev_pos) = gpu_name.find(" (rev ") {
                            gpu_name = gpu_name[..rev_pos].to_string();
                        }
                        
                        // Clean up the name
                        gpu_name = gpu_name
                            .replace("Corporation ", "")
                            .replace("Integrated Graphics Controller", "Graphics")
                            .trim()
                            .to_string();
                        
                        // Skip NVIDIA duplicates
                        let normalized = gpu_name.to_lowercase();
                        let is_nvidia_duplicate = gpus.iter().any(|g| {
                            let g_lower = g.to_lowercase();
                            g_lower.contains("nvidia")
                                && normalized.contains("nvidia")
                                && (g_lower.contains(&normalized.replace("nvidia ", ""))
                                    || normalized.contains(&g_lower.replace("nvidia ", "")))
                        });
                        
                        if !is_nvidia_duplicate && seen_names.insert(normalized) {
                            gpus.push(gpu_name);
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

fn get_memory_swap() -> (String, String) {
    let (mut mem_total, mut mem_available, mut swap_total, mut swap_free) =
        (None, None, None, None);
    
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        for line in content.lines() {
            if mem_total.is_none() && line.starts_with("MemTotal:") {
                mem_total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            } else if mem_available.is_none() && line.starts_with("MemAvailable:") {
                mem_available = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            } else if swap_total.is_none() && line.starts_with("SwapTotal:") {
                swap_total = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            } else if swap_free.is_none() && line.starts_with("SwapFree:") {
                swap_free = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok());
            }
            
            if mem_total.is_some()
                && mem_available.is_some()
                && swap_total.is_some()
                && swap_free.is_some()
            {
                break;
            }
        }
    }
    
    let memory = if let (Some(total), Some(available)) = (mem_total, mem_available) {
        let used = total - available;
        format!(
            "{:.1} GiB / {:.1} GiB",
            used as f64 / 1048576.0,
            total as f64 / 1048576.0
        )
    } else {
        "Unknown".to_string()
    };
    
    let swap = if let (Some(total), Some(free)) = (swap_total, swap_free) {
        if total > 0 {
            let used = total - free;
            format!(
                "{:.1} GiB / {:.1} GiB",
                used as f64 / 1048576.0,
                total as f64 / 1048576.0
            )
        } else {
            "0 B".to_string()
        }
    } else {
        "0 B".to_string()
    };
    
    (memory, swap)
}

fn get_disk() -> String {
    run_command("df -h /")
        .ok()
        .and_then(|output| {
            output.lines().nth(1).map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    format!("{} / {} ({})", parts[2], parts[1], parts[4])
                } else {
                    "Unknown".to_string()
                }
            })
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_lsblk() -> Vec<String> {
    run_command("lsblk -o NAME,SIZE,TYPE,MOUNTPOINT")
        .ok()
        .map(|output| output.lines().skip(1).map(String::from).collect())
        .unwrap_or_default()
}

fn get_locale() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "Unknown".to_string())
}

fn get_local_ip() -> String {
    run_command("hostname -I")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(String::from))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_public_ip() -> String {
    run_command("curl -s ifconfig.me").unwrap_or_else(|_| "Unknown".to_string())
}

fn get_battery() -> String {
    fs::read_dir("/sys/class/power_supply")
        .ok()
        .and_then(|entries| {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with("BAT") {
                        let capacity = fs::read_to_string(path.join("capacity"))
                            .ok()?
                            .trim()
                            .to_string();
                        let status = fs::read_to_string(path.join("status"))
                            .ok()?
                            .trim()
                            .to_string();
                        return Some(format!("{}% ({})", capacity, status));
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_logo(os: &str) -> Vec<String> {
    let os_lower = os.to_lowercase();
    
    let lines = if os_lower.contains("arch") || os_lower.contains("cachy") {
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
    } else {
        vec![
            "   ______   ",
            "  /      \\  ",
            " |  ◉  ◉  | ",
            " |    >   | ",
            " |  \\___/ | ",
            "  \\______/  ",
        ]
    };
    
    lines.into_iter().map(String::from).collect()
}

fn run_command(cmd: &str) -> Result<String, ()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(());
    }
    
    Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .ok_or(())
}

fn get_cached(cache: &Arc<Mutex<Cache>>, key: &str, f: fn() -> String) -> String {
    let mut cache_guard = cache.lock().unwrap();
    
    if let Some(value) = cache_guard.get(key) {
        return value;
    }
    
    let value = f();
    cache_guard.set(key, value.clone());
    value
}

fn get_cached_vec(cache: &Arc<Mutex<Cache>>, key: &str, f: fn() -> Vec<String>) -> Vec<String> {
    let mut cache_guard = cache.lock().unwrap();
    
    if let Some(value) = cache_guard.get(key) {
        return value.split("||").map(String::from).collect();
    }
    
    let value = f();
    cache_guard.set(key, value.join("||"));
    value
}

fn get_cached_static(cache: &Cache, key: &str, f: fn() -> String) -> String {
    if let Some(value) = cache.get(key) {
        return value;
    }
    f()
}

fn get_cached_vec_static(cache: &Cache, key: &str, f: fn() -> Vec<String>) -> Vec<String> {
    if let Some(value) = cache.get(key) {
        return value.split("||").map(String::from).collect();
    }
    f()
}
