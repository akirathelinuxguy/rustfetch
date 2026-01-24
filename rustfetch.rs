use std::{fs, process::Command, thread, sync::{Arc, Mutex}, io::{self, Write}, collections::HashMap};

// ╔══════════════════════════════════════════════════════════════════════════╗
// ║                            CONFIGURATION                                  ║
// ╚══════════════════════════════════════════════════════════════════════════╝

const PROGRESSIVE_DISPLAY: bool = true;      // Fastfetch-style progressive loading
const USE_COLOR: bool = true;                // Colored output
const CACHE_ENABLED: bool = true;            // Cache static info 2-3x faster on reruns
const CACHE_FILE: &str = "/tmp/rustfetch_cache";

// Display toggles - turn off what you don't want
const SHOW_USER_HOST: bool = true;
const SHOW_OS: bool = true;
const SHOW_KERNEL: bool = true;
const SHOW_UPTIME: bool = true;
const SHOW_PACKAGES: bool = true;            // Pacman/dpkg/rpm package count
const SHOW_SHELL: bool = true;
const SHOW_DE: bool = true;                  // Desktop Environment
const SHOW_WM: bool = true;                  // Window Manager
const SHOW_WM_THEME: bool = true;            // WM Theme
const SHOW_TERMINAL: bool = true;
const SHOW_CPU: bool = true;
const SHOW_GPU: bool = true;
const SHOW_MEMORY: bool = true;
const SHOW_SWAP: bool = true;
const SHOW_DISK: bool = true;
const SHOW_DISK_DETAILED: bool = true;       // Show lsblk output
const SHOW_LOCALE: bool = false;
const SHOW_LOCAL_IP: bool = true;
const SHOW_PUBLIC_IP: bool = false;          // Slower, requires internet
const SHOW_BATTERY: bool = true;
const SHOW_COLORS: bool = true;              // Color palette at bottom

// Colors (change as you like)
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

struct Info { user: Option<String>, hostname: Option<String>, os: Option<String>, kernel: Option<String>,
    uptime: Option<String>, packages: Option<String>, shell: Option<String>, de: Option<String>,
    wm: Option<String>, wm_theme: Option<String>, terminal: Option<String>, cpu: Option<String>,
    gpu: Option<Vec<String>>, memory: Option<String>, swap: Option<String>, disk: Option<String>,
    disk_detailed: Option<Vec<String>>, locale: Option<String>, local_ip: Option<String>,
    public_ip: Option<String>, battery: Option<String>, }

impl Info { fn new() -> Self { Info { user: None, hostname: None, os: None, kernel: None, uptime: None,
    packages: None, shell: None, de: None, wm: None, wm_theme: None, terminal: None, cpu: None,
    gpu: None, memory: None, swap: None, disk: None, disk_detailed: None, locale: None,
    local_ip: None, public_ip: None, battery: None, } } }

struct Cache { data: HashMap<String, String> }
impl Cache {
    fn load() -> Self { if !CACHE_ENABLED { return Cache { data: HashMap::new() }; }
        let data = fs::read_to_string(CACHE_FILE).ok().and_then(|c| {
            let mut m = HashMap::new(); for l in c.lines() { if let Some((k, v)) = l.split_once('=') {
                m.insert(k.to_string(), v.to_string()); } } Some(m) }).unwrap_or_default();
        Cache { data } }
    fn get(&self, key: &str) -> Option<String> { self.data.get(key).cloned() }
    fn set(&mut self, key: &str, value: String) { self.data.insert(key.to_string(), value); }
    fn save(&self) { if !CACHE_ENABLED { return; }
        let _ = fs::write(CACHE_FILE, self.data.iter().map(|(k,v)| format!("{}={}",k,v))
            .collect::<Vec<_>>().join("\n")); }
}

fn main() {
    let cache = Arc::new(Mutex::new(Cache::load()));
    let info = Arc::new(Mutex::new(Info::new()));
    let logo = Arc::new(Mutex::new(Vec::new()));

    if PROGRESSIVE_DISPLAY {
        let i = Arc::clone(&info); let l = Arc::clone(&logo);
        let display = thread::spawn(move || { let mut last = 0; loop {
            thread::sleep(std::time::Duration::from_millis(20));
            let ig = i.lock().unwrap(); let lg = l.lock().unwrap();
            if last > 0 { print!("\x1b[{}A\x1b[J", last); }
            last = display_info(&ig, &lg); io::stdout().flush().unwrap();
            if check_loaded(&ig) { break; } } });

        let mut handles = vec![
            { let i = Arc::clone(&info); let c = Arc::clone(&cache);
              thread::spawn(move || { let (u, h, o) = (get_user(), get_cached(&c, "host", get_hostname),
                  get_cached(&c, "os", get_os)); let mut g = i.lock().unwrap();
                  g.user = Some(u); g.hostname = Some(h); g.os = Some(o.clone()); drop(g);
                  *logo.lock().unwrap() = get_logo(&o); }) }
        ];
        if SHOW_KERNEL { let i = Arc::clone(&info); let c = Arc::clone(&cache);
            handles.push(thread::spawn(move || { i.lock().unwrap().kernel = Some(get_cached(&c, "kernel", get_kernel)); })); }
        if SHOW_UPTIME { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().uptime = Some(get_uptime()); })); }
        if SHOW_PACKAGES { let i = Arc::clone(&info); let c = Arc::clone(&cache);
            handles.push(thread::spawn(move || { i.lock().unwrap().packages = Some(get_cached(&c, "pkgs", get_packages)); })); }
        if SHOW_SHELL || SHOW_DE || SHOW_WM || SHOW_WM_THEME || SHOW_TERMINAL {
            let i = Arc::clone(&info); let c = Arc::clone(&cache);
            handles.push(thread::spawn(move || { let mut g = i.lock().unwrap();
                if SHOW_SHELL { g.shell = Some(get_shell()); }
                if SHOW_DE { g.de = Some(get_de()); }
                if SHOW_WM { g.wm = Some(get_cached(&c, "wm", get_wm)); }
                if SHOW_WM_THEME { g.wm_theme = Some(get_wm_theme()); }
                if SHOW_TERMINAL { g.terminal = Some(get_terminal()); } })); }
        if SHOW_CPU { let i = Arc::clone(&info); let c = Arc::clone(&cache);
            handles.push(thread::spawn(move || { i.lock().unwrap().cpu = Some(get_cached(&c, "cpu", get_cpu)); })); }
        if SHOW_GPU { let i = Arc::clone(&info); let c = Arc::clone(&cache);
            handles.push(thread::spawn(move || { i.lock().unwrap().gpu = Some(get_cached_vec(&c, "gpu", get_gpu)); })); }
        if SHOW_MEMORY || SHOW_SWAP { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { let (m, s) = get_memory_swap();
                let mut g = i.lock().unwrap(); if SHOW_MEMORY { g.memory = Some(m); }
                if SHOW_SWAP { g.swap = Some(s); } })); }
        if SHOW_DISK { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().disk = Some(get_disk()); })); }
        if SHOW_DISK_DETAILED { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().disk_detailed = Some(get_lsblk()); })); }
        if SHOW_LOCALE { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().locale = Some(get_locale()); })); }
        if SHOW_LOCAL_IP { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().local_ip = Some(get_local_ip()); })); }
        if SHOW_PUBLIC_IP { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().public_ip = Some(get_public_ip()); })); }
        if SHOW_BATTERY { let i = Arc::clone(&info);
            handles.push(thread::spawn(move || { i.lock().unwrap().battery = Some(get_battery()); })); }

        for h in handles { let _ = h.join(); }
        cache.lock().unwrap().save();
        let _ = display.join();
    } else {
        let (user, host, os) = (get_user(), get_cached(&cache, "host", get_hostname), get_cached(&cache, "os", get_os));
        *logo.lock().unwrap() = get_logo(&os);
        let mut i = info.lock().unwrap();
        i.user = Some(user); i.hostname = Some(host); i.os = Some(os);
        if SHOW_KERNEL { i.kernel = Some(get_cached(&cache, "kernel", get_kernel)); }
        if SHOW_UPTIME { i.uptime = Some(get_uptime()); }
        if SHOW_PACKAGES { i.packages = Some(get_cached(&cache, "pkgs", get_packages)); }
        if SHOW_SHELL { i.shell = Some(get_shell()); }
        if SHOW_DE { i.de = Some(get_de()); }
        if SHOW_WM { i.wm = Some(get_cached(&cache, "wm", get_wm)); }
        if SHOW_WM_THEME { i.wm_theme = Some(get_wm_theme()); }
        if SHOW_TERMINAL { i.terminal = Some(get_terminal()); }
        if SHOW_CPU { i.cpu = Some(get_cached(&cache, "cpu", get_cpu)); }
        if SHOW_GPU { i.gpu = Some(get_cached_vec(&cache, "gpu", get_gpu)); }
        if SHOW_MEMORY || SHOW_SWAP { let (m, s) = get_memory_swap();
            if SHOW_MEMORY { i.memory = Some(m); } if SHOW_SWAP { i.swap = Some(s); } }
        if SHOW_DISK { i.disk = Some(get_disk()); }
        if SHOW_DISK_DETAILED { i.disk_detailed = Some(get_lsblk()); }
        if SHOW_LOCALE { i.locale = Some(get_locale()); }
        if SHOW_LOCAL_IP { i.local_ip = Some(get_local_ip()); }
        if SHOW_PUBLIC_IP { i.public_ip = Some(get_public_ip()); }
        if SHOW_BATTERY { i.battery = Some(get_battery()); }
        drop(i); cache.lock().unwrap().save();
        display_info(&info.lock().unwrap(), &logo.lock().unwrap());
    }
}

fn check_loaded(i: &Info) -> bool {
    i.user.is_some() && i.hostname.is_some() && i.os.is_some() &&
    (!SHOW_KERNEL || i.kernel.is_some()) && (!SHOW_UPTIME || i.uptime.is_some()) &&
    (!SHOW_PACKAGES || i.packages.is_some()) && (!SHOW_SHELL || i.shell.is_some()) &&
    (!SHOW_DE || i.de.is_some()) && (!SHOW_WM || i.wm.is_some()) &&
    (!SHOW_WM_THEME || i.wm_theme.is_some()) && (!SHOW_TERMINAL || i.terminal.is_some()) &&
    (!SHOW_CPU || i.cpu.is_some()) && (!SHOW_GPU || i.gpu.is_some()) &&
    (!SHOW_MEMORY || i.memory.is_some()) && (!SHOW_SWAP || i.swap.is_some()) &&
    (!SHOW_DISK || i.disk.is_some()) && (!SHOW_DISK_DETAILED || i.disk_detailed.is_some()) &&
    (!SHOW_LOCALE || i.locale.is_some()) && (!SHOW_LOCAL_IP || i.local_ip.is_some()) &&
    (!SHOW_PUBLIC_IP || i.public_ip.is_some()) && (!SHOW_BATTERY || i.battery.is_some())
}

fn display_info(i: &Info, logo: &[String]) -> usize {
    let mut lines = Vec::new();
    if let (Some(u), Some(h)) = (&i.user, &i.hostname) {
        lines.push(format!("{}{}{}", c(&u, C_BOLD), c("@", C_RESET), c(&h, C_BOLD)));
        lines.push("─".repeat(u.len() + h.len() + 1)); }
    if SHOW_OS { if let Some(o) = &i.os { lines.push(format!("{}: {}", c("OS", C_CYAN), o)); } }
    if SHOW_KERNEL { if let Some(k) = &i.kernel { lines.push(format!("{}: {}", c("Kernel", C_CYAN), k)); } }
    if SHOW_UPTIME { if let Some(u) = &i.uptime { lines.push(format!("{}: {}", c("Uptime", C_CYAN), u)); } }
    if SHOW_PACKAGES { if let Some(p) = &i.packages { lines.push(format!("{}: {}", c("Packages", C_CYAN), p)); } }
    if SHOW_SHELL { if let Some(s) = &i.shell { lines.push(format!("{}: {}", c("Shell", C_CYAN), s)); } }
    if SHOW_DE { if let Some(d) = &i.de { if d != "Unknown" { lines.push(format!("{}: {}", c("DE", C_CYAN), d)); } } }
    if SHOW_WM { if let Some(w) = &i.wm { if w != "Unknown" { lines.push(format!("{}: {}", c("WM", C_CYAN), w)); } } }
    if SHOW_WM_THEME { if let Some(t) = &i.wm_theme { if t != "Unknown" { lines.push(format!("{}: {}", c("Theme", C_CYAN), t)); } } }
    if SHOW_TERMINAL { if let Some(t) = &i.terminal { if t != "Unknown" { lines.push(format!("{}: {}", c("Terminal", C_CYAN), t)); } } }
    if SHOW_CPU { if let Some(cpu) = &i.cpu { lines.push(format!("{}: {}", c("CPU", C_GREEN), cpu)); } }
    if SHOW_GPU { if let Some(gs) = &i.gpu { for (idx, g) in gs.iter().enumerate() {
        lines.push(if idx == 0 { format!("{}: {}", c("GPU", C_MAGENTA), g) } else { format!("    {}", g) }); } } }
    if SHOW_MEMORY { if let Some(m) = &i.memory { lines.push(format!("{}: {}", c("Memory", C_YELLOW), m)); } }
    if SHOW_SWAP { if let Some(s) = &i.swap { if s != "0 B" { lines.push(format!("{}: {}", c("Swap", C_YELLOW), s)); } } }
    if SHOW_DISK { if let Some(d) = &i.disk { lines.push(format!("{}: {}", c("Disk (/)", C_BLUE), d)); } }
    if SHOW_DISK_DETAILED { if let Some(ds) = &i.disk_detailed { for (idx, d) in ds.iter().enumerate() {
        lines.push(if idx == 0 { format!("{}: {}", c("Disks", C_BLUE), d) } else { format!("       {}", d) }); } } }
    if SHOW_LOCALE { if let Some(l) = &i.locale { if l != "Unknown" { lines.push(format!("{}: {}", c("Locale", C_CYAN), l)); } } }
    if SHOW_LOCAL_IP { if let Some(ip) = &i.local_ip { if ip != "Unknown" { lines.push(format!("{}: {}", c("Local IP", C_GREEN), ip)); } } }
    if SHOW_PUBLIC_IP { if let Some(ip) = &i.public_ip { if ip != "Unknown" { lines.push(format!("{}: {}", c("Public IP", C_GREEN), ip)); } } }
    if SHOW_BATTERY { if let Some(b) = &i.battery { if b != "Unknown" { lines.push(format!("{}: {}", c("Battery", C_RED), b)); } } }
    if SHOW_COLORS { lines.push("".to_string());
        lines.push(format!("{}███{}███{}███{}███{}███{}███{}███{}███{}",
            "\x1b[40m", "\x1b[41m", "\x1b[42m", "\x1b[43m", "\x1b[44m", "\x1b[45m", "\x1b[46m", "\x1b[47m", C_RESET)); }
    
    let lw = logo.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let ml = logo.len().max(lines.len());
    for idx in 0..ml {
        let ll = if idx < logo.len() { format!("{:width$}", c(&logo[idx], C_BLUE), width = lw + 10) } else { " ".repeat(lw + 4) };
        let il = if idx < lines.len() { &lines[idx] } else { "" };
        println!("{}  {}", ll, il); }
    ml
}

#[inline(always)] fn c(t: &str, col: &str) -> String { if USE_COLOR { format!("{}{}{}", col, t, C_RESET) } else { t.to_string() } }
#[inline(always)] fn get_user() -> String { std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "unknown".to_string()) }
fn get_hostname() -> String { fs::read_to_string("/etc/hostname").unwrap_or_else(|_| 
    cmd("hostname").unwrap_or_else(|| "unknown".to_string())).trim().to_string() }
fn get_os() -> String { fs::read_to_string("/etc/os-release").ok().and_then(|c| 
    c.lines().find_map(|l| l.strip_prefix("PRETTY_NAME=")).map(|s| s.trim_matches('"').to_string()))
    .unwrap_or_else(|| "Unknown OS".to_string()) }
fn get_kernel() -> String { cmd("uname -r").unwrap_or_else(|| "Unknown".to_string()) }
fn get_uptime() -> String { fs::read_to_string("/proc/uptime").ok().and_then(|c| 
    c.split_whitespace().next().and_then(|s| s.parse::<f64>().ok().map(|sec| {
        let d = (sec / 86400.0) as u64; let h = ((sec % 86400.0) / 3600.0) as u64;
        let m = ((sec % 3600.0) / 60.0) as u64;
        match (d, h) { (dd, hh) if dd > 0 => format!("{}d {}h {}m", dd, hh, m),
            (_, hh) if hh > 0 => format!("{}h {}m", hh, m), _ => format!("{}m", m) } })))
    .unwrap_or_else(|| "Unknown".to_string()) }
fn get_packages() -> String {
    let mut count = 0;
    if let Ok(o) = Command::new("pacman").arg("-Qq").output() {
        if let Ok(s) = String::from_utf8(o.stdout) { count += s.lines().count(); }
    } else if let Ok(o) = Command::new("dpkg").arg("-l").output() {
        if let Ok(s) = String::from_utf8(o.stdout) { count += s.lines().count().saturating_sub(5); }
    } else if let Ok(o) = Command::new("rpm").arg("-qa").output() {
        if let Ok(s) = String::from_utf8(o.stdout) { count += s.lines().count(); }
    }
    if count > 0 { count.to_string() } else { "Unknown".to_string() }
}
#[inline(always)] fn get_shell() -> String { std::env::var("SHELL").ok().and_then(|s| 
    s.rsplit('/').next().map(String::from)).unwrap_or_else(|| "unknown".to_string()) }
fn get_de() -> String { std::env::var("XDG_CURRENT_DESKTOP").or_else(|_| std::env::var("DESKTOP_SESSION"))
    .unwrap_or_else(|_| if std::env::var("GNOME_DESKTOP_SESSION_ID").is_ok() { "GNOME".to_string() }
    else if std::env::var("KDE_FULL_SESSION").is_ok() { "KDE".to_string() } else { "Unknown".to_string() }) }
fn get_wm() -> String { for wm in &["hyprland", "sway", "i3", "bspwm", "awesome", "dwm", "openbox", "xmonad"] {
    if cmd(&format!("pgrep -x {}", wm)).is_some() { return wm.to_string(); } } "Unknown".to_string() }
fn get_wm_theme() -> String { 
    std::env::var("GTK_THEME").ok().or_else(|| 
        fs::read_to_string(format!("{}/.config/gtk-3.0/settings.ini", std::env::var("HOME").unwrap_or_default()))
        .ok().and_then(|c| c.lines().find_map(|l| l.strip_prefix("gtk-theme-name=").map(String::from))))
    .unwrap_or_else(|| "Unknown".to_string()) 
}
fn get_terminal() -> String { std::env::var("TERM_PROGRAM").or_else(|_| std::env::var("TERMINAL"))
    .unwrap_or_else(|_| "Unknown".to_string()) }
fn get_cpu() -> String { 
    fs::read_to_string("/proc/cpuinfo").ok().and_then(|c| {
        let mut m = None; 
        let mut core_count = 0; 
        
        for l in c.lines() {
            if m.is_none() && l.starts_with("model name") { 
                m = l.split(':').nth(1).map(|s| 
                    s.trim().replace("(R)", "").replace("(TM)", "").replace("  ", " ").trim().to_string()); 
            }
            if l.starts_with("processor") { 
                core_count += 1; 
            }
        }
        
        m.map(|mm| format!("{} ({} cores)", mm, core_count)) 
    }).unwrap_or_else(|| "Unknown CPU".to_string()) 
}
fn get_gpu() -> Vec<String> {
    let mut gpus = Vec::new();
    let mut seen_names = std::collections::HashSet::new();
    
    // Method 1: NVIDIA GPUs via nvidia-smi (most accurate for NVIDIA)
    if let Ok(o) = Command::new("nvidia-smi").args(&["--query-gpu=gpu_name", "--format=csv,noheader"]).output() {
        if o.status.success() { 
            if let Ok(s) = String::from_utf8(o.stdout) {
                for line in s.lines().filter(|l| !l.is_empty()) {
                    let gpu = line.trim().to_string();
                    if seen_names.insert(gpu.to_lowercase()) {
                        gpus.push(format!("NVIDIA {}", gpu));
                    }
                }
            } 
        } 
    }
    
    // Method 2: lspci for ALL GPUs
    if let Ok(o) = Command::new("lspci").output() { 
        if let Ok(s) = String::from_utf8(o.stdout) {
            for l in s.lines() { 
                if l.contains("VGA compatible controller") || l.contains("3D controller") || l.contains("Display controller") { 
                    // Extract the GPU name after the last colon
                    if let Some(after_colon) = l.split(": ").last() {
                        let mut gpu_name = after_colon.trim().to_string();
                        
                        // Remove revision info like (rev 04)
                        if let Some(rev_pos) = gpu_name.find(" (rev ") {
                            gpu_name = gpu_name[..rev_pos].to_string();
                        }
                        
                        // Clean up the name
                        gpu_name = gpu_name
                            .replace("Corporation ", "")
                            .replace("Integrated Graphics Controller", "Graphics")
                            .trim()
                            .to_string();
                        
                        // Skip if it's a duplicate of what nvidia-smi found
                        let normalized = gpu_name.to_lowercase();
                        let is_nvidia_duplicate = gpus.iter().any(|g| {
                            let g_lower = g.to_lowercase();
                            g_lower.contains("nvidia") && normalized.contains("nvidia") &&
                            (g_lower.contains(&normalized.replace("nvidia ", "")) || 
                             normalized.contains(&g_lower.replace("nvidia ", "")))
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
    let (mut mt, mut ma, mut st, mut sf) = (None, None, None, None);
    if let Ok(c) = fs::read_to_string("/proc/meminfo") { for l in c.lines() {
        if mt.is_none() && l.starts_with("MemTotal:") { mt = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()); }
        else if ma.is_none() && l.starts_with("MemAvailable:") { ma = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()); }
        else if st.is_none() && l.starts_with("SwapTotal:") { st = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()); }
        else if sf.is_none() && l.starts_with("SwapFree:") { sf = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()); }
        if mt.is_some() && ma.is_some() && st.is_some() && sf.is_some() { break; } } }
    let mem = if let (Some(t), Some(a)) = (mt, ma) { let u = t - a;
        format!("{:.1} GiB / {:.1} GiB", u as f64 / 1048576.0, t as f64 / 1048576.0) } else { "Unknown".to_string() };
    let swap = if let (Some(t), Some(f)) = (st, sf) { if t > 0 { let u = t - f;
        format!("{:.1} GiB / {:.1} GiB", u as f64 / 1048576.0, t as f64 / 1048576.0) } else { "0 B".to_string() } } else { "0 B".to_string() };
    (mem, swap)
}
fn get_disk() -> String { cmd("df -h /").and_then(|o| o.lines().nth(1).map(|l| {
    let p: Vec<&str> = l.split_whitespace().collect();
    if p.len() >= 5 { format!("{} / {} ({})", p[2], p[1], p[4]) } else { "Unknown".to_string() } }))
    .unwrap_or_else(|| "Unknown".to_string()) }
fn get_lsblk() -> Vec<String> { cmd("lsblk -o NAME,SIZE,TYPE,MOUNTPOINT").map(|o| 
    o.lines().skip(1).map(String::from).collect()).unwrap_or_default() }
fn get_locale() -> String { std::env::var("LANG").or_else(|_| std::env::var("LC_ALL"))
    .unwrap_or_else(|_| "Unknown".to_string()) }
fn get_local_ip() -> String { cmd("hostname -I").map(|s| s.split_whitespace().next()
    .unwrap_or("Unknown").to_string()).unwrap_or_else(|| "Unknown".to_string()) }
fn get_public_ip() -> String { cmd("curl -s ifconfig.me").unwrap_or_else(|| "Unknown".to_string()) }
fn get_battery() -> String { fs::read_dir("/sys/class/power_supply").ok().and_then(|d| {
    for e in d.flatten() { let p = e.path(); if let Some(n) = p.file_name() {
        if n.to_string_lossy().starts_with("BAT") {
            let cap = fs::read_to_string(p.join("capacity")).ok()?.trim().to_string();
            let stat = fs::read_to_string(p.join("status")).ok()?.trim().to_string();
            return Some(format!("{}% ({})", cap, stat)); } } } None })
    .unwrap_or_else(|| "Unknown".to_string()) }

fn get_logo(os: &str) -> Vec<String> {
    let o = os.to_lowercase();
    if o.contains("arch") || o.contains("cachy") { 
        vec!["      /\\      ", "     /  \\     ", "    /\\   \\    ",
             "   /  \\   \\   ", "  /    \\   \\  ", " /______\\___\\ "] 
    } else if o.contains("ubuntu") { 
        vec!["         _     ", "     ---(_)    ", " _/  ---  \\    ",
             "(_) |   |      ", "  \\  --- _/    ", "     ---(_)    "] 
    } else if o.contains("debian") { 
        vec!["  _____  ", " /  __ \\ ", "|  /    |", "|  \\___- ", " -_      ", "   --_   "] 
    } else if o.contains("fedora") { 
        vec!["      _____    ", "     /   __)\\  ", "     |  /  \\ \\ ",
             "  ___|  |__/ / ", " / (_    _)_/  ", "/ /  |  |      "] 
    } else { 
        vec!["   ______   ", "  /      \\  ", " |  ◉  ◉  | ", 
             " |    >   | ", " |  \\___/ | ", "  \\______/  "] 
    }
    .into_iter().map(|s| s.to_string()).collect()
}

fn cmd(c: &str) -> Option<String> { 
    let parts: Vec<&str> = c.split_whitespace().collect();
    Command::new(parts[0]).args(&parts[1..]).output().ok()
        .and_then(|o| if o.status.success() { 
            String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string()) 
        } else { None }) 
}

fn get_cached(cache: &Arc<Mutex<Cache>>, key: &str, f: fn() -> String) -> String {
    let mut c = cache.lock().unwrap(); 
    if let Some(v) = c.get(key) { return v; }
    let v = f(); 
    c.set(key, v.clone()); 
    v 
}

fn get_cached_vec(cache: &Arc<Mutex<Cache>>, key: &str, f: fn() -> Vec<String>) -> Vec<String> {
    let mut c = cache.lock().unwrap(); 
    if let Some(v) = c.get(key) { return v.split("||").map(String::from).collect(); }
    let v = f(); 
    c.set(key, v.join("||")); 
    v 
}
