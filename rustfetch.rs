use std::{
    fs, process::Command, thread, sync::{Arc, Mutex}, 
    path::Path,
};

// ============================================================================
// CONFIGURATION CONSTANTS
// ============================================================================

const PROGRESSIVE_DISPLAY: bool = false;
const USE_COLOR: bool = true;
const CACHE_ENABLED: bool = true;
const CACHE_FILE: &str = "/tmp/rustfetch_cache";
const PROGRESS_BAR_WIDTH: usize = 20;

// Color scheme selection - now defaults to "classic" for universal appeal
const COLOR_SCHEME: &str = "classic";

// Toggle display of each section
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
const SHOW_DISKS_DETAILED: bool = false; // Disabled - partition info is more useful
const SHOW_PARTITIONS: bool = true;
const SHOW_NETWORK: bool = true;
const SHOW_DISPLAY: bool = true;
const SHOW_BATTERY: bool = true;
const SHOW_COLORS: bool = true;

// ============================================================================
// RGB COLOR SCHEMES
// ============================================================================

struct ColorScheme {
    reset: &'static str,
    bold: &'static str,
    primary: String,
    secondary: String,
    warning: String,
    error: String,
    muted: String,
    color1: String,
    color2: String,
    color3: String,
    color4: String,
    color5: String,
    color6: String,
}

impl ColorScheme {
    fn get() -> Self {
        match COLOR_SCHEME {
            "classic" => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(70, 170, 200),
                secondary: format_rgb(120, 190, 80),
                warning: format_rgb(220, 180, 70),
                error: format_rgb(220, 80, 90),
                muted: format_rgb(150, 150, 150),
                color1: format_rgb(220, 80, 90),
                color2: format_rgb(120, 190, 80),
                color3: format_rgb(220, 180, 70),
                color4: format_rgb(70, 140, 220),
                color5: format_rgb(140, 120, 200),
                color6: format_rgb(70, 170, 200),
            },
            "pastel" => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(100, 180, 200),
                secondary: format_rgb(150, 200, 130),
                warning: format_rgb(230, 200, 120),
                error: format_rgb(230, 130, 130),
                muted: format_rgb(170, 170, 180),
                color1: format_rgb(230, 130, 130),
                color2: format_rgb(150, 200, 130),
                color3: format_rgb(230, 200, 120),
                color4: format_rgb(130, 170, 230),
                color5: format_rgb(180, 160, 210),
                color6: format_rgb(130, 200, 210),
            },
            "gruvbox" => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(131, 165, 152),
                secondary: format_rgb(184, 187, 38),
                warning: format_rgb(250, 189, 47),
                error: format_rgb(251, 73, 52),
                muted: format_rgb(168, 153, 132),
                color1: format_rgb(251, 73, 52),
                color2: format_rgb(184, 187, 38),
                color3: format_rgb(250, 189, 47),
                color4: format_rgb(131, 165, 152),
                color5: format_rgb(211, 134, 155),
                color6: format_rgb(254, 128, 25),
            },
            "nord" => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(136, 192, 208),
                secondary: format_rgb(163, 190, 140),
                warning: format_rgb(235, 203, 139),
                error: format_rgb(191, 97, 106),
                muted: format_rgb(216, 222, 233),
                color1: format_rgb(191, 97, 106),
                color2: format_rgb(163, 190, 140),
                color3: format_rgb(235, 203, 139),
                color4: format_rgb(129, 161, 193),
                color5: format_rgb(180, 142, 173),
                color6: format_rgb(136, 192, 208),
            },
            "dracula" => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(139, 233, 253),
                secondary: format_rgb(80, 250, 123),
                warning: format_rgb(241, 250, 140),
                error: format_rgb(255, 85, 85),
                muted: format_rgb(98, 114, 164),
                color1: format_rgb(255, 85, 85),
                color2: format_rgb(80, 250, 123),
                color3: format_rgb(241, 250, 140),
                color4: format_rgb(98, 114, 164),
                color5: format_rgb(189, 147, 249),
                color6: format_rgb(255, 121, 198),
            },
            _ => ColorScheme {
                reset: "\x1b[0m",
                bold: "\x1b[1m",
                primary: format_rgb(80, 160, 200),
                secondary: format_rgb(100, 180, 100),
                warning: format_rgb(220, 180, 80),
                error: format_rgb(220, 80, 80),
                muted: format_rgb(140, 140, 160),
                color1: format_rgb(220, 80, 80),
                color2: format_rgb(100, 180, 100),
                color3: format_rgb(220, 180, 80),
                color4: format_rgb(80, 120, 200),
                color5: format_rgb(160, 120, 200),
                color6: format_rgb(80, 160, 200),
            },
        }
    }
}

fn format_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

const KB_TO_GIB: f64 = 1024.0 * 1024.0;
const BYTES_TO_GIB: f64 = 1024.0 * 1024.0 * 1024.0;
const MIN_TEMP_MILLIDEGREES: i32 = 1000;
const MAX_TEMP_MILLIDEGREES: i32 = 150_000;
const FILLED_CHAR: char = '█';
const EMPTY_CHAR: char = '░';

// ============================================================================
// SIMPLE JSON SERIALIZATION (NO DEPENDENCIES)
// ============================================================================

trait ToJson {
    fn to_json(&self) -> String;
}

impl ToJson for String {
    fn to_json(&self) -> String {
        format!("\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))
    }
}

impl ToJson for f64 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl ToJson for u8 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl ToJson for u64 {
    fn to_json(&self) -> String {
        self.to_string()
    }
}

impl<T: ToJson> ToJson for Option<T> {
    fn to_json(&self) -> String {
        match self {
            Some(v) => v.to_json(),
            None => "null".to_string(),
        }
    }
}

impl<T: ToJson> ToJson for Vec<T> {
    fn to_json(&self) -> String {
        let items: Vec<String> = self.iter().map(|x| x.to_json()).collect();
        format!("[{}]", items.join(","))
    }
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Default, Clone)]
struct NetworkInfo {
    interface: String,
    ipv4: Option<String>,
    ipv6: Option<String>,
    mac: Option<String>,
    state: String,
    rx_bytes: Option<u64>,
    tx_bytes: Option<u64>,
}

impl ToJson for NetworkInfo {
    fn to_json(&self) -> String {
        let rx = self.rx_bytes.map(|v| v.to_string()).unwrap_or("null".to_string());
        let tx = self.tx_bytes.map(|v| v.to_string()).unwrap_or("null".to_string());
        
        format!(
            "{{\"interface\":{},\"ipv4\":{},\"ipv6\":{},\"mac\":{},\"state\":{},\"rx_bytes\":{},\"tx_bytes\":{}}}",
            self.interface.to_json(),
            self.ipv4.to_json(),
            self.ipv6.to_json(),
            self.mac.to_json(),
            self.state.to_json(),
            rx,
            tx
        )
    }
}

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
    gpu_temps: Option<Vec<Option<String>>>, // Changed to Vec to support multiple GPUs
    memory: Option<(f64, f64)>,
    swap: Option<(f64, f64)>,
    disks: Option<Vec<(String, f64, f64, f64, String)>>,
    partitions: Option<Vec<(String, String, f64, f64)>>,
    network: Option<Vec<NetworkInfo>>,
    display: Option<String>,
    battery: Option<(u8, String)>,
}

impl ToJson for Info {
    fn to_json(&self) -> String {
        let mut parts = vec![];
        
        if let Some(ref v) = self.user {
            parts.push(format!("\"user\":{}", v.to_json()));
        }
        if let Some(ref v) = self.hostname {
            parts.push(format!("\"hostname\":{}", v.to_json()));
        }
        if let Some(ref v) = self.os {
            parts.push(format!("\"os\":{}", v.to_json()));
        }
        if let Some(ref v) = self.kernel {
            parts.push(format!("\"kernel\":{}", v.to_json()));
        }
        if let Some(ref v) = self.uptime {
            parts.push(format!("\"uptime\":{}", v.to_json()));
        }
        if let Some(ref v) = self.boot_time {
            parts.push(format!("\"boot_time\":{}", v.to_json()));
        }
        if let Some(ref v) = self.bootloader {
            parts.push(format!("\"bootloader\":{}", v.to_json()));
        }
        if let Some(ref v) = self.packages {
            parts.push(format!("\"packages\":{}", v.to_json()));
        }
        if let Some(ref v) = self.shell {
            parts.push(format!("\"shell\":{}", v.to_json()));
        }
        if let Some(ref v) = self.de {
            parts.push(format!("\"de\":{}", v.to_json()));
        }
        if let Some(ref v) = self.wm {
            parts.push(format!("\"wm\":{}", v.to_json()));
        }
        if let Some(ref v) = self.init {
            parts.push(format!("\"init\":{}", v.to_json()));
        }
        if let Some(ref v) = self.terminal {
            parts.push(format!("\"terminal\":{}", v.to_json()));
        }
        if let Some(ref v) = self.cpu {
            parts.push(format!("\"cpu\":{}", v.to_json()));
        }
        if let Some(ref v) = self.cpu_temp {
            parts.push(format!("\"cpu_temp\":{}", v.to_json()));
        }
        if let Some(ref v) = self.gpu {
            parts.push(format!("\"gpu\":{}", v.to_json()));
        }
        if let Some(ref v) = self.gpu_temps {
            let temps_json: Vec<String> = v.iter().map(|t| t.to_json()).collect();
            parts.push(format!("\"gpu_temps\":[{}]", temps_json.join(",")));
        }
        if let Some((used, total)) = self.memory {
            parts.push(format!("\"memory\":{{\"used\":{},\"total\":{}}}", used, total));
        }
        if let Some((used, total)) = self.swap {
            parts.push(format!("\"swap\":{{\"used\":{},\"total\":{}}}", used, total));
        }
        if let Some(ref v) = self.network {
            parts.push(format!("\"network\":{}", v.to_json()));
        }
        if let Some(ref v) = self.display {
            parts.push(format!("\"display\":{}", v.to_json()));
        }
        if let Some((cap, ref status)) = self.battery {
            parts.push(format!("\"battery\":{{\"capacity\":{},\"status\":{}}}", cap, status.to_json()));
        }
        
        format!("{{{}}}", parts.join(","))
    }
}

// ============================================================================
// CACHE SYSTEM (SIMPLE KEY-VALUE)
// ============================================================================

fn save_cache(info: &Info) {
    if !CACHE_ENABLED {
        return;
    }
    
    let json = info.to_json();
    let _ = fs::write(CACHE_FILE, json);
}

// Cache loading would require a JSON parser - disabled for now
// The cache is still saved for potential future use or external tools

// ============================================================================
// MAIN ENTRY
// ============================================================================

fn main() {
    let info = Arc::new(Mutex::new(Info::default()));
    let mut threads = vec![];
    
    let tasks: Vec<(&str, Box<dyn Fn() -> Box<dyn std::any::Any + Send> + Send>)> = vec![
        ("user_hostname", Box::new(|| Box::new((get_user(), get_hostname())))),
        ("os", Box::new(|| Box::new(get_os()))),
        ("kernel", Box::new(|| Box::new(get_kernel()))),
        ("uptime", Box::new(|| Box::new(get_uptime()))),
        ("boot_time", Box::new(|| Box::new(get_boot_time()))),
        ("bootloader", Box::new(|| Box::new(get_bootloader()))),
        ("packages", Box::new(|| Box::new(get_packages()))),
        ("shell", Box::new(|| Box::new(get_shell()))),
        ("de", Box::new(|| Box::new(get_de()))),
        ("wm", Box::new(|| Box::new(get_wm()))),
        ("init", Box::new(|| Box::new(get_init()))),
        ("terminal", Box::new(|| Box::new(get_terminal()))),
        ("cpu", Box::new(|| Box::new(get_cpu()))),
        ("cpu_temp", Box::new(|| Box::new(get_cpu_temp()))),
        ("gpu", Box::new(|| Box::new(get_gpu()))),
        ("gpu_temps", Box::new(|| Box::new(get_gpu_temp()))),
        ("memory", Box::new(|| Box::new(get_memory()))),
        ("swap", Box::new(|| Box::new(get_swap()))),
        ("disks", Box::new(|| Box::new(get_disks()))),
        ("partitions", Box::new(|| Box::new(get_partitions()))),
        ("network", Box::new(|| Box::new(get_network()))),
        ("display", Box::new(|| Box::new(get_display()))),
        ("battery", Box::new(|| Box::new(get_battery()))),
    ];
    
    for (name, task) in tasks {
        let info_clone = Arc::clone(&info);
        let t = thread::spawn(move || {
            let result = task();
            let mut data = info_clone.lock().unwrap();
            match name {
                "user_hostname" => {
                    if let Some((user, host)) = result.downcast_ref::<(Option<String>, Option<String>)>() {
                        data.user = user.clone();
                        data.hostname = host.clone();
                    }
                }
                "os" => data.os = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "kernel" => data.kernel = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "uptime" => data.uptime = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "boot_time" => data.boot_time = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "bootloader" => data.bootloader = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "packages" => data.packages = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "shell" => data.shell = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "de" => data.de = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "wm" => data.wm = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "init" => data.init = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "terminal" => data.terminal = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "cpu" => data.cpu = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "cpu_temp" => data.cpu_temp = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "gpu" => data.gpu = result.downcast_ref::<Option<Vec<String>>>().cloned().flatten(),
                "gpu_temps" => data.gpu_temps = result.downcast_ref::<Option<Vec<Option<String>>>>().cloned().flatten(),
                "memory" => data.memory = result.downcast_ref::<Option<(f64, f64)>>().cloned().flatten(),
                "swap" => data.swap = result.downcast_ref::<Option<(f64, f64)>>().cloned().flatten(),
                "disks" => data.disks = result.downcast_ref::<Option<Vec<(String, f64, f64, f64, String)>>>().cloned().flatten(),
                "partitions" => data.partitions = result.downcast_ref::<Option<Vec<(String, String, f64, f64)>>>().cloned().flatten(),
                "network" => data.network = result.downcast_ref::<Option<Vec<NetworkInfo>>>().cloned().flatten(),
                "display" => data.display = result.downcast_ref::<Option<String>>().cloned().flatten(),
                "battery" => data.battery = result.downcast_ref::<Option<(u8, String)>>().cloned().flatten(),
                _ => {}
            }
            
            if PROGRESSIVE_DISPLAY {
                render_output(&*data);
            }
        });
        threads.push(t);
    }
    
    for t in threads {
        let _ = t.join();
    }
    
    let final_info = info.lock().unwrap().clone();
    
    if !PROGRESSIVE_DISPLAY {
        render_output(&final_info);
    }
    
    if CACHE_ENABLED {
        save_cache(&final_info);
    }
}

// ============================================================================
// RENDERING
// ============================================================================

fn render_output(info: &Info) {
    let cs = if USE_COLOR { ColorScheme::get() } else { ColorScheme::get() };
    
    let logo_lines = if let Some(ref os) = info.os {
        get_logo(os)
    } else {
        get_logo("unknown")
    };
    
    let mut info_lines = vec![];
    
    if let (Some(ref user), Some(ref host)) = (&info.user, &info.hostname) {
        info_lines.push(format!("{}{}@{}{}", cs.bold, user, host, cs.reset));
        info_lines.push("─".repeat(user.len() + host.len() + 1));
    }
    
    if SHOW_OS {
        if let Some(ref os) = info.os {
            info_lines.push(format!("{}OS:{} {}", cs.primary, cs.reset, os));
        }
    }
    
    if SHOW_KERNEL {
        if let Some(ref kernel) = info.kernel {
            info_lines.push(format!("{}Kernel:{} {}", cs.primary, cs.reset, kernel));
        }
    }
    
    if SHOW_UPTIME {
        if let Some(ref uptime) = info.uptime {
            info_lines.push(format!("{}Uptime:{} {}", cs.primary, cs.reset, uptime));
        }
    }
    
    if SHOW_BOOT_TIME {
        if let Some(ref boot) = info.boot_time {
            info_lines.push(format!("{}Boot Time:{} {}", cs.primary, cs.reset, boot));
        }
    }
    
    if SHOW_BOOTLOADER {
        if let Some(ref bootloader) = info.bootloader {
            info_lines.push(format!("{}Bootloader:{} {}", cs.primary, cs.reset, bootloader));
        }
    }
    
    if SHOW_PACKAGES {
        if let Some(ref packages) = info.packages {
            info_lines.push(format!("{}Packages:{} {}", cs.primary, cs.reset, packages));
        }
    }
    
    if SHOW_SHELL {
        if let Some(ref shell) = info.shell {
            info_lines.push(format!("{}Shell:{} {}", cs.primary, cs.reset, shell));
        }
    }
    
    if SHOW_DE {
        if let Some(ref de) = info.de {
            info_lines.push(format!("{}DE:{} {}", cs.primary, cs.reset, de));
        }
    }
    
    if SHOW_WM {
        if let Some(ref wm) = info.wm {
            info_lines.push(format!("{}WM:{} {}", cs.primary, cs.reset, wm));
        }
    }
    
    if SHOW_INIT {
        if let Some(ref init) = info.init {
            info_lines.push(format!("{}Init:{} {}", cs.primary, cs.reset, init));
        }
    }
    
    if SHOW_TERMINAL {
        if let Some(ref terminal) = info.terminal {
            info_lines.push(format!("{}Terminal:{} {}", cs.primary, cs.reset, terminal));
        }
    }
    
    if SHOW_CPU {
        if let Some(ref cpu) = info.cpu {
            info_lines.push(format!("{}CPU:{} {}", cs.primary, cs.reset, cpu));
        }
    }
    
    if SHOW_CPU_TEMP {
        if let Some(ref temp) = info.cpu_temp {
            info_lines.push(format!("{}CPU Temp:{} {}", cs.primary, cs.reset, temp));
        }
    }
    
    if SHOW_GPU {
        if let Some(ref gpus) = info.gpu {
            let temps = info.gpu_temps.as_ref();
            for (i, gpu) in gpus.iter().enumerate() {
                let temp_str = if let Some(temps_vec) = temps {
                    if let Some(Some(ref temp)) = temps_vec.get(i) {
                        format!(" ({})", temp)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                info_lines.push(format!("{}GPU:{} {}{}", cs.primary, cs.reset, gpu, temp_str));
            }
        }
    }
    
    if SHOW_GPU_TEMP {
        // Temps are now shown inline with GPU info above
    }
    
    if SHOW_MEMORY {
        if let Some((used, total)) = info.memory {
            let percent = (used / total * 100.0) as u8;
            let bar = create_bar(percent, &cs.secondary, &cs.muted);
            info_lines.push(format!("{}Memory:{} {:.1}GiB / {:.1}GiB {}",
                cs.primary, cs.reset, used, total, bar));
        }
    }
    
    if SHOW_SWAP {
        if let Some((used, total)) = info.swap {
            if total > 0.0 {
                let percent = (used / total * 100.0) as u8;
                let bar = create_bar(percent, &cs.warning, &cs.muted);
                info_lines.push(format!("{}Swap:{} {:.1}GiB / {:.1}GiB {}",
                    cs.primary, cs.reset, used, total, bar));
            }
        }
    }
    
    if SHOW_PARTITIONS {
        if let Some(ref parts) = info.partitions {
            for (dev, _mount, used, total) in parts {
                let percent = if *total > 0.0 { (used / total * 100.0) as u8 } else { 0 };
                let bar = create_bar(percent, &cs.secondary, &cs.muted);
                info_lines.push(format!("{}Disk (/):{} {} - {:.1}GiB / {:.1}GiB {}",
                    cs.primary, cs.reset, dev, used, total, bar));
            }
        }
    }
    
    if SHOW_DISKS_DETAILED {
        // Skip showing raw disk info if we already showed partition info
        // Raw disk info is less useful than partition usage
    }
    
    if SHOW_NETWORK {
        if let Some(ref networks) = info.network {
            for net in networks {
                let mut parts = vec![net.interface.clone()];
                
                if let Some(ref ip) = net.ipv4 {
                    parts.push(ip.clone());
                }
                
                if net.state != "UP" {
                    parts.push(format!("({})", net.state));
                }
                
                if let (Some(rx), Some(tx)) = (net.rx_bytes, net.tx_bytes) {
                    parts.push(format!("↓{} ↑{}", format_bytes(rx), format_bytes(tx)));
                }
                
                info_lines.push(format!("{}Network:{} {}", cs.primary, cs.reset, parts.join(" ")));
            }
        }
    }
    
    if SHOW_DISPLAY {
        if let Some(ref display) = info.display {
            info_lines.push(format!("{}Display:{} {}", cs.primary, cs.reset, display));
        }
    }
    
    if SHOW_BATTERY {
        if let Some((capacity, ref status)) = info.battery {
            let bar_color = if capacity > 50 { &cs.secondary } else if capacity > 20 { &cs.warning } else { &cs.error };
            let bar = create_bar(capacity, bar_color, &cs.muted);
            info_lines.push(format!("{}Battery:{} {}% ({}) {}",
                cs.primary, cs.reset, capacity, status, bar));
        }
    }
    
    if SHOW_COLORS {
        info_lines.push(String::new());
        info_lines.push(format!("{}███{}███{}███{}███{}███{}███{}",
            cs.color1, cs.color2, cs.color3, cs.color4, cs.color5, cs.color6, cs.reset));
    }
    
    let max_lines = std::cmp::max(logo_lines.len(), info_lines.len());
    
    for i in 0..max_lines {
        let logo_part = if i < logo_lines.len() {
            format!("{}{}{}", cs.primary, logo_lines[i], cs.reset)
        } else {
            " ".repeat(15)
        };
        
        let info_part = if i < info_lines.len() {
            &info_lines[i]
        } else {
            ""
        };
        
        println!("{}  {}", logo_part, info_part);
    }
}

fn create_bar(percent: u8, filled_color: &str, empty_color: &str) -> String {
    let filled = (percent as usize * PROGRESS_BAR_WIDTH) / 100;
    let empty = PROGRESS_BAR_WIDTH - filled;
    format!("[{}{}{}{}{}]",
        filled_color,
        FILLED_CHAR.to_string().repeat(filled),
        empty_color,
        EMPTY_CHAR.to_string().repeat(empty),
        "\x1b[0m")
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes >= TB {
        format!("{:.1}T", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1}G", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0}K", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

// ============================================================================
// SYSTEM INFO GATHERING
// ============================================================================

fn get_user() -> Option<String> {
    std::env::var("USER").ok()
}

fn get_hostname() -> Option<String> {
    fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
}

fn get_os() -> Option<String> {
    let os_release = fs::read_to_string("/etc/os-release").ok()?;
    
    for line in os_release.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Some(line.split('=').nth(1)?.trim_matches('"').to_string());
        }
    }
    
    None
}

fn get_kernel() -> Option<String> {
    run_cmd("uname", &["-r"])
}

fn get_uptime() -> Option<String> {
    let uptime_str = fs::read_to_string("/proc/uptime").ok()?;
    let seconds = uptime_str.split_whitespace().next()?.parse::<f64>().ok()?;
    
    let days = (seconds / 86400.0) as u64;
    let hours = ((seconds % 86400.0) / 3600.0) as u64;
    let mins = ((seconds % 3600.0) / 60.0) as u64;
    
    if days > 0 {
        Some(format!("{}d {}h {}m", days, hours, mins))
    } else if hours > 0 {
        Some(format!("{}h {}m", hours, mins))
    } else {
        Some(format!("{}m", mins))
    }
}

fn get_boot_time() -> Option<String> {
    let stat = fs::read_to_string("/proc/stat").ok()?;
    
    for line in stat.lines() {
        if line.starts_with("btime ") {
            let timestamp = line.split_whitespace().nth(1)?.parse::<i64>().ok()?;
            
            if let Some(output) = run_cmd("date", &["-d", &format!("@{}", timestamp), "+%Y-%m-%d %H:%M:%S"]) {
                return Some(output);
            }
        }
    }
    
    None
}

fn get_bootloader() -> Option<String> {
    if let Some(output) = run_cmd("efibootmgr", &[]) {
        let lower = output.to_lowercase();
        if lower.contains("grub") {
            return Some("GRUB".to_string());
        } else if lower.contains("systemd") {
            return Some("systemd-boot".to_string());
        } else if lower.contains("refind") {
            return Some("rEFInd".to_string());
        } else if lower.contains("limine") {
            return Some("Limine".to_string());
        }
    }
    
    let systemd_paths = [
        "/boot/efi/loader/loader.conf",
        "/boot/loader/loader.conf",
        "/efi/loader/loader.conf",
    ];
    
    for path in &systemd_paths {
        if Path::new(path).exists() {
            return Some("systemd-boot".to_string());
        }
    }
    
    let grub_paths = [
        "/boot/grub/grub.cfg",
        "/boot/grub2/grub.cfg",
        "/boot/efi/EFI/grub/grub.cfg",
        "/boot/efi/EFI/GRUB/grub.cfg",
        "/boot/efi/EFI/ubuntu/grub.cfg",
        "/boot/efi/EFI/cachyos/grub.cfg",
        "/boot/efi/EFI/arch/grub.cfg",
        "/boot/efi/EFI/fedora/grub.cfg",
        "/boot/efi/EFI/debian/grub.cfg",
    ];
    
    for path in &grub_paths {
        if Path::new(path).exists() {
            return Some("GRUB".to_string());
        }
    }
    
    if Path::new("/boot/efi/EFI/refind/refind.conf").exists() ||
       Path::new("/efi/EFI/refind/refind.conf").exists() {
        return Some("rEFInd".to_string());
    }
    
    let limine_paths = [
        "/boot/limine.cfg",
        "/boot/efi/limine.cfg",
        "/efi/limine.cfg",
        "/boot/limine/limine.cfg",
        "/boot/efi/EFI/limine/limine.cfg",
        "/boot/efi/EFI/BOOT/limine.cfg",
    ];
    
    for path in &limine_paths {
        if Path::new(path).exists() {
            return Some("Limine".to_string());
        }
    }
    
    if Path::new("/etc/lilo.conf").exists() {
        return Some("LILO".to_string());
    }
    
    if Path::new("/boot/syslinux/syslinux.cfg").exists() {
        return Some("Syslinux".to_string());
    }
    
    None
}

fn get_packages() -> Option<String> {
    let mut counts = vec![];
    
    if let Some(count) = run_cmd("pacman", &["-Q"]).map(|s| s.lines().count()) {
        counts.push(format!("{} (pacman)", count));
    }
    if let Some(count) = run_cmd("dpkg", &["-l"]).map(|s| s.lines().filter(|l| l.starts_with("ii")).count()) {
        counts.push(format!("{} (dpkg)", count));
    }
    if let Some(count) = run_cmd("rpm", &["-qa"]).map(|s| s.lines().count()) {
        counts.push(format!("{} (rpm)", count));
    }
    if let Some(count) = run_cmd("flatpak", &["list"]).map(|s| s.lines().count()) {
        if count > 0 {
            counts.push(format!("{} (flatpak)", count));
        }
    }
    if let Some(count) = run_cmd("snap", &["list"]).map(|s| s.lines().count().saturating_sub(1)) {
        if count > 0 {
            counts.push(format!("{} (snap)", count));
        }
    }
    
    if counts.is_empty() {
        None
    } else {
        Some(counts.join(", "))
    }
}

fn get_shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .map(|s| s.rsplit('/').next().unwrap_or(&s).to_string())
}

fn get_de() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP").ok()
        .or_else(|| std::env::var("DESKTOP_SESSION").ok())
}

fn get_wm() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| run_cmd("wmctrl", &["-m"]).and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Name:"))
                .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
        }))
}

fn get_init() -> Option<String> {
    if Path::new("/run/systemd/system").exists() {
        Some("systemd".to_string())
    } else if Path::new("/sbin/openrc").exists() {
        Some("OpenRC".to_string())
    } else if Path::new("/etc/runit").exists() {
        Some("runit".to_string())
    } else {
        None
    }
}

fn get_terminal() -> Option<String> {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("PPid:") {
                if let Some(ppid_str) = line.split_whitespace().nth(1) {
                    let parent_comm_path = format!("/proc/{}/comm", ppid_str);
                    if let Ok(parent_comm) = fs::read_to_string(&parent_comm_path) {
                        let parent = parent_comm.trim();
                        
                        if parent != "sh" && parent != "bash" && parent != "fish" && 
                           parent != "zsh" && parent != "rustfetch" && parent != "dash" {
                            return Some(parent.to_string());
                        }
                        
                        if let Ok(parent_status) = fs::read_to_string(format!("/proc/{}/status", ppid_str)) {
                            for pline in parent_status.lines() {
                                if pline.starts_with("PPid:") {
                                    if let Some(gppid_str) = pline.split_whitespace().nth(1) {
                                        let gparent_comm_path = format!("/proc/{}/comm", gppid_str);
                                        if let Ok(gparent_comm) = fs::read_to_string(&gparent_comm_path) {
                                            let gparent = gparent_comm.trim();
                                            if !gparent.is_empty() && gparent != "systemd" && 
                                               gparent != "init" && !gparent.starts_with("login") {
                                                return Some(gparent.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    std::env::var("TERM").ok()
}

fn get_cpu() -> Option<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    
    let mut cpu_name = String::new();
    let mut thread_count = 0;
    
    for line in cpuinfo.lines() {
        if line.starts_with("processor") {
            thread_count += 1;
        }
        
        if line.starts_with("model name") && cpu_name.is_empty() {
            let name = line.split(':').nth(1)?.trim();
            cpu_name = name.replace("(R)", "")
                           .replace("(TM)", "")
                           .replace("Intel Core", "Intel")
                           .split_whitespace()
                           .filter(|s| !s.is_empty())
                           .collect::<Vec<_>>()
                           .join(" ");
        }
    }
    
    if !cpu_name.is_empty() {
        if thread_count > 0 {
            Some(format!("{} ({})", cpu_name, thread_count))
        } else {
            Some(cpu_name)
        }
    } else {
        None
    }
}

fn get_cpu_temp() -> Option<String> {
    let hwmon_path = Path::new("/sys/class/hwmon");
    let entries = fs::read_dir(hwmon_path).ok()?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        
        let name_file = path.join("name");
        if let Ok(name) = fs::read_to_string(&name_file) {
            let name = name.trim().to_lowercase();
            
            if name.contains("coretemp") || name.contains("k10temp") || 
               name.contains("cpu") || name.contains("zenpower") {
                
                for i in 1..=10 {
                    let temp_file = path.join(format!("temp{}_input", i));
                    if let Ok(temp_str) = fs::read_to_string(&temp_file) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            if temp_millidegrees >= MIN_TEMP_MILLIDEGREES && 
                               temp_millidegrees <= MAX_TEMP_MILLIDEGREES {
                                let temp_c = temp_millidegrees / 1000;
                                return Some(format!("{}°C", temp_c));
                            }
                        }
                    }
                }
            }
        }
    }
    
    None
}

fn get_gpu() -> Option<Vec<String>> {
    let mut gpus = vec![];
    
    if let Some(output) = run_cmd("lspci", &[]) {
        for line in output.lines() {
            let lower = line.to_lowercase();
            
            if lower.contains("bridge") || lower.contains("audio") || lower.contains("usb") {
                continue;
            }
            
            if !((lower.contains("vga") || lower.contains("3d") || 
                  (lower.contains("display") && !lower.contains("audio"))) &&
                 lower.contains("controller")) {
                continue;
            }
            
            if lower.contains("controller:") {
                let desc_start = line.find("controller:").unwrap() + 11;
                let mut desc = line[desc_start..].trim().to_string();
                
                if let Some(rev_pos) = desc.find(" (rev ") {
                    desc = desc[..rev_pos].to_string();
                }
                
                desc = desc
                    .replace("Intel Corporation", "Intel")
                    .replace("Advanced Micro Devices, Inc.", "AMD")
                    .replace("[AMD/ATI]", "AMD")
                    .replace("NVIDIA Corporation", "NVIDIA")
                    .replace("Corporation", "")
                    .trim()
                    .to_string();
                
                if desc.len() > 10 && 
                   !desc.to_lowercase().contains("bridge") &&
                   !desc.starts_with("Device ") {
                    gpus.push(desc);
                }
            }
        }
    }
    
    if gpus.is_empty() { None } else { Some(gpus) }
}

fn get_gpu_temp() -> Option<Vec<Option<String>>> {
    let gpus = get_gpu()?;
    let gpu_count = gpus.len();
    let mut gpu_temps: Vec<Option<String>> = vec![None; gpu_count];
    
    let has_intel = gpus.iter().any(|g| g.to_lowercase().contains("intel"));
    let has_nvidia = gpus.iter().any(|g| g.to_lowercase().contains("nvidia"));
    let has_amd = gpus.iter().any(|g| g.to_lowercase().contains("amd"));
    
    let hwmon_path = Path::new("/sys/class/hwmon");
    
    if let Ok(entries) = fs::read_dir(hwmon_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if let Ok(name) = fs::read_to_string(path.join("name")) {
                let name = name.trim().to_lowercase();
                
                if (name.contains("i915") || name.contains("pch")) && has_intel {
                    if let Ok(temp_str) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            if temp_millidegrees >= MIN_TEMP_MILLIDEGREES && 
                               temp_millidegrees <= MAX_TEMP_MILLIDEGREES {
                                let idx = gpus.iter().position(|g| g.to_lowercase().contains("intel")).unwrap_or(0);
                                gpu_temps[idx] = Some(format!("{}°C", temp_millidegrees / 1000));
                            }
                        }
                    }
                }
                else if name.contains("amdgpu") && has_amd {
                    if let Ok(temp_str) = fs::read_to_string(path.join("temp1_input")) {
                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                            if temp_millidegrees >= MIN_TEMP_MILLIDEGREES && 
                               temp_millidegrees <= MAX_TEMP_MILLIDEGREES {
                                let idx = gpus.iter().position(|g| g.to_lowercase().contains("amd")).unwrap_or(0);
                                gpu_temps[idx] = Some(format!("{}°C", temp_millidegrees / 1000));
                            }
                        }
                    }
                }
            }
        }
    }
    
    if has_nvidia {
        if let Some(output) = run_cmd("nvidia-smi", &["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"]) {
            for line in output.lines() {
                if let Ok(temp) = line.trim().parse::<i32>() {
                    if temp > 0 && temp < 150 {
                        if let Some(idx) = gpus.iter().position(|g| g.to_lowercase().contains("nvidia")) {
                            gpu_temps[idx] = Some(format!("{}°C", temp));
                        }
                        break;
                    }
                }
            }
        }
    }
    
    if gpu_temps.iter().any(|t| t.is_some()) {
        Some(gpu_temps)
    } else {
        None
    }
}

fn get_memory() -> Option<(f64, f64)> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0.0;
    let mut available = 0.0;
    
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1)?.parse::<f64>().ok()? / KB_TO_GIB;
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1)?.parse::<f64>().ok()? / KB_TO_GIB;
        }
    }
    
    if total > 0.0 {
        Some((total - available, total))
    } else {
        None
    }
}

fn get_swap() -> Option<(f64, f64)> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0.0;
    let mut free = 0.0;
    
    for line in meminfo.lines() {
        if line.starts_with("SwapTotal:") {
            total = line.split_whitespace().nth(1)?.parse::<f64>().ok()? / KB_TO_GIB;
        } else if line.starts_with("SwapFree:") {
            free = line.split_whitespace().nth(1)?.parse::<f64>().ok()? / KB_TO_GIB;
        }
    }
    
    if total > 0.0 {
        Some((total - free, total))
    } else {
        None
    }
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn read_file_trim(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

// ============================================================================
// DISK INFORMATION
// ============================================================================

fn get_disks() -> Option<Vec<(String, f64, f64, f64, String)>> {
    let mut disks = vec![];
    let entries = fs::read_dir("/sys/block").ok()?;
    
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        
        if name.starts_with("loop") || name.starts_with("ram") {
            continue;
        }
        
        let size_path = format!("/sys/block/{}/size", name);
        let size_str = read_file_trim(&size_path)?;
        let blocks = size_str.parse::<u64>().ok()?;
        let size_gib = (blocks * 512) as f64 / BYTES_TO_GIB;
        
        if size_gib < 1.0 {
            continue;
        }
        
        let is_rotational = read_file_trim(&format!("/sys/block/{}/queue/rotational", name))
            .and_then(|s| s.parse::<u8>().ok())
            .map(|v| v == 1)
            .unwrap_or(false);
        
        let type_label = if name.starts_with("nvme") {
            "NVME"
        } else if name.starts_with("sd") {
            if is_rotational { "HDD" } else { "SSD" }
        } else if name.starts_with("mmc") {
            "MMC"
        } else if name.starts_with("zram") {
            "SWAP"
        } else {
            "disk"
        };
        
        let used_gib = if name.starts_with("zram") {
            read_file_trim(&format!("/sys/block/{}/mm_stat", name))
                .and_then(|stat| {
                    stat.split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<f64>().ok())
                        .map(|bytes| bytes / BYTES_TO_GIB)
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        
        disks.push((name, size_gib, used_gib, size_gib, type_label.to_string()));
        
        if disks.len() >= 2 { break; }
    }
    
    if disks.is_empty() { None } else { Some(disks) }
}

fn get_partitions() -> Option<Vec<(String, String, f64, f64)>> {
    if let Some(output) = run_cmd("df", &["-hT", "/"]) {
        for line in output.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 6 {
                let source = fields[0];
                let fstype = fields[1];
                let _size = fields[2];
                let _used = fields[3];
                
                if source == "Filesystem" || source == "none" || source == "tmpfs" {
                    continue;
                }
                
                if let (Some(total), Some(used)) = (
                    parse_human_size(fields[2]),
                    parse_human_size(fields[3])
                ) {
                    let dev_name = source.rsplit('/').next().unwrap_or(source);
                    let display = format!("{} - {}", dev_name, fstype);
                    return Some(vec![(display, "/".to_string(), used, total)]);
                }
            }
        }
    }
    
    None
}

fn parse_human_size(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    
    let (num_str, unit) = if s.ends_with('G') || s.ends_with('g') {
        (&s[..s.len()-1], "G")
    } else if s.ends_with('M') || s.ends_with('m') {
        (&s[..s.len()-1], "M")
    } else if s.ends_with('T') || s.ends_with('t') {
        (&s[..s.len()-1], "T")
    } else {
        (s, "")
    };
    
    if let Ok(num) = num_str.parse::<f64>() {
        match unit {
            "G" => Some(num),
            "M" => Some(num / 1024.0),
            "T" => Some(num * 1024.0),
            _ => Some(num / (1024.0 * 1024.0 * 1024.0)),
        }
    } else {
        None
    }
}

fn get_battery() -> Option<(u8, String)> {
    let entries = fs::read_dir("/sys/class/power_supply").ok()?;
    
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name()?.to_string_lossy();
        
        if file_name.starts_with("BAT") {
            let capacity = read_file_trim(&path.join("capacity").to_string_lossy().to_string())
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);
            
            let status = read_file_trim(&path.join("status").to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            
            return Some((capacity, status));
        }
    }
    
    None
}

// ============================================================================
// ENHANCED NETWORK INFORMATION
// ============================================================================

fn get_network() -> Option<Vec<NetworkInfo>> {
    let mut networks = vec![];
    
    let net_dev = fs::read_to_string("/proc/net/dev").ok()?;
    
    for line in net_dev.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 10 {
            continue;
        }
        
        let interface = parts[0].trim_end_matches(':').to_string();
        
        if interface == "lo" {
            continue;
        }
        
        let rx_bytes = parts[1].parse::<u64>().ok();
        let tx_bytes = parts[9].parse::<u64>().ok();
        
        let state = read_file_trim(&format!("/sys/class/net/{}/operstate", interface))
            .unwrap_or_else(|| "unknown".to_string())
            .to_uppercase();
        
        let mac = read_file_trim(&format!("/sys/class/net/{}/address", interface));
        
        let ipv4 = if let Some(output) = run_cmd("ip", &["-4", "addr", "show", &interface]) {
            output.lines()
                .find(|l| l.trim().starts_with("inet "))
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|addr| addr.split('/').next().unwrap_or(addr).to_string())
        } else {
            None
        };
        
        let ipv6 = if let Some(output) = run_cmd("ip", &["-6", "addr", "show", &interface]) {
            output.lines()
                .find(|l| l.trim().starts_with("inet6 ") && !l.contains("::1") && !l.contains("fe80"))
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|addr| addr.split('/').next().unwrap_or(addr).to_string())
        } else {
            None
        };
        
        if ipv4.is_some() || ipv6.is_some() || state == "UP" {
            networks.push(NetworkInfo {
                interface,
                ipv4,
                ipv6,
                mac,
                state,
                rx_bytes,
                tx_bytes,
            });
        }
    }
    
    networks.sort_by(|a, b| {
        match (a.state.as_str(), b.state.as_str()) {
            ("UP", "UP") => a.interface.cmp(&b.interface),
            ("UP", _) => std::cmp::Ordering::Less,
            (_, "UP") => std::cmp::Ordering::Greater,
            _ => a.interface.cmp(&b.interface),
        }
    });
    
    if networks.is_empty() { None } else { Some(networks) }
}

fn get_display() -> Option<String> {
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if session_type == "wayland" {
            if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
                return Some(format!("Wayland ({})", wayland_display));
            }
            return Some("Wayland".to_string());
        } else if session_type == "x11" {
            if let Some(output) = run_cmd("sh", &["-c", "xrandr --current 2>/dev/null | grep '*'"]) {
                if let Some(res) = output.split_whitespace()
                    .find(|w: &&str| w.contains('x') && w.chars().next().unwrap_or('a').is_numeric())
                {
                    return Some(format!("{} (X11)", res));
                }
            }
            return Some("X11".to_string());
        }
    }
    
    if std::env::var("DISPLAY").is_ok() {
        Some("X11".to_string())
    } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("Wayland".to_string())
    } else {
        None
    }
}

// ============================================================================
// ASCII LOGOS
// ============================================================================

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
