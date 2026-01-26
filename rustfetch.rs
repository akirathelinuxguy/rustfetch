use std::{
    fs, process::Command, thread, sync::{Arc, Mutex}, io::{self, Write},
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
const SHOW_PACKAGES: bool = true;
const SHOW_SHELL: bool = true;
const SHOW_DE: bool = true;
const SHOW_WM: bool = true;
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

const C_RESET: &str = "\x1b[0m";
const C_BOLD: &str = "\x1b[1m";
const C_CYAN: &str = "\x1b[96m";
const C_GREEN: &str = "\x1b[92m";
const C_YELLOW: &str = "\x1b[93m";
const C_BLUE: &str = "\x1b[94m";
const C_MAGENTA: &str = "\x1b[95m";
const C_RED: &str = "\x1b[91m";

const KB_TO_GIB: f64 = 1048576.0;
const BYTES_TO_GIB: f64 = 1073741824.0;

#[derive(Default, Clone)]
struct Info {
    user: Option<String>, hostname: Option<String>, os: Option<String>,
    kernel: Option<String>, uptime: Option<String>, boot_time: Option<String>,
    packages: Option<String>, shell: Option<String>, de: Option<String>, 
    wm: Option<String>, terminal: Option<String>, cpu: Option<String>,
    cpu_temp: Option<String>, gpu: Option<Vec<String>>, gpu_temp: Option<String>,
    memory: Option<(f64, f64)>, swap: Option<(f64, f64)>,
    disks_detailed: Option<Vec<(String, f64, f64, f64, String)>>,
    partitions: Option<Vec<(String, String, f64, f64)>>,
    network: Option<String>, display: Option<String>,
    battery: Option<(u8, String)>,
}

struct Cache { data: HashMap<String, String> }

impl Cache {
    fn load() -> Self {
        if !CACHE_ENABLED { return Cache { data: HashMap::new() }; }
        let data = fs::read_to_string(CACHE_FILE).ok()
            .map(|c| c.lines().filter_map(|l| l.split_once('=')
                .map(|(k, v)| (k.to_string(), v.to_string()))).collect())
            .unwrap_or_default();
        Cache { data }
    }
    fn get(&self, key: &str) -> Option<String> { self.data.get(key).cloned() }
    fn save(&self) {
        if !CACHE_ENABLED { return; }
        let content: String = self.data.iter()
            .map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("\n");
        let _ = fs::write(CACHE_FILE, content);
    }
}

fn main() {
    let cache = Arc::new(Mutex::new(Cache::load()));
    let info = Arc::new(Mutex::new(Info::default()));
    let logo = Arc::new(Mutex::new(Vec::new()));

    if PROGRESSIVE_DISPLAY {
        progressive_display(cache, info, logo);
    } else {
        static_display(cache, info, logo);
    }
}

fn progressive_display(cache: Arc<Mutex<Cache>>, info: Arc<Mutex<Info>>, logo: Arc<Mutex<Vec<String>>>) {
    let (info_c, logo_c) = (Arc::clone(&info), Arc::clone(&logo));
    
    let display_thread = thread::spawn(move || {
        let mut last_lines = 0;
        loop {
            thread::sleep(Duration::from_millis(8));
            if let (Ok(i), Ok(l)) = (info_c.lock(), logo_c.lock()) {
                if last_lines > 0 { print!("\x1b[{}A\x1b[J", last_lines); }
                last_lines = display_info(&i, &l);
                let _ = io::stdout().flush();
                if check_loaded(&i) { break; }
            }
        }
    });

    let handles = vec![
        spawn(&info, &cache, &logo, |i, c| {
            gather_basic_info(i, c);
            gather_system_info(i, c);
        }),
        spawn(&info, &cache, &logo, gather_hardware_info),
        spawn(&info, &cache, &logo, gather_resources),
    ];

    for h in handles { let _ = h.join(); }
    if let Ok(c) = cache.lock() { c.save(); }
    let _ = display_thread.join();
}

fn static_display(cache: Arc<Mutex<Cache>>, info: Arc<Mutex<Info>>, logo: Arc<Mutex<Vec<String>>>) {
    if let Ok(c) = cache.lock() {
        if let Ok(mut i) = info.lock() {
            gather_basic_info(&mut i, &c);
            gather_system_info(&mut i, &c);
            gather_hardware_info(&mut i, &c);
            gather_resources(&mut i, &c);
            
            if let Ok(mut l) = logo.lock() {
                *l = get_logo(i.os.as_ref().map(|s| s.as_str()).unwrap_or(""));
            }
        }
        c.save();
    }
    if let (Ok(i), Ok(l)) = (info.lock(), logo.lock()) { display_info(&i, &l); }
}

fn gather_basic_info(info: &mut Info, cache: &Cache) {
    info.user = Some(std::env::var("USER").or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string()));
    info.hostname = Some(cache_or(&cache, "host", || 
        fs::read_to_string("/etc/hostname").ok()
            .or_else(|| run("hostname -s"))
            .unwrap_or_else(|| "unknown".to_string()).trim().to_string()));
    info.os = Some(cache_or(&cache, "os", || 
        fs::read_to_string("/etc/os-release").ok()
            .and_then(|c| c.lines().find_map(|l| l.strip_prefix("PRETTY_NAME="))
                .map(|s| s.trim_matches('"').to_string()))
            .unwrap_or_else(|| "Unknown OS".to_string())));
}

fn gather_system_info(info: &mut Info, cache: &Cache) {
    if SHOW_KERNEL {
        info.kernel = Some(cache_or(&cache, "kernel", || run("uname -r").unwrap_or_else(|| "Unknown".to_string())));
    }
    if SHOW_UPTIME { info.uptime = Some(get_uptime()); }
    if SHOW_BOOT_TIME { info.boot_time = Some(get_boot_time()); }
    if SHOW_PACKAGES { info.packages = Some(cache_or(&cache, "pkgs", get_packages)); }
    if SHOW_SHELL {
        info.shell = Some(std::env::var("SHELL").ok()
            .and_then(|s| s.rsplit('/').next().map(String::from))
            .unwrap_or_else(|| "unknown".to_string()));
    }
    if SHOW_DE {
        info.de = Some(std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "Unknown".to_string()));
    }
    if SHOW_WM { info.wm = Some(cache_or(&cache, "wm", get_wm)); }
    if SHOW_TERMINAL {
        info.terminal = Some(std::env::var("TERM_PROGRAM")
            .or_else(|_| std::env::var("TERMINAL"))
            .unwrap_or_else(|_| "Unknown".to_string()));
    }
}

fn gather_hardware_info(info: &mut Info, cache: &Cache) {
    if SHOW_CPU { info.cpu = Some(cache_or(&cache, "cpu", get_cpu)); }
    if SHOW_CPU_TEMP { info.cpu_temp = get_cpu_temp(); }
    if SHOW_GPU { info.gpu = Some(cache_or_vec(&cache, "gpu", get_gpu)); }
    if SHOW_GPU_TEMP { info.gpu_temp = get_gpu_temp(); }
}

fn gather_resources(info: &mut Info, _cache: &Cache) {
    if SHOW_MEMORY || SHOW_SWAP {
        let (mem, swap) = get_memory_swap();
        if SHOW_MEMORY { info.memory = mem; }
        if SHOW_SWAP { info.swap = swap; }
    }
    if SHOW_DISKS_DETAILED { info.disks_detailed = get_disks_detailed(); }
    if SHOW_PARTITIONS { info.partitions = get_partitions(); }
    if SHOW_NETWORK { info.network = get_network(); }
    if SHOW_DISPLAY { info.display = get_display(); }
    if SHOW_BATTERY { info.battery = get_battery(); }
}

fn spawn<F>(i: &Arc<Mutex<Info>>, c: &Arc<Mutex<Cache>>, l: &Arc<Mutex<Vec<String>>>, f: F) 
    -> thread::JoinHandle<()>
where F: FnOnce(&mut Info, &Cache) + Send + 'static {
    let (i, c, l) = (Arc::clone(i), Arc::clone(c), Arc::clone(l));
    thread::spawn(move || {
        if let (Ok(mut info), Ok(cache)) = (i.lock(), c.lock()) {
            f(&mut info, &cache);
            if info.os.is_some() {
                if let Ok(mut logo) = l.lock() {
                    *logo = get_logo(info.os.as_ref().unwrap());
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
        && (!SHOW_PACKAGES || i.packages.is_some())
        && (!SHOW_SHELL || i.shell.is_some())
        && (!SHOW_DE || i.de.is_some())
        && (!SHOW_WM || i.wm.is_some())
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
    let empty = PROGRESS_BAR_WIDTH - filled;
    format!("{}{:.1}/{:.1} GiB {}[{}{}] {:.0}%{}",
        color, used, total, C_RESET,
        colorize(&"█".repeat(filled), color),
        colorize(&"░".repeat(empty), C_RESET),
        pct, C_RESET)
}

fn battery_bar(level: u8, status: &str, color: &str) -> String {
    let filled = ((level as f64 / 100.0) * PROGRESS_BAR_WIDTH as f64) as usize;
    let empty = PROGRESS_BAR_WIDTH - filled;
    format!("{}% ({}) [{}{}]{}",
        level, status,
        colorize(&"█".repeat(filled), color),
        colorize(&"░".repeat(empty), C_RESET),
        C_RESET)
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' { in_escape = true; }
        else if in_escape && ch == 'm' { in_escape = false; }
        else if !in_escape { result.push(ch); }
    }
    result
}

fn display_info(info: &Info, logo: &[String]) -> usize {
    let mut lines = Vec::new();
    
    if let (Some(u), Some(h)) = (&info.user, &info.hostname) {
        lines.push(format!("{}{}{}", colorize(u, C_BOLD), colorize("@", C_RESET), colorize(h, C_BOLD)));
        lines.push("─".repeat(u.len() + h.len() + 1));
    }
    
    add(&mut lines, SHOW_OS, &info.os, "OS", C_CYAN);
    add(&mut lines, SHOW_KERNEL, &info.kernel, "Kernel", C_CYAN);
    add(&mut lines, SHOW_UPTIME, &info.uptime, "Uptime", C_CYAN);
    add(&mut lines, SHOW_BOOT_TIME, &info.boot_time, "Boot Time", C_GREEN);
    add(&mut lines, SHOW_PACKAGES, &info.packages, "Packages", C_CYAN);
    add(&mut lines, SHOW_SHELL, &info.shell, "Shell", C_CYAN);
    add_filt(&mut lines, SHOW_DE, &info.de, "DE", C_CYAN);
    add_filt(&mut lines, SHOW_WM, &info.wm, "WM", C_CYAN);
    add_filt(&mut lines, SHOW_TERMINAL, &info.terminal, "Terminal", C_CYAN);
    add(&mut lines, SHOW_CPU, &info.cpu, "CPU", C_GREEN);
    add_filt(&mut lines, SHOW_CPU_TEMP, &info.cpu_temp, "CPU Temp", C_YELLOW);
    
    if SHOW_GPU {
        if let Some(gpus) = &info.gpu {
            for (i, g) in gpus.iter().enumerate() {
                lines.push(if i == 0 { format!("{}: {}", colorize("GPU", C_MAGENTA), g) }
                    else { format!("    {}", g) });
            }
        }
    }
    add_filt(&mut lines, SHOW_GPU_TEMP, &info.gpu_temp, "GPU Temp", C_YELLOW);
    
    if SHOW_MEMORY {
        if let Some((used, total)) = info.memory {
            lines.push(format!("{}: {}", colorize("Memory", C_YELLOW), 
                progress_bar(used, total, C_YELLOW)));
        }
    }
    
    if SHOW_SWAP {
        if let Some((used, total)) = info.swap {
            if total > 0.0 {
                lines.push(format!("{}: {}", colorize("Swap", C_YELLOW), 
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
            let color = if *level > 50 { C_GREEN } else if *level > 20 { C_YELLOW } else { C_RED };
            lines.push(format!("{}: {}", colorize("Battery", color), 
                battery_bar(*level, status, color)));
        }
    }
    
    if SHOW_COLORS {
        lines.push(String::new());
        lines.push(format!("{}███{}███{}███{}███{}███{}███{}███{}███{}",
            "\x1b[40m", "\x1b[41m", "\x1b[42m", "\x1b[43m",
            "\x1b[44m", "\x1b[45m", "\x1b[46m", "\x1b[47m", C_RESET));
    }
    
    let logo_width = logo.iter().map(|s| strip_ansi(s).chars().count()).max().unwrap_or(0);
    let max_lines = logo.len().max(lines.len());
    
    for idx in 0..max_lines {
        let logo_line = if idx < logo.len() {
            let stripped_len = strip_ansi(&logo[idx]).chars().count();
            let padding = logo_width.saturating_sub(stripped_len);
            format!("{}{}", colorize(&logo[idx], C_BLUE), " ".repeat(padding))
        } else { " ".repeat(logo_width) };
        
        let info_line = if idx < lines.len() { &lines[idx] } else { "" };
        println!("{}  {}", logo_line, info_line);
    }
    max_lines
}

fn add(lines: &mut Vec<String>, show: bool, val: &Option<String>, label: &str, color: &str) {
    if show { if let Some(v) = val { lines.push(format!("{}: {}", colorize(label, color), v)); } }
}

fn add_filt(lines: &mut Vec<String>, show: bool, val: &Option<String>, label: &str, color: &str) {
    if show { if let Some(v) = val { if v != "Unknown" { 
        lines.push(format!("{}: {}", colorize(label, color), v)); } } }
}

fn colorize(text: &str, color: &str) -> String {
    if USE_COLOR { format!("{}{}{}", color, text, C_RESET) } else { text.to_string() }
}

fn cache_or<F: Fn() -> String>(cache: &Cache, key: &str, f: F) -> String {
    cache.get(key).unwrap_or_else(f)
}

fn cache_or_vec<F: Fn() -> Vec<String>>(cache: &Cache, key: &str, f: F) -> Vec<String> {
    cache.get(key).map(|v| v.split("||").map(String::from).collect()).unwrap_or_else(f)
}

fn run(cmd: &str) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() { return None; }
    Command::new(parts[0]).args(&parts[1..]).output().ok()
        .and_then(|o| if o.status.success() { 
            String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()) 
        } else { None })
}

fn get_uptime() -> String {
    fs::read_to_string("/proc/uptime").ok()
        .and_then(|c| c.split_whitespace().next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|sec| {
                let (d, h, m) = ((sec / 86400.0) as u64, ((sec % 86400.0) / 3600.0) as u64, 
                    ((sec % 3600.0) / 60.0) as u64);
                match (d, h) {
                    (d, h) if d > 0 => format!("{}d {}h {}m", d, h, m),
                    (_, h) if h > 0 => format!("{}h {}m", h, m),
                    _ => format!("{}m", m),
                }
            })).unwrap_or_else(|| "Unknown".to_string())
}

fn get_boot_time() -> String {
    fs::read_to_string("/proc/stat").ok()
        .and_then(|c| c.lines()
            .find(|l| l.starts_with("btime"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<i64>().ok())
            .map(|ts| {
                let secs_per_day = 86400;
                let secs_per_hour = 3600;
                let secs_per_min = 60;
                
                let days = ts / secs_per_day;
                let remaining = ts % secs_per_day;
                let hours = remaining / secs_per_hour;
                let mins = (remaining % secs_per_hour) / secs_per_min;
                let secs = remaining % secs_per_min;
                
                let years = days / 365;
                let year = 1970 + years;
                let days_in_year = days % 365;
                let month = (days_in_year / 30) + 1;
                let day = (days_in_year % 30) + 1;
                
                format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", 
                    year, month.min(12), day.min(31), hours, mins, secs)
            }))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_packages() -> String {
    let count = try_count("pacman", &["-Qq"])
        .or_else(|| try_count("dpkg-query", &["-f", ".\n", "-W"]))
        .or_else(|| try_count("rpm", &["-qa"]))
        .unwrap_or(0);
    if count > 0 { count.to_string() } else { "Unknown".to_string() }
}

fn try_count(cmd: &str, args: &[&str]) -> Option<usize> {
    Command::new(cmd).args(args).stderr(std::process::Stdio::null()).output().ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.is_empty()).count())
}

fn get_wm() -> String {
    let wms = ["hyprland", "sway", "i3", "bspwm", "awesome", "dwm", "openbox", "xmonad"];
    if let Ok(output) = Command::new("ps").args(&["-e", "-o", "comm="]).output() {
        if let Ok(procs) = String::from_utf8(output.stdout) {
            for wm in &wms {
                if procs.lines().any(|l| l == *wm) {
                    return wm.to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

fn get_cpu() -> String {
    fs::read_to_string("/proc/cpuinfo").ok()
        .and_then(|c| {
            let mut model = None;
            let mut cores = 0;
            for l in c.lines() {
                if model.is_none() && l.starts_with("model name") {
                    model = l.split_once(':').map(|(_, s)| s.trim()
                        .replace("(R)", "").replace("(TM)", "").replace("(tm)", "")
                        .split_whitespace().collect::<Vec<_>>().join(" "));
                }
                if l.starts_with("processor") { cores += 1; }
            }
            model.map(|m| format!("{} ({} cores)", m, cores))
        }).unwrap_or_else(|| "Unknown CPU".to_string())
}

fn get_gpu() -> Vec<String> {
    let mut gpus = Vec::new();
    let mut seen = HashSet::new();
    
    if let Ok(o) = Command::new("lspci").stderr(std::process::Stdio::null()).output() {
        if let Ok(s) = String::from_utf8(o.stdout) {
            for l in s.lines() {
                if l.contains("VGA") || l.contains("3D controller") {
                    if let Some(name) = l.split(": ").nth(1) {
                        let clean = name.split(" (rev").next().unwrap_or(name)
                            .replace("Corporation ", "").trim().to_string();
                        let key = clean.to_lowercase();
                        if seen.insert(key) { 
                            gpus.push(clean); 
                            if gpus.len() >= 3 { break; }
                        }
                    }
                }
            }
        }
    }
    if gpus.is_empty() { gpus.push("No GPU detected".to_string()); }
    gpus
}

fn get_memory_swap() -> (Option<(f64, f64)>, Option<(f64, f64)>) {
    let (mut mt, mut ma, mut st, mut sf) = (None, None, None, None);
    if let Ok(c) = fs::read_to_string("/proc/meminfo") {
        for l in c.lines() {
            if mt.is_none() && l.starts_with("MemTotal:") {
                mt = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if ma.is_none() && l.starts_with("MemAvailable:") {
                ma = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if st.is_none() && l.starts_with("SwapTotal:") {
                st = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            } else if sf.is_none() && l.starts_with("SwapFree:") {
                sf = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok());
            }
            if mt.is_some() && ma.is_some() && st.is_some() && sf.is_some() { break; }
        }
    }
    let mem = if let (Some(t), Some(a)) = (mt, ma) {
        Some(((t - a) as f64 / KB_TO_GIB, t as f64 / KB_TO_GIB))
    } else { None };
    let swap = if let (Some(t), Some(f)) = (st, sf) {
        if t > 0 { Some(((t - f) as f64 / KB_TO_GIB, t as f64 / KB_TO_GIB)) }
        else { None }
    } else { None };
    (mem, swap)
}

fn get_disks_detailed() -> Option<Vec<(String, f64, f64, f64, String)>> {
    let output = run("lsblk -bdno NAME,SIZE,TYPE,ROTA -e 7 -e 11")?;
    let mut disks = Vec::new();
    
    for line in output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 { continue; }
        
        let name = fields[0];
        let size_bytes = fields[1].parse::<f64>().ok()?;
        let disk_type = fields[2];
        let is_rotational = fields[3] == "1";
        
        // Skip if not a disk
        if disk_type != "disk" { continue; }
        
        let size_gib = size_bytes / BYTES_TO_GIB;
        
        // Get used space for this disk by summing all partitions
        let used_gib = if let Some(used_output) = run(&format!("lsblk -bno NAME,FSUSED /dev/{}", name)) {
            used_output.lines()
                .filter_map(|l| {
                    let parts: Vec<&str> = l.split_whitespace().collect();
                    if parts.len() >= 2 {
                        parts[1].parse::<f64>().ok()
                    } else {
                        None
                    }
                })
                .sum::<f64>() / BYTES_TO_GIB
        } else {
            0.0
        };
        
        // Determine disk type label
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
        
        if disks.len() >= 4 { break; }
    }
    
    if disks.is_empty() { None } else { Some(disks) }
}

fn get_partitions() -> Option<Vec<(String, String, f64, f64)>> {
    let output = run("findmnt -rno SOURCE,TARGET,SIZE,USED -t ext4,xfs,btrfs,f2fs,ntfs,exfat,vfat")?;
    let mut parts: Vec<(String, String, f64, f64)> = Vec::new();
    let mut seen_devices = HashSet::new();
    
    for line in output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 { continue; }
        
        let device = fields[0];
        let mount = fields[1];
        
        // Skip root and system mounts
        if mount == "/" 
            || mount.starts_with("/boot") 
            || mount.starts_with("/snap") 
            || mount.starts_with("/run") 
            || mount.starts_with("/sys") 
            || mount.starts_with("/proc") 
            || mount.starts_with("/dev")
            || mount.starts_with("/var/lib")
            || mount.starts_with("/var/snap")
            || mount.starts_with("/var/log")
            || mount.starts_with("/var/tmp")
            || mount.starts_with("/var/cache")
            || mount.starts_with("/root")
            || mount.starts_with("/srv")
            || mount.starts_with("/tmp")
            || mount.starts_with("/opt")
            || device.contains("[") {  // Skip btrfs subvolumes
            continue;
        }
        
        // Skip duplicate devices
        if seen_devices.contains(device) { continue; }
        
        // Parse sizes (findmnt returns bytes by default)
        if let (Some(total_bytes), Some(used_bytes)) = (
            fields[2].parse::<f64>().ok(),
            fields[3].parse::<f64>().ok()
        ) {
            let total = total_bytes / BYTES_TO_GIB;
            let used = used_bytes / BYTES_TO_GIB;
            
            // Only show partitions > 5 GiB and actually separate from root
            if total > 5.0 {
                let dev_name = device.rsplit('/').next().unwrap_or(device);
                parts.push((dev_name.to_string(), mount.to_string(), used, total));
                seen_devices.insert(device.to_string());
                
                if parts.len() >= 5 { break; }
            }
        }
    }
    
    if parts.is_empty() { None } else { Some(parts) }
}

fn get_battery() -> Option<(u8, String)> {
    fs::read_dir("/sys/class/power_supply").ok()?.find_map(|entry| {
        let e = entry.ok()?;
        let p = e.path();
        let n = p.file_name()?;
        
        if n.to_string_lossy().starts_with("BAT") {
            let cap = fs::read_to_string(p.join("capacity")).ok()?
                .trim().parse::<u8>().ok()?;
            let stat = fs::read_to_string(p.join("status")).ok()?
                .trim().to_string();
            Some((cap, stat))
        } else {
            None
        }
    })
}

fn get_cpu_temp() -> Option<String> {
    // Try hwmon sensors first
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            
            // Check if this is a CPU thermal sensor
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name = name.trim().to_lowercase();
                if name.contains("coretemp") || name.contains("k10temp") || name.contains("zenpower") {
                    // Find temp1_input (package temperature)
                    if let Ok(temp) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                            let celsius = millidegrees / 1000;
                            return Some(format!("{}°C", celsius));
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to thermal_zone
    for i in 0..10 {
        let zone_path = format!("/sys/class/thermal/thermal_zone{}", i);
        if let Ok(zone_type) = fs::read_to_string(format!("{}/type", zone_path)) {
            let zone_type = zone_type.trim().to_lowercase();
            if zone_type.contains("cpu") || zone_type.contains("x86_pkg") || zone_type.contains("acpi") {
                if let Ok(temp) = fs::read_to_string(format!("{}/temp", zone_path)) {
                    if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                        let celsius = millidegrees / 1000;
                        return Some(format!("{}°C", celsius));
                    }
                }
            }
        }
    }
    
    None
}

fn get_gpu_temp() -> Option<String> {
    // NVIDIA GPU temperature
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
        .stderr(std::process::Stdio::null())
        .output() {
        if output.status.success() {
            if let Ok(temp_str) = String::from_utf8(output.stdout) {
                if let Ok(temp) = temp_str.trim().parse::<i32>() {
                    return Some(format!("{}°C", temp));
                }
            }
        }
    }
    
    // AMD/Intel GPU via hwmon
    if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name = name.trim().to_lowercase();
                if name.contains("amdgpu") || name.contains("radeon") || name.contains("i915") {
                    if let Ok(temp) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(millidegrees) = temp.trim().parse::<i32>() {
                            let celsius = millidegrees / 1000;
                            return Some(format!("{}°C", celsius));
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn get_network() -> Option<String> {
    // Read default route to find active interface
    let route_output = run("ip route show default")?;
    let interface = route_output
        .split_whitespace()
        .skip_while(|&w| w != "dev")
        .nth(1)?;
    
    // Get IP address for that interface
    let ip_output = run(&format!("ip -4 addr show {}", interface))?;
    let ip = ip_output
        .lines()
        .find(|l| l.trim().starts_with("inet "))?
        .split_whitespace()
        .nth(1)?
        .split('/')
        .next()?;
    
    Some(format!("{} ({})", ip, interface))
}

fn get_display() -> Option<String> {
    // Check display server
    let session_type = std::env::var("XDG_SESSION_TYPE").ok()?;
    
    // Get resolution
    if session_type == "wayland" {
        // Try wlr-randr for wlroots compositors
        if let Some(output) = run("wlr-randr") {
            for line in output.lines() {
                if line.contains("current") {
                    if let Some(res) = line.split_whitespace()
                        .find(|w| w.contains('x') && w.chars().next().unwrap().is_numeric()) {
                        let res_clean = res.trim_end_matches(',');
                        return Some(format!("{} @ Wayland", res_clean));
                    }
                }
            }
        }
        // Fallback for Wayland
        return Some("Wayland".to_string());
    } else {
        // X11 - use xrandr
        if let Some(output) = run("xrandr") {
            for line in output.lines() {
                if line.contains('*') {
                    if let Some(res) = line.split_whitespace()
                        .find(|w| w.contains('x') && w.chars().next().unwrap().is_numeric()) {
                        let rate = line.split_whitespace()
                            .find(|w| w.contains('*'))
                            .and_then(|r| r.trim_end_matches('*').parse::<f32>().ok())
                            .map(|r| format!("{}Hz", r.round() as i32))
                            .unwrap_or_default();
                        return Some(format!("{} @ {} (X11)", res, rate));
                    }
                }
            }
        }
        return Some("X11".to_string());
    }
}

fn get_logo(os: &str) -> Vec<String> {
    let ol = os.to_lowercase();
    let lines = if ol.contains("arch") || ol.contains("cachy") {
        vec!["      /\\      ", "     /  \\     ", "    /\\   \\    ", 
             "   /  \\   \\   ", "  /    \\   \\  ", " /______\\___\\ "]
    } else if ol.contains("ubuntu") {
        vec!["         _     ", "     ---(_)    ", " _/  ---  \\    ", 
             "(_) |   |      ", "  \\  --- _/    ", "     ---(_)    "]
    } else if ol.contains("debian") {
        vec!["  _____  ", " /  __ \\ ", "|  /    |", "|  \\___- ", " -_      ", "   --_   "]
    } else if ol.contains("fedora") {
        vec!["      _____    ", "     /   __)\\  ", "     |  /  \\ \\ ", 
             "  ___|  |__/ / ", " / (_    _)_/  ", "/ /  |  |      "]
    } else {
        vec!["   ______   ", "  /      \\  ", " |  ◉  ◉  | ", 
             " |    >   | ", " |  \\___/ | ", "  \\______/  "]
    };
    lines.into_iter().map(String::from).collect()
}
