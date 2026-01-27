use std::{
    fs, process::Command, thread, sync::{Arc, Mutex, OnceLock}, 
    io::{self, Write, BufReader, BufRead},
    collections::{HashMap, HashSet}, time::Duration,
};

const PROGRESSIVE_DISPLAY: bool = false;
const USE_COLOR: bool = true;
const CACHE_ENABLED: bool = true;
const CACHE_FILE: &str = "/tmp/rustfetch_cache";
const PROGRESS_BAR_WIDTH: usize = 20;

const SHOW_OS: bool = true;
const SHOW_KERNEL: bool = true;
const SHOW_UPTIME: bool = true;
const SHOW_BOOT_TIME: bool = true;
const SHOW_BOOTLOADER: bool = true;
const SHOW_PACKAGES: bool = true;
const SHOW_SHELL: bool = true;
const SHOW_DE: bool = true;
const SHOW_WM: bool = true;
const SHOW_INIT: bool = true;
const SHOW_TERMINAL: bool = true;
const SHOW_CPU: bool = true;
const SHOW_CPU_TEMP: bool = true;
const SHOW_GPU: bool = true;
const SHOW_GPU_TEMP: bool = true;
const SHOW_MEMORY: bool = true;
const SHOW_SWAP: bool = true;
const SHOW_DISKS_DETAILED: bool = true;
const SHOW_PARTITIONS: bool = true;
const SHOW_NETWORK: bool = true;
const SHOW_DISPLAY: bool = true;
const SHOW_BATTERY: bool = true;
const SHOW_COLORS: bool = true;

// RGB color constants
const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_CYAN: &str = "\x1b[38;2;0;255;255m";
const C_GREEN: &str = "\x1b[38;2;0;255;0m";
const C_YELLOW: &str = "\x1b[38;2;255;255;0m";
const C_BLUE: &str = "\x1b[38;2;0;0;255m";
const C_MAGENTA: &str = "\x1b[38;2;255;0;255m";
const C_RED: &str = "\x1b[38;2;255;0;0m";
const C_ORANGE: &str = "\x1b[38;2;255;165;0m";

const KB_TO_GIB: f64 = 1024.0 * 1024.0;
const BYTES_TO_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

const UNKNOWN: &str = "Unknown";

// Progress bar characters
const FILLED_CHAR: char = '█';
const EMPTY_CHAR: char = '░';

#[derive(Default, Clone)]
struct Info {
    user: Option<String>, 
    hostname: Option<String>, 
    os: Option<String>,
    kernel: Option<String>, 
    uptime: Option<String>, 
    boot_time: Option<String>,
    bootloader: Option<String>,
    packages: Option<String>, 
    shell: Option<String>, 
    de: Option<String>, 
    wm: Option<String>, 
    init: Option<String>, 
    terminal: Option<String>, 
    cpu: Option<String>, 
    cpu_temp: Option<String>, 
    gpu: Option<Vec<String>>, 
    gpu_temp: Option<String>, 
    memory: Option<(f64, f64)>, 
    swap: Option<(f64, f64)>,
    disks_detailed: Option<Vec<(String, f64, f64, f64, String)>>,
    partitions: Option<Vec<(String, String, f64, f64)>>,
    network: Option<String>, 
    display: Option<String>,
    battery: Option<(u8, String)>,
}

struct Cache { 
    data: HashMap<String, String>,
    dirty: bool,
}

impl Cache {
    fn load() -> Self {
        if !CACHE_ENABLED { 
            return Cache { data: HashMap::new(), dirty: false }; 
        }
        fs::read_to_string(CACHE_FILE).ok()
            .map(|c| {
                let mut map = HashMap::with_capacity(20);
                for l in c.lines() {
                    if let Some((k, v)) = l.split_once('=') {
                        map.insert(k.to_string(), v.to_string());
                    }
                }
                Cache { data: map, dirty: false }
            })
            .unwrap_or_else(|| Cache { data: HashMap::with_capacity(20), dirty: false })
    }
    
    fn get(&self, key: &str) -> Option<&str> { 
        self.data.get(key).map(|s| s.as_str())
    }
    
    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
        self.dirty = true;
    }
    
    fn save(&self) {
        if !CACHE_ENABLED || !self.dirty { return; }
        let mut content = String::with_capacity(self.data.len() * 30);
        for (k, v) in &self.data {
            content.push_str(k);
            content.push('=');
            content.push_str(v);
            content.push('\n');
        }
        let _ = fs::write(CACHE_FILE, content);
    }
}

fn main() {
    let cache = Arc::new(Mutex::new(Cache::load()));
    let info = Arc::new(Mutex::new(Info::default()));
    let logo = Arc::new(OnceLock::new());

    if PROGRESSIVE_DISPLAY {
        progressive_display(cache, info, logo);
    } else {
        static_display(cache, info, logo);
    }
}

fn progressive_display(
    cache: Arc<Mutex<Cache>>, 
    info: Arc<Mutex<Info>>, 
    logo: Arc<OnceLock<Vec<String>>>
) {
    let (info_c, logo_c) = (Arc::clone(&info), Arc::clone(&logo));
    
    let display_thread = thread::spawn(move || {
        let mut last_lines = 0;
        loop {
            thread::sleep(Duration::from_millis(50));
            if let Ok(i) = info_c.lock() {
                let l = logo_c.get().map(|v| v.as_slice()).unwrap_or(&[]);
                if last_lines > 0 { 
                    print!("\x1b[{}A\x1b[J", last_lines); 
                }
                last_lines = display_info(&i, l);
                let _ = io::stdout().flush();
                if check_loaded(&i) { break; }
            }
        }
    });

    let handles = vec![
        spawn_gather(&info, &cache, &logo, |i, c| {
            gather_basic_info(i, c);
            gather_system_info(i, c);
        }),
        spawn_gather(&info, &cache, &logo, gather_hardware_info),
        spawn_gather(&info, &cache, &logo, gather_resources),
    ];

    for h in handles { let _ = h.join(); }
    if let Ok(c) = cache.lock() { c.save(); }
    let _ = display_thread.join();
}

fn static_display(
    cache: Arc<Mutex<Cache>>, 
    info: Arc<Mutex<Info>>, 
    logo: Arc<OnceLock<Vec<String>>>
) {
    if let Ok(mut c) = cache.lock() {
        if let Ok(mut i) = info.lock() {
            gather_basic_info(&mut i, &mut c);
            gather_system_info(&mut i, &mut c);
            gather_hardware_info(&mut i, &mut c);
            gather_resources(&mut i, &mut c);
            
            if let Some(os) = &i.os {
                let _ = logo.set(get_logo(os));
            }
        }
        c.save();
    }
    
    if let (Ok(i), Some(l)) = (info.lock(), logo.get()) { 
        display_info(&i, l); 
    }
}

fn gather_basic_info(info: &mut Info, cache: &mut Cache) {
    info.user = Some(std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| UNKNOWN.to_string()));
    
    info.hostname = Some(cache_or(cache, "host", || 
        fs::read_to_string("/etc/hostname").ok()
            .or_else(|| run("hostname -s"))
            .unwrap_or_else(|| UNKNOWN.to_string())
            .trim()
            .to_string()));
    
    info.os = Some(cache_or(cache, "os", || 
        fs::read_to_string("/etc/os-release").ok()
            .and_then(|c| c.lines().find_map(|l| l.strip_prefix("PRETTY_NAME="))
                .map(|s| s.trim_matches('"').to_string()))
            .unwrap_or_else(|| "Unknown OS".to_string())));
}

fn gather_system_info(info: &mut Info, cache: &mut Cache) {
    if SHOW_KERNEL {
        info.kernel = Some(cache_or(cache, "kernel", || 
            run("uname -r").unwrap_or_else(|| UNKNOWN.to_string())));
    }
    if SHOW_UPTIME { 
        info.uptime = Some(get_uptime()); 
    }
    if SHOW_BOOT_TIME { 
        info.boot_time = Some(get_boot_time()); 
    }
    if SHOW_BOOTLOADER {
        info.bootloader = Some(cache_or(cache, "bootloader", get_bootloader));
    }
    if SHOW_PACKAGES { 
        info.packages = Some(cache_or(cache, "pkgs", get_packages)); 
    }
    if SHOW_SHELL {
        info.shell = Some(std::env::var("SHELL").ok()
            .and_then(|s| s.rsplit('/').next().map(String::from))
            .unwrap_or_else(|| UNKNOWN.to_string()));
    }
    if SHOW_DE {
        info.de = Some(std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| UNKNOWN.to_string()));
    }
    if SHOW_WM { 
        info.wm = Some(cache_or(cache, "wm", get_wm)); 
    }
    if SHOW_INIT { 
        info.init = Some(cache_or(cache, "init", get_init)); 
    }
    if SHOW_TERMINAL {
        info.terminal = Some(std::env::var("TERM_PROGRAM")
            .or_else(|_| std::env::var("TERMINAL"))
            .unwrap_or_else(|_| UNKNOWN.to_string()));
    }
}

fn gather_hardware_info(info: &mut Info, cache: &mut Cache) {
    if SHOW_CPU { 
        info.cpu = Some(cache_or(cache, "cpu", get_cpu)); 
    }
    
    // Parallel temperature gathering
    if SHOW_CPU_TEMP && SHOW_GPU_TEMP {
        let cpu_handle = thread::spawn(get_cpu_temp);
        let gpu_temp = get_gpu_temp();
        info.gpu_temp = gpu_temp;
        info.cpu_temp = cpu_handle.join().ok().flatten();
    } else {
        if SHOW_CPU_TEMP { 
            info.cpu_temp = get_cpu_temp(); 
        }
        if SHOW_GPU_TEMP { 
            info.gpu_temp = get_gpu_temp(); 
        }
    }
    
    if SHOW_GPU { 
        info.gpu = Some(cache_or_vec(cache, "gpu", get_gpu)); 
    }
}

fn gather_resources(info: &mut Info, _cache: &mut Cache) {
    if SHOW_MEMORY || SHOW_SWAP {
        let (mem, swap) = get_memory_swap();
        if SHOW_MEMORY { info.memory = mem; }
        if SHOW_SWAP { info.swap = swap; }
    }
    if SHOW_DISKS_DETAILED { 
        info.disks_detailed = get_disks_detailed(); 
    }
    if SHOW_PARTITIONS { 
        info.partitions = get_partitions(); 
    }
    if SHOW_NETWORK { 
        info.network = get_network(); 
    }
    if SHOW_DISPLAY { 
        info.display = get_display(); 
    }
    if SHOW_BATTERY { 
        info.battery = get_battery(); 
    }
}

fn spawn_gather<F>(
    i: &Arc<Mutex<Info>>, 
    c: &Arc<Mutex<Cache>>, 
    l: &Arc<OnceLock<Vec<String>>>, 
    f: F
) -> thread::JoinHandle<()>
where F: FnOnce(&mut Info, &mut Cache) + Send + 'static {
    let (i, c, l) = (Arc::clone(i), Arc::clone(c), Arc::clone(l));
    thread::spawn(move || {
        if let (Ok(mut info), Ok(mut cache)) = (i.lock(), c.lock()) {
            f(&mut info, &mut cache);
            
            if l.get().is_none() {
                if let Some(os) = &info.os {
                    let _ = l.set(get_logo(os));
                }
            }
        }
    })
}

fn check_loaded(i: &Info) -> bool {
    i.user.is_some() && i.hostname.is_some() && i.os.is_some()
        && (!SHOW_KERNEL || i.kernel.is_some())
        && (!SHOW_UPTIME || i.uptime.is_some())
        && (!SHOW_BOOT_TIME || i.boot_time.is_some())
        && (!SHOW_BOOTLOADER || i.bootloader.is_some())
        && (!SHOW_PACKAGES || i.packages.is_some())
        && (!SHOW_SHELL || i.shell.is_some())
        && (!SHOW_DE || i.de.is_some())
        && (!SHOW_WM || i.wm.is_some())
        && (!SHOW_INIT || i.init.is_some())
        && (!SHOW_TERMINAL || i.terminal.is_some())
        && (!SHOW_CPU || i.cpu.is_some())
        && (!SHOW_CPU_TEMP || i.cpu_temp.is_some())
        && (!SHOW_GPU || i.gpu.is_some())
        && (!SHOW_GPU_TEMP || i.gpu_temp.is_some())
        && (!SHOW_MEMORY || i.memory.is_some())
        && (!SHOW_SWAP || i.swap.is_some())
        && (!SHOW_DISKS_DETAILED || i.disks_detailed.is_some())
        && (!SHOW_PARTITIONS || i.partitions.is_some())
        && (!SHOW_NETWORK || i.network.is_some())
        && (!SHOW_DISPLAY || i.display.is_some())
        && (!SHOW_BATTERY || i.battery.is_some())
}

fn progress_bar(used: f64, total: f64, color: &str) -> String {
    let pct = if total > 0.0 { (used / total * 100.0).min(100.0) } else { 0.0 };
    let filled = ((pct / 100.0) * PROGRESS_BAR_WIDTH as f64) as usize;
    let empty = PROGRESS_BAR_WIDTH.saturating_sub(filled);
    
    let mut result = String::with_capacity(100);
    if USE_COLOR { result.push_str(color); }
    result.push_str(&format!("{:.1}/{:.1} GiB ", used, total));
    if USE_COLOR { result.push_str(C_RESET); }
    result.push('[');
    
    if USE_COLOR {
        result.push_str(color);
        result.extend(std::iter::repeat(FILLED_CHAR).take(filled));
        result.push_str(C_RESET);
        result.extend(std::iter::repeat(EMPTY_CHAR).take(empty));
    } else {
        result.extend(std::iter::repeat('#').take(filled));
        result.extend(std::iter::repeat('-').take(empty));
    }
    
    result.push_str(&format!("] {:.0}%", pct));
    if USE_COLOR { result.push_str(C_RESET); }
    result
}

fn battery_bar(level: u8, status: &str, color: &str) -> String {
    let filled = ((level as f64 / 100.0) * PROGRESS_BAR_WIDTH as f64) as usize;
    let empty = PROGRESS_BAR_WIDTH.saturating_sub(filled);
    
    let mut result = String::with_capacity(100);
    result.push_str(&format!("{}% ({}) [", level, status));
    
    if USE_COLOR {
        result.push_str(color);
        result.extend(std::iter::repeat(FILLED_CHAR).take(filled));
        result.push_str(C_RESET);
        result.extend(std::iter::repeat(EMPTY_CHAR).take(empty));
    } else {
        result.extend(std::iter::repeat('#').take(filled));
        result.extend(std::iter::repeat('-').take(empty));
    }
    
    result.push(']');
    if USE_COLOR { result.push_str(C_RESET); }
    result
}

fn strip_ansi(s: &str) -> usize {
    let mut count = 0;
    let mut chars = s.chars();
    
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() { 
                    break; 
                }
            }
        } else {
            count += 1;
        }
    }
    count
}

fn display_info(info: &Info, logo: &[String]) -> usize {
    let mut lines = Vec::with_capacity(30);
    
    if let (Some(u), Some(h)) = (&info.user, &info.hostname) {
        lines.push(format!("{}{}{}", 
            colorize(u, C_BOLD), 
            colorize("@", C_RESET), 
            colorize(h, C_BOLD)));
        lines.push("─".repeat(u.len() + h.len() + 1));
    }
    
    add(&mut lines, SHOW_OS, &info.os, "OS", C_CYAN);
    add(&mut lines, SHOW_KERNEL, &info.kernel, "Kernel", C_CYAN);
    add(&mut lines, SHOW_UPTIME, &info.uptime, "Uptime", C_CYAN);
    add(&mut lines, SHOW_BOOT_TIME, &info.boot_time, "Boot Time", C_GREEN);
    add(&mut lines, SHOW_BOOTLOADER, &info.bootloader, "Bootloader", C_CYAN);
    add(&mut lines, SHOW_PACKAGES, &info.packages, "Packages", C_CYAN);
    add(&mut lines, SHOW_SHELL, &info.shell, "Shell", C_CYAN);
    add_filt(&mut lines, SHOW_DE, &info.de, "DE", C_CYAN);
    add_filt(&mut lines, SHOW_WM, &info.wm, "WM", C_CYAN);
    add_filt(&mut lines, SHOW_INIT, &info.init, "Init", C_CYAN);
    add_filt(&mut lines, SHOW_TERMINAL, &info.terminal, "Terminal", C_CYAN);
    add(&mut lines, SHOW_CPU, &info.cpu, "CPU", C_GREEN);
    add_filt(&mut lines, SHOW_CPU_TEMP, &info.cpu_temp, "CPU Temp", C_YELLOW);
    
    if SHOW_GPU {
        if let Some(gpus) = &info.gpu {
            for (i, g) in gpus.iter().enumerate() {
                lines.push(if i == 0 { 
                    format!("{}: {}", colorize("GPU", C_MAGENTA), g) 
                } else { 
                    format!("    {}", g) 
                });
            }
        }
    }
    add_filt(&mut lines, SHOW_GPU_TEMP, &info.gpu_temp, "GPU Temp", C_YELLOW);
    
    if SHOW_MEMORY {
        if let Some((used, total)) = info.memory {
            lines.push(format!("{}: {}", 
                colorize("Memory", C_YELLOW), 
                progress_bar(used, total, C_YELLOW)));
        }
    }
    
    if SHOW_SWAP {
        if let Some((used, total)) = info.swap {
            if total > 0.0 {
                lines.push(format!("{}: {}", 
                    colorize("Swap", C_YELLOW), 
                    progress_bar(used, total, C_YELLOW)));
            }
        }
    }
    
    if SHOW_DISKS_DETAILED {
        if let Some(disks) = &info.disks_detailed {
            for (idx, (name, size, used, _total, disk_type)) in disks.iter().enumerate() {
                if idx == 0 {
                    lines.push(format!("{}: {} {} {}", 
                        colorize("Disks", C_BLUE), name, 
                        progress_bar(*used, *size, C_BLUE), disk_type));
                } else {
                    lines.push(format!("       {} {} {}", name, 
                        progress_bar(*used, *size, C_BLUE), disk_type));
                }
            }
        }
    }
    
    if SHOW_PARTITIONS {
        if let Some(parts) = &info.partitions {
            for (dev, mount, used, total) in parts.iter() {
                lines.push(format!("       {} {} {}", 
                    dev, mount, progress_bar(*used, *total, C_BLUE)));
            }
        }
    }
    
    add_filt(&mut lines, SHOW_NETWORK, &info.network, "Network", C_CYAN);
    add_filt(&mut lines, SHOW_DISPLAY, &info.display, "Display", C_CYAN);
    
    if SHOW_BATTERY {
        if let Some((level, status)) = &info.battery {
            let color = if *level > 50 { C_GREEN } 
                       else if *level > 20 { C_YELLOW } 
                       else { C_RED };
            lines.push(format!("{}: {}", 
                colorize("Battery", color), 
                battery_bar(*level, status, color)));
        }
    }
    
    if SHOW_COLORS {
        lines.push(String::new());
        lines.push(color_blocks());
    }
    
    let logo_width = logo.iter().map(|s| strip_ansi(s)).max().unwrap_or(0);
    let max_lines = logo.len().max(lines.len());
    
    for idx in 0..max_lines {
        let logo_line = if idx < logo.len() {
            let stripped_len = strip_ansi(&logo[idx]);
            let padding = logo_width.saturating_sub(stripped_len);
            format!("{}{}", colorize(&logo[idx], C_BLUE), " ".repeat(padding))
        } else { 
            " ".repeat(logo_width) 
        };
        
        let info_line = if idx < lines.len() { &lines[idx] } else { "" };
        println!("{}  {}", logo_line, info_line);
    }
    max_lines
}

fn color_blocks() -> String {
    if !USE_COLOR {
        return "███████████████████".to_string();
    }
    
    let mut result = String::new();
    let colors = [
        (255, 0, 0),     // Red
        (255, 127, 0),   // Orange
        (255, 255, 0),   // Yellow
        (0, 255, 0),     // Green
        (0, 255, 255),   // Cyan
        (0, 0, 255),     // Blue
        (127, 0, 255),   // Indigo
        (255, 0, 255),   // Magenta
        (255, 255, 255), // White
    ];
    
    for (r, g, b) in colors {
        result.push_str(&format!("\x1b[38;2;{};{};{}m██", r, g, b));
    }
    result.push_str(C_RESET);
    result
}

#[inline]
fn add(lines: &mut Vec<String>, show: bool, val: &Option<String>, label: &str, color: &str) {
    if show { 
        if let Some(v) = val { 
            lines.push(format!("{}: {}", colorize(label, color), v)); 
        } 
    }
}

#[inline]
fn add_filt(lines: &mut Vec<String>, show: bool, val: &Option<String>, label: &str, color: &str) {
    if show { 
        if let Some(v) = val { 
            if v != UNKNOWN { 
                lines.push(format!("{}: {}", colorize(label, color), v)); 
            } 
        } 
    }
}

#[inline]
fn colorize(text: &str, color: &str) -> String {
    if !USE_COLOR { 
        return text.to_string(); 
    }
    let mut s = String::with_capacity(text.len() + color.len() + C_RESET.len());
    s.push_str(color);
    s.push_str(text);
    s.push_str(C_RESET);
    s
}

fn cache_or<F: Fn() -> String>(cache: &mut Cache, key: &str, f: F) -> String {
    if let Some(val) = cache.get(key) {
        return val.to_string();
    }
    let val = f();
    cache.set(key.to_string(), val.clone());
    val
}

fn cache_or_vec<F: Fn() -> Vec<String>>(cache: &mut Cache, key: &str, f: F) -> Vec<String> {
    if let Some(v) = cache.get(key) {
        return v.split("||").map(String::from).collect();
    }
    let val = f();
    let joined = val.join("||");
    cache.set(key.to_string(), joined);
    val
}

#[inline]
fn run(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return None; }
    
    Command::new(parts[0])
        .args(&parts[1..])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|o| if o.status.success() { 
            String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()) 
        } else { 
            None 
        })
}

fn get_uptime() -> String {
    fs::read_to_string("/proc/uptime").ok()
        .and_then(|c| c.split_whitespace().next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|sec| {
                let days = (sec / 86400.0) as u64;
                let hours = ((sec % 86400.0) / 3600.0) as u64;
                let mins = ((sec % 3600.0) / 60.0) as u64;
                
                match (days, hours) {
                    (d, h) if d > 0 => format!("{}d {}h {}m", d, h, mins),
                    (_, h) if h > 0 => format!("{}h {}m", h, mins),
                    _ => format!("{}m", mins),
                }
            }))
        .unwrap_or_else(|| UNKNOWN.to_string())
}

#[inline]
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn get_boot_time() -> String {
    fs::read_to_string("/proc/stat").ok()
        .and_then(|c| c.lines()
            .find(|l| l.starts_with("btime"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<i64>().ok())
            .map(|timestamp| {
                // Convert Unix timestamp to human readable date/time
                let seconds_since_epoch = timestamp;
                let days_since_epoch = seconds_since_epoch / 86400;
                let seconds_today = seconds_since_epoch % 86400;
                
                // Calculate year
                let mut year = 1970;
                let mut remaining_days = days_since_epoch;
                
                loop {
                    let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                    if remaining_days < days_in_year { break; }
                    remaining_days -= days_in_year;
                    year += 1;
                }
                
                // Calculate month
                let days_in_months = if is_leap_year(year) {
                    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                } else {
                    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                };
                
                let mut month = 1;
                let mut day_in_month = remaining_days + 1;
                
                for &days in &days_in_months {
                    if day_in_month <= days { break; }
                    day_in_month -= days;
                    month += 1;
                }
                
                // Calculate time
                let hours = seconds_today / 3600;
                let mins = (seconds_today % 3600) / 60;
                let secs = seconds_today % 60;
                
                format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", 
                    year, month, day_in_month, hours, mins, secs)
            }))
        .unwrap_or_else(|| UNKNOWN.to_string())
}

fn get_bootloader() -> String {
    // Check for Limine first (you mentioned you use it)
    let limine_checks = [
        "/boot/limine.cfg",
        "/boot/EFI/limine/limine.cfg",
        "/boot/limine/limine.cfg",
        "/efi/limine.cfg",
    ];
    
    for check in &limine_checks {
        if fs::metadata(check).is_ok() {
            if let Some(version) = run("limine version 2>/dev/null") {
                return format!("Limine {}", version.trim());
            }
            return "Limine".to_string();
        }
    }
    
    // Check for Limine binary files
    let limine_files = [
        "/boot/limine.sys",
        "/boot/limine-efi.bin",
        "/boot/limine-bios.bin",
    ];
    
    for file in &limine_files {
        if fs::metadata(file).is_ok() {
            return "Limine".to_string();
        }
    }
    
    // Check UEFI vs BIOS
    let is_uefi = fs::metadata("/sys/firmware/efi").is_ok();
    
    // Check for systemd-boot
    if fs::metadata("/boot/loader/loader.conf").is_ok() ||
       fs::metadata("/boot/efi/loader/loader.conf").is_ok() {
        if let Some(version) = run("bootctl --version") {
            if let Some(ver) = version.lines().next() {
                return format!("systemd-boot {}", ver.trim());
            }
        }
        return "systemd-boot".to_string();
    }
    
    // Check for GRUB
    if fs::metadata("/boot/grub/grub.cfg").is_ok() ||
       fs::metadata("/boot/grub2/grub.cfg").is_ok() {
        if let Some(version) = run("grub-install --version") {
            if let Some(ver) = version.split_whitespace().last() {
                let mode = if is_uefi { "UEFI" } else { "BIOS" };
                return format!("GRUB {} ({})", ver, mode);
            }
        }
        let mode = if is_uefi { "UEFI" } else { "BIOS" };
        return format!("GRUB ({})", mode);
    }
    
    // Check Windows Boot Manager
    if fs::metadata("/boot/efi/EFI/Microsoft").is_ok() {
        return "Windows Boot Manager".to_string();
    }
    
    // Check rEFInd
    if fs::metadata("/boot/efi/EFI/refind").is_ok() {
        return "rEFInd".to_string();
    }
    
    // Check for U-Boot (common on ARM/Raspberry Pi)
    if fs::metadata("/boot/u-boot").is_ok() ||
       fs::metadata("/boot/boot.scr").is_ok() {
        return "U-Boot".to_string();
    }
    
    // Check /proc/cmdline for hints (fast)
    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
        if cmdline.contains("limine") {
            return "Limine".to_string();
        } else if cmdline.contains("systemd") {
            return "systemd-boot".to_string();
        } else if cmdline.contains("grub") {
            return "GRUB".to_string();
        }
    }
    
    // Final fallback
    if is_uefi {
        "UEFI Firmware".to_string()
    } else {
        UNKNOWN.to_string()
    }
}

fn get_packages() -> String {
    // FAST: Pacman (Arch-based)
    if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
        let count = entries.filter_map(Result::ok).count().saturating_sub(1);
        if count > 0 { 
            return count.to_string(); 
        }
    }
    
    // dpkg (Debian-based)
    if let Some(count) = try_count("dpkg-query", &["-f", ".\n", "-W"]) {
        return count.to_string();
    }
    
    // rpm (RedHat-based)
    if let Some(count) = try_count("rpm", &["-qa"]) {
        return count.to_string();
    }
    
    // xbps (Void Linux)
    if let Some(count) = try_count("xbps-query", &["-l"]) {
        return count.to_string();
    }
    
    UNKNOWN.to_string()
}

fn try_count(cmd: &str, args: &[&str]) -> Option<usize> {
    Command::new(cmd)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.is_empty()).count())
}

fn get_wm() -> String {
    // Check environment variable first (fastest)
    if let Ok(wm) = std::env::var("XDG_CURRENT_WM") {
        return wm;
    }
    
    // Check common window managers
    const WMS: [&str; 12] = [
        "hyprland", "sway", "i3", "bspwm", "awesome", "dwm", 
        "openbox", "xmonad", "qtile", "river", "wayfire", "xfwm4"
    ];
    
    // Read process list once
    if let Ok(output) = Command::new("ps")
        .args(&["-e", "-o", "comm="])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output() 
    {
        if let Ok(procs) = String::from_utf8(output.stdout) {
            for wm in &WMS {
                if procs.lines().any(|l| l == *wm) {
                    return wm.to_string();
                }
            }
        }
    }
    
    UNKNOWN.to_string()
}

fn get_init() -> String {
    // Check systemd (fastest)
    if fs::metadata("/run/systemd/system").is_ok() {
        return "systemd".to_string();
    }
    
    // Read /proc/1/comm
    if let Ok(init_name) = fs::read_to_string("/proc/1/comm") {
        let init = init_name.trim();
        return match init {
            "systemd" => "systemd".to_string(),
            "init" => {
                if fs::metadata("/run/openrc").is_ok() {
                    "openrc".to_string()
                } else if fs::metadata("/etc/runit").is_ok() {
                    "runit".to_string()
                } else {
                    "sysvinit".to_string()
                }
            },
            "runit" => "runit".to_string(),
            "dinit" => "dinit".to_string(),
            "s6-svscan" => "s6".to_string(),
            _ => init.to_string(),
        }
    }
    
    UNKNOWN.to_string()
}

fn get_cpu() -> String {
    let file = match fs::File::open("/proc/cpuinfo") {
        Ok(f) => f,
        Err(_) => return "Unknown CPU".to_string(),
    };
    
    let reader = BufReader::new(file);
    let mut model = None;
    let mut cores = 0;
    
    for line in reader.lines().flatten() {
        if model.is_none() && line.starts_with("model name") {
            model = line.split_once(':').map(|(_, s)| {
                s.trim()
                    .replace("(R)", "")
                    .replace("(TM)", "")
                    .replace("(tm)", "")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            });
        }
        if line.starts_with("processor") { 
            cores += 1;
        }
        // Early exit if we have both
        if model.is_some() && cores > 0 {
            break;
        }
    }
    
    model.map(|m| format!("{} ({} cores)", m, cores))
        .unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_gpu() -> Vec<String> {
    let mut gpus = Vec::with_capacity(2);
    
    // Try lspci first
    if let Ok(o) = Command::new("lspci")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output() 
    {
        if let Ok(s) = String::from_utf8(o.stdout) {
            let mut seen = HashSet::with_capacity(2);
            for l in s.lines() {
                if l.contains("VGA") || l.contains("3D controller") {
                    if let Some(name) = l.split(": ").nth(1) {
                        let clean = name.split(" (rev").next().unwrap_or(name)
                            .replace("Corporation ", "")
                            .replace("Advanced Micro Devices, Inc. ", "")
                            .replace("[AMD/ATI] ", "")
                            .replace("NVIDIA ", "")
                            .trim()
                            .to_string();
                        let key = clean.to_lowercase();
                        if seen.insert(key) { 
                            gpus.push(clean); 
                            if gpus.len() >= 2 { break; }
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

fn get_memory_swap() -> (Option<(f64, f64)>, Option<(f64, f64)>) {
    let file = match fs::File::open("/proc/meminfo") {
        Ok(f) => f,
        Err(_) => return (None, None),
    };
    
    let reader = BufReader::new(file);
    let mut mem_total = None;
    let mut mem_available = None;
    let mut swap_total = None;
    let mut swap_free = None;
    
    for line in reader.lines().flatten() {
        if mem_total.is_none() && line.starts_with("MemTotal:") {
            mem_total = line.split_whitespace().nth(1)
                .and_then(|s| s.parse::<u64>().ok());
        } else if mem_available.is_none() && line.starts_with("MemAvailable:") {
            mem_available = line.split_whitespace().nth(1)
                .and_then(|s| s.parse::<u64>().ok());
        } else if swap_total.is_none() && line.starts_with("SwapTotal:") {
            swap_total = line.split_whitespace().nth(1)
                .and_then(|s| s.parse::<u64>().ok());
        } else if swap_free.is_none() && line.starts_with("SwapFree:") {
            swap_free = line.split_whitespace().nth(1)
                .and_then(|s| s.parse::<u64>().ok());
        }
        
        // Early exit if we have all values
        if mem_total.is_some() && mem_available.is_some() && 
           swap_total.is_some() && swap_free.is_some() { 
            break; 
        }
    }
    
    let mem = if let (Some(t), Some(a)) = (mem_total, mem_available) {
        Some(((t - a) as f64 / KB_TO_GIB, t as f64 / KB_TO_GIB))
    } else { 
        None 
    };
    
    let swap = if let (Some(t), Some(f)) = (swap_total, swap_free) {
        if t > 0 { 
            Some(((t - f) as f64 / KB_TO_GIB, t as f64 / KB_TO_GIB)) 
        } else { 
            None 
        }
    } else { 
        None 
    };
    
    (mem, swap)
}

fn get_disks_detailed() -> Option<Vec<(String, f64, f64, f64, String)>> {
    let output = run("lsblk -bdno NAME,SIZE,TYPE,ROTA -e 7 -e 11")?;
    let mut disks = Vec::with_capacity(2);
    
    for line in output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 { continue; }
        
        let name = fields[0];
        let disk_type = fields[2];
        
        if disk_type != "disk" { continue; }
        
        let size_bytes = fields[1].parse::<f64>().ok()?;
        let is_rotational = fields[3] == "1";
        let size_gib = size_bytes / BYTES_TO_GIB;
        
        // Simple used space estimation
        let used_gib = if let Some(df_output) = run("df -B1 --output=source,used") {
            let mut total: f64 = 0.0;
            for l in df_output.lines().skip(1) {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 2 && parts[0].starts_with(&format!("/dev/{}", name)) {
                    if let Ok(used) = parts[1].parse::<f64>() {
                        total += used;
                    }
                }
            }
            total / BYTES_TO_GIB
        } else {
            0.0
        };
        
        let type_label = if name.starts_with("nvme") {
            "disk [NVME]"
        } else if name.starts_with("sd") {
            if is_rotational { "disk [HDD]" } else { "disk [SSD]" }
        } else if name.starts_with("mmc") {
            "disk [MMC]"
        } else if name.starts_with("zram") {
            "disk [SWAP]"
        } else {
            "disk"
        };
        
        disks.push((name.to_string(), size_gib, used_gib, size_gib, type_label.to_string()));
        
        if disks.len() >= 2 { break; }
    }
    
    if disks.is_empty() { None } else { Some(disks) }
}

fn get_partitions() -> Option<Vec<(String, String, f64, f64)>> {
    let output = run("findmnt -rno SOURCE,TARGET,SIZE,USED -t ext4,xfs,btrfs,f2fs -e /,/boot,/snap")?;
    let mut parts = Vec::with_capacity(3);
    let mut seen = HashSet::new();
    
    for line in output.lines().take(3) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 { continue; }
        
        let device = fields[0];
        if seen.contains(device) { continue; }
        seen.insert(device);
        
        if let (Ok(total_bytes), Ok(used_bytes)) = (
            fields[2].parse::<f64>(),
            fields[3].parse::<f64>()
        ) {
            let total = total_bytes / BYTES_TO_GIB;
            let used = used_bytes / BYTES_TO_GIB;
            
            if total > 5.0 {
                let dev_name = device.rsplit('/').next().unwrap_or(device);
                parts.push((dev_name.to_string(), fields[1].to_string(), used, total));
            }
        }
    }
    
    if parts.is_empty() { None } else { Some(parts) }
}

fn get_battery() -> Option<(u8, String)> {
    if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };
            
            if file_name.starts_with("BAT") {
                let capacity = fs::read_to_string(path.join("capacity"))
                    .ok()
                    .and_then(|s| s.trim().parse::<u8>().ok())
                    .unwrap_or(0);
                
                let status = fs::read_to_string(path.join("status"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                return Some((capacity, status));
            }
        }
    }
    None
}

fn get_cpu_temp() -> Option<String> {
    // Check thermal zones first (fast)
    for i in 0..3 {
        let path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(temp) = fs::read_to_string(&path) {
            if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                if millidegrees > 1000 && millidegrees < 150000 {
                    return Some(format!("{}°C", millidegrees / 1000));
                }
            }
        }
    }
    
    // Check hwmon
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name = name.trim().to_lowercase();
                if name.contains("coretemp") || name.contains("k10temp") || name.contains("cpu") {
                    for i in 1..=3 {
                        let temp_file = format!("temp{}_input", i);
                        if let Ok(temp) = fs::read_to_string(path.join(temp_file)) {
                            if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                                if millidegrees > 1000 && millidegrees < 150000 {
                                    return Some(format!("{}°C", millidegrees / 1000));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn get_gpu_temp() -> Option<String> {
    // Check nvidia-smi first (NVIDIA GPUs)
    if let Some(output) = run("nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits 2>/dev/null") {
        if let Ok(temp) = output.trim().parse::<i32>() {
            if temp > 0 && temp < 150 {
                return Some(format!("{}°C", temp));
            }
        }
    }
    
    // Check hwmon for AMD/Intel
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name_lower = name.trim().to_lowercase();
                if name_lower.contains("amdgpu") || name_lower.contains("radeon") || name_lower.contains("i915") {
                    for i in 1..=3 {
                        let temp_file = format!("temp{}_input", i);
                        if let Ok(temp) = fs::read_to_string(path.join(temp_file)) {
                            if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                                if millidegrees > 1000 && millidegrees < 150000 {
                                    return Some(format!("{}°C", millidegrees / 1000));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn get_network() -> Option<String> {
    // Read from /proc/net/route
    if let Ok(routes) = fs::read_to_string("/proc/net/route") {
        for line in routes.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 && fields[1] == "00000000" {
                let iface = fields[0];
                // Get IP quickly
                if let Some(ip) = run(&format!("ip -4 -br addr show {}", iface)) {
                    if let Some(ip_addr) = ip.split_whitespace().nth(2) {
                        let clean_ip = ip_addr.split('/').next().unwrap_or(ip_addr);
                        return Some(format!("{} ({})", clean_ip, iface));
                    }
                }
                break;
            }
        }
    }
    None
}

fn get_display() -> Option<String> {
    // Check display server
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if session_type == "wayland" {
            if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
                return Some(format!("Wayland ({})", wayland_display));
            }
            return Some("Wayland".to_string());
        } else if session_type == "x11" {
            if let Some(output) = run("xrandr --current 2>/dev/null | grep '*'") {
                if let Some(res) = output.split_whitespace()
                    .find(|w| w.contains('x') && w.chars().next().unwrap().is_numeric()) 
                {
                    return Some(format!("{} (X11)", res));
                }
            }
            return Some("X11".to_string());
        }
    }
    
    // Fallback
    if std::env::var("DISPLAY").is_ok() {
        Some("X11".to_string())
    } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("Wayland".to_string())
    } else {
        None
    }
}

fn get_logo(os: &str) -> Vec<String> {
    let ol = os.to_lowercase();
    let lines: &[&str] = if ol.contains("arch") || ol.contains("cachy") {
        &["      /\\      ", "     /  \\     ", "    /\\   \\    ", 
          "   /  \\   \\   ", "  /    \\   \\  ", " /______\\___\\ "]
    } else if ol.contains("ubuntu") {
        &["         _     ", "     ---(_)    ", " _/  ---  \\    ", 
          "(_) |   |      ", "  \\  --- _/    ", "     ---(_)    "]
    } else if ol.contains("debian") {
        &["  _____  ", " /  __ \\ ", "|  /    |", "|  \\___- ", " -_      ", "   --_   "]
    } else if ol.contains("fedora") {
        &["      _____    ", "     /   __)\\  ", "     |  /  \\ \\ ", 
          "  ___|  |__/ / ", " / (_    _)_/  ", "/ /  |  |      "]
    } else {
        &["   ┌─────┐   ", "   │ ● ● │   ", "   │  ◉  │   ", 
          "   │ ─── │   ", "   └─────┘   ", "             "]
    };
    
    lines.iter().map(|&s| s.to_string()).collect()
}
