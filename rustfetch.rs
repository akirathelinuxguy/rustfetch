use std::{
    env,
    fs,
    path::Path,
    process::Command,
    thread,
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
    io::Write,
};

// ============================================================================
// LOGGING CONFIGURATION
// ============================================================================

const LOG_FILE: &str = "/tmp/rustfetch_log";
const LOG_ENABLED: bool = true;

/// Logs a message to the rustfetch log file with timestamp and severity level.
/// This function provides detailed, human-readable logging for debugging and monitoring.
fn log_message(level: &str, category: &str, message: &str) {
    if !LOG_ENABLED {
        return;
    }
    
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                1970 + (secs / 31536000),
                ((secs / 2592000) % 12) + 1,
                ((secs / 86400) % 30) + 1,
                (secs / 3600) % 24,
                (secs / 60) % 60,
                secs % 60
            );
            datetime
        }
        Err(_) => "UNKNOWN_TIME".to_string(),
    };
    
    let log_entry = format!(
        "[{}] [{:7}] [{}] {}\n",
        timestamp, level, category, message
    );
    
    // Try to append to log file, create if it doesn't exist
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

/// Logs an informational message - routine operations and status updates
fn log_info(category: &str, message: &str) {
    log_message("INFO", category, message);
}

/// Logs a warning message - unexpected but non-critical issues
fn log_warn(category: &str, message: &str) {
    log_message("WARNING", category, message);
}

/// Logs an error message - critical failures that prevent normal operation
fn log_error(category: &str, message: &str) {
    log_message("ERROR", category, message);
}

/// Logs a debug message - detailed information for troubleshooting
fn log_debug(category: &str, message: &str) {
    log_message("DEBUG", category, message);
}

// ============================================================================
// VERSION INFO
// ============================================================================

const VERSION: &str = "0.2.0";
const PROGRAM_NAME: &str = "rustfetch";

macro_rules! module {
    ($info_lines:expr, $config_field:expr, $label:expr, $value:expr, $cs:expr) => {
        if $config_field {
            if let Some(ref val) = $value {
                $info_lines.push(format!("{}{}:{} {}", $cs.primary, $label, $cs.reset, val));
            }
        }
    };
}

// ============================================================================
// CLI ARGUMENT PARSING
// ============================================================================

#[derive(Clone)]
struct Config {
    use_color: bool,
    color_scheme: String,
    json_output: bool,
    cache_enabled: bool,
    cache_ttl: u64,
    fast_mode: bool,
    benchmark: bool,
    show_os: bool,
    show_kernel: bool,
    show_uptime: bool,
    show_boot_time: bool,
    show_bootloader: bool,
    show_packages: bool,
    show_shell: bool,
    show_de: bool,
    show_wm: bool,
    show_init: bool,
    show_terminal: bool,
    show_cpu: bool,
    show_cpu_temp: bool,
    show_gpu: bool,
    show_memory: bool,
    show_swap: bool,
    show_partitions: bool,
    show_network: bool,
    show_network_ping: bool,
    show_display: bool,
    show_battery: bool,
    show_colors: bool,
    show_model: bool,
    show_motherboard: bool,
    show_bios: bool,
    show_theme: bool,
    show_icons: bool,
    show_font: bool,
    show_processes: bool,
    show_cpu_freq: bool,
    show_locale: bool,
    show_public_ip: bool,
    show_cpu_cores: bool,
    show_cpu_cache: bool,
    show_gpu_vram: bool,
    show_resolution: bool,
    show_entropy: bool,
    show_users: bool,
    show_failed_units: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_color: true,
            color_scheme: "classic".to_string(),
            json_output: false,
            cache_enabled: true,
            cache_ttl: 60,
            fast_mode: false,
            benchmark: false,
            show_os: true,
            show_kernel: true,
            show_uptime: true,
            show_boot_time: true,
            show_bootloader: true,
            show_packages: true,
            show_shell: true,
            show_de: true,
            show_wm: true,
            show_init: true,
            show_terminal: true,
            show_cpu: true,
            show_cpu_temp: true,
            show_gpu: true,
            show_memory: true,
            show_swap: true,
            show_partitions: true,
            show_network: true,
            show_network_ping: false,
            show_display: true,
            show_battery: true,
            show_colors: true,
            show_model: true,
            show_motherboard: true,
            show_bios: true,
            show_theme: true,
            show_icons: true,
            show_font: true,
            show_processes: true,
            show_cpu_freq: true,
            show_locale: true,
            show_public_ip: false,
            show_cpu_cores: true,
            show_cpu_cache: true,
            show_gpu_vram: true,
            show_resolution: true,
            show_entropy: true,
            show_users: true,
            show_failed_units: true,
        }
    }
}

fn print_help() {
    println!(
        r#"{} {} - A fast system information tool

USAGE:
    {} [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    -j, --json          Output system info as JSON
    -n, --no-color      Disable colored output
    -t, --theme <NAME>  Set color theme (classic, pastel, gruvbox, nord, dracula)
    --no-cache          Disable caching
    --cache-ttl <SEC>   Set cache TTL in seconds (default: 60)
    --fast              Fast mode - skip expensive operations (temps, ping)
    --benchmark         Show timing for each operation
    --network-ping      Enable network ping tests (slower)

MODULES:
    --os / --kernel / --uptime / --boot / --packages
    --cpu / --gpu / --memory / --swap / --disk
    --shell / --terminal / --de / --wm / --init
    --model / --mobo / --bios / --locale / --public-ip
    --desktop-theme / --icons / --font / --resolution / --entropy
    --network / --battery / --users / --failed
    (Most modules enabled by default)

EXAMPLES:
    {}              Show system info with default settings
    {} --fast       Fast mode (~60% faster)
    {} --benchmark  Show performance timing
    {} -t gruvbox   Use gruvbox color theme
    {} --network-ping   Enable network latency tests"#,
        PROGRAM_NAME, VERSION, PROGRAM_NAME, PROGRAM_NAME, PROGRAM_NAME, PROGRAM_NAME, PROGRAM_NAME, PROGRAM_NAME
    );
}

fn parse_args() -> Option<Config> {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();
    
    if env::var("NO_COLOR").is_ok() {
        config.use_color = false;
    }
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return None;
            }
            "-j" | "--json" => {
                config.json_output = true;
                config.use_color = false;
            }
            "-n" | "--no-color" => {
                config.use_color = false;
            }
            "--no-cache" => {
                config.cache_enabled = false;
            }
            "--cache-ttl" => {
                i += 1;
                if i < args.len() {
                    config.cache_ttl = args[i].parse().unwrap_or(60);
                }
            }
            "--fast" => {
                config.fast_mode = true;
                config.show_cpu_temp = false;
                config.show_network_ping = false;
                config.show_public_ip = false;
            }
            "--benchmark" => {
                config.benchmark = true;
            }
            "--network-ping" => {
                config.show_network_ping = true;
            }
            "-t" | "--theme" => {
                i += 1;
                if i < args.len() {
                    let theme = args[i].to_lowercase();
                    match theme.as_str() {
                        "classic" | "pastel" | "gruvbox" | "nord" | "dracula" => {
                            config.color_scheme = theme;
                        }
                        _ => {
                            eprintln!("Unknown theme '{}'. Available: classic, pastel, gruvbox, nord, dracula", args[i]);
                            return None;
                        }
                    }
                } else {
                    eprintln!("Error: --theme requires a theme name");
                    return None;
                }
            }
            "--os" => config.show_os = true,
            "--no-os" => config.show_os = false,
            "--kernel" => config.show_kernel = true,
            "--no-kernel" => config.show_kernel = false,
            "--uptime" => config.show_uptime = true,
            "--no-uptime" => config.show_uptime = false,
            "--boot-time" => config.show_boot_time = true,
            "--no-boot-time" => config.show_boot_time = false,
            "--bootloader" => config.show_bootloader = true,
            "--no-bootloader" => config.show_bootloader = false,
            "--packages" => config.show_packages = true,
            "--no-packages" => config.show_packages = false,
            "--shell" => config.show_shell = true,
            "--no-shell" => config.show_shell = false,
            "--de" => config.show_de = true,
            "--no-de" => config.show_de = false,
            "--wm" => config.show_wm = true,
            "--no-wm" => config.show_wm = false,
            "--init" => config.show_init = true,
            "--no-init" => config.show_init = false,
            "--terminal" => config.show_terminal = true,
            "--no-terminal" => config.show_terminal = false,
            "--cpu" => config.show_cpu = true,
            "--no-cpu" => config.show_cpu = false,
            "--cpu-temp" => config.show_cpu_temp = true,
            "--no-cpu-temp" => config.show_cpu_temp = false,
            "--gpu" => config.show_gpu = true,
            "--no-gpu" => config.show_gpu = false,
            "--memory" => config.show_memory = true,
            "--no-memory" => config.show_memory = false,
            "--swap" => config.show_swap = true,
            "--no-swap" => config.show_swap = false,
            "--disk" | "--partitions" => config.show_partitions = true,
            "--no-disk" | "--no-partitions" => config.show_partitions = false,
            "--network" => config.show_network = true,
            "--no-network" => config.show_network = false,
            "--display" => config.show_display = true,
            "--no-display" => config.show_display = false,
            "--battery" => config.show_battery = true,
            "--no-battery" => config.show_battery = false,
            "--colors" => config.show_colors = true,
            "--no-colors" => config.show_colors = false,
            "--model" => config.show_model = true,
            "--no-model" => config.show_model = false,
            "--mobo" | "--motherboard" => config.show_motherboard = true,
            "--no-mobo" | "--no-motherboard" => config.show_motherboard = false,
            "--bios" => config.show_bios = true,
            "--no-bios" => config.show_bios = false,
            "--desktop-theme" => config.show_theme = true,
            "--no-desktop-theme" => config.show_theme = false,
            "--icons" => config.show_icons = true,
            "--no-icons" => config.show_icons = false,
            "--font" => config.show_font = true,
            "--no-font" => config.show_font = false,
            "--processes" => config.show_processes = true,
            "--no-processes" => config.show_processes = false,
            "--cpu-freq" => config.show_cpu_freq = true,
            "--no-cpu-freq" => config.show_cpu_freq = false,
            "--locale" => config.show_locale = true,
            "--no-locale" => config.show_locale = false,
            "--public-ip" => config.show_public_ip = true,
            "--no-public-ip" => config.show_public_ip = false,
            "--cores" => config.show_cpu_cores = true,
            "--no-cores" => config.show_cpu_cores = false,
            "--cache" => config.show_cpu_cache = true,
            "--no-cache-module" => config.show_cpu_cache = false,
            "--vram" => config.show_gpu_vram = true,
            "--no-vram" => config.show_gpu_vram = false,
            "--resolution" => config.show_resolution = true,
            "--no-resolution" => config.show_resolution = false,
            "--entropy" => config.show_entropy = true,
            "--no-entropy" => config.show_entropy = false,
            "--users" => config.show_users = true,
            "--no-users" => config.show_users = false,
            "--failed" => config.show_failed_units = true,
            "--no-failed" => config.show_failed_units = false,
            
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Try '{} --help' for more information.", PROGRAM_NAME);
                return None;
            }
            _ => {}
        }
        i += 1;
    }
    
    Some(config)
}

// ============================================================================
// CONSTANTS
// ============================================================================

const CACHE_FILE: &str = "/tmp/rustfetch_cache";
const KB_TO_GIB: f64 = 1024.0 * 1024.0;
const MIN_TEMP_MILLIDEGREES: i32 = 1000;
const MAX_TEMP_MILLIDEGREES: i32 = 150_000;
const FILLED_CHAR: char = '█';
const EMPTY_CHAR: char = '░';

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
    fn new(config: &Config) -> Self {
        if !config.use_color {
            return ColorScheme {
                reset: "",
                bold: "",
                primary: String::new(),
                secondary: String::new(),
                warning: String::new(),
                error: String::new(),
                muted: String::new(),
                color1: String::new(),
                color2: String::new(),
                color3: String::new(),
                color4: String::new(),
                color5: String::new(),
                color6: String::new(),
            };
        }

        match config.color_scheme.as_str() {
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

// ============================================================================
// SIMPLE JSON SERIALIZATION
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

impl ToJson for usize {
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
    rx_rate_mbs: Option<f64>,
    tx_rate_mbs: Option<f64>,
    ping: Option<f64>,
    jitter: Option<f64>,
    packet_loss: Option<f64>,
}

impl ToJson for NetworkInfo {
    fn to_json(&self) -> String {
        format!(
            "{{\"interface\":{},\"ipv4\":{},\"ipv6\":{},\"mac\":{},\"state\":{},\"rx_bytes\":{},\"tx_bytes\":{},\"rx_rate_mbs\":{},\"tx_rate_mbs\":{},\"ping\":{},\"jitter\":{},\"packet_loss\":{}}}",
            self.interface.to_json(),
            self.ipv4.to_json(),
            self.ipv6.to_json(),
            self.mac.to_json(),
            self.state.to_json(),
            self.rx_bytes.to_json(),
            self.tx_bytes.to_json(),
            self.rx_rate_mbs.to_json(),
            self.tx_rate_mbs.to_json(),
            self.ping.to_json(),
            self.jitter.to_json(),
            self.packet_loss.to_json(),
        )
    }
}

#[derive(Default, Clone)]
struct CpuInfo {
    name: Option<String>,
    threads: usize,
    cores: Option<usize>,
    cache: Option<String>,
    freq: Option<String>,
}

#[derive(Default, Clone)]
struct Info {
    user: Option<String>,
    hostname: Option<String>,
    os: Option<String>,
    kernel: Option<String>,
    public_ip: Option<String>,
    cpu_cores: Option<(usize, usize)>,
    cpu_cache: Option<String>,
    gpu_vram: Option<Vec<String>>,
    resolution: Option<String>,
    entropy: Option<String>,
    users: Option<usize>,
    failed_units: Option<usize>,
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
    gpu_temps: Option<Vec<Option<String>>>,
    memory: Option<(f64, f64)>,
    swap: Option<(f64, f64)>,
    partitions: Option<Vec<(String, String, f64, f64)>>,
    network: Option<Vec<NetworkInfo>>,
    display: Option<String>,
    battery: Option<(u8, String)>,
    model: Option<String>,
    motherboard: Option<String>,
    bios: Option<String>,
    theme: Option<String>,
    icons: Option<String>,
    font: Option<String>,
    processes: Option<usize>,
    cpu_freq: Option<String>,
    locale: Option<String>,
}

impl ToJson for Info {
    fn to_json(&self) -> String {
        let mut parts = Vec::with_capacity(40);
        
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
        
        if let Some(ref v) = self.model { parts.push(format!("\"model\":{}", v.to_json())); }
        if let Some(ref v) = self.motherboard { parts.push(format!("\"motherboard\":{}", v.to_json())); }
        if let Some(ref v) = self.bios { parts.push(format!("\"bios\":{}", v.to_json())); }
        if let Some(ref v) = self.theme { parts.push(format!("\"theme\":{}", v.to_json())); }
        if let Some(ref v) = self.icons { parts.push(format!("\"icons\":{}", v.to_json())); }
        if let Some(ref v) = self.font { parts.push(format!("\"font\":{}", v.to_json())); }
        if let Some(ref v) = self.processes { parts.push(format!("\"processes\":{}", v.to_json())); }
        if let Some(ref v) = self.cpu_freq { parts.push(format!("\"cpu_freq\":{}", v.to_json())); }
        if let Some(ref v) = self.locale { parts.push(format!("\"locale\":{}", v.to_json())); }
        if let Some(ref v) = self.public_ip { parts.push(format!("\"public_ip\":{}", v.to_json())); }
        
        format!("{{{}}}", parts.join(","))
    }
}

// ============================================================================
// CACHE SYSTEM
// ============================================================================

fn save_cache(info: &Info) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    
    let json = format!("{{\"timestamp\":{},\"data\":{}}}", now, info.to_json());
    let _ = fs::write(CACHE_FILE, json);
}

// ============================================================================
// MAIN ENTRY
// ============================================================================

fn main() {
    log_info("STARTUP", "Rustfetch starting up");
    log_debug("STARTUP", &format!("Version: {}", VERSION));
    
    let config = match parse_args() {
        Some(cfg) => {
            log_info("CONFIG", "Command line arguments parsed successfully");
            log_debug("CONFIG", &format!("Color enabled: {}, Theme: {}, JSON output: {}", 
                cfg.use_color, cfg.color_scheme, cfg.json_output));
            log_debug("CONFIG", &format!("Cache enabled: {}, TTL: {}s, Fast mode: {}", 
                cfg.cache_enabled, cfg.cache_ttl, cfg.fast_mode));
            cfg
        },
        None => {
            log_info("STARTUP", "Help displayed or invalid arguments, exiting normally");
            return;
        }
    };
    
    if config.benchmark {
        log_info("BENCHMARK", "Running in benchmark mode");
        run_benchmarks(&config);
        log_info("BENCHMARK", "Benchmark completed");
        return;
    }
    
    log_info("EXECUTION", "Beginning system information collection");
    let start_time = std::time::Instant::now();
    // Snapshot /proc/net/dev as early as possible for bandwidth delta
    let net_start = if config.show_network { 
        log_debug("NETWORK", "Reading initial network statistics from /proc/net/dev");
        match read_file_trim("/proc/net/dev") {
            Some(data) => {
                log_debug("NETWORK", "Successfully captured initial network state");
                Some(data)
            },
            None => {
                log_warn("NETWORK", "Failed to read /proc/net/dev for network statistics");
                None
            }
        }
    } else { 
        log_debug("NETWORK", "Network display disabled, skipping network stats");
        None 
    };

    log_info("THREADS", "Spawning 5 parallel threads for system information gathering");
    let info = thread::scope(|s| {
        // ── Thread 1: pure env + file reads. ZERO spawns. ──
        log_debug("THREAD1", "Starting Thread 1: Environment and file-based info (user, hostname, OS, kernel, etc.)");
        let cfg1 = config.clone();
        let t1 = s.spawn(move || {
            log_debug("THREAD1", "Collecting user information");
            let user        = get_user();
            if user.is_some() { log_debug("THREAD1", "User information collected successfully"); }
            else { log_warn("THREAD1", "Failed to determine current user"); }
            
            log_debug("THREAD1", "Collecting hostname");
            let hostname    = get_hostname();
            if hostname.is_some() { log_debug("THREAD1", "Hostname collected successfully"); }
            else { log_warn("THREAD1", "Failed to determine hostname"); }
            
            log_debug("THREAD1", "Detecting operating system");
            let os          = get_os();
            if os.is_some() { log_debug("THREAD1", &format!("OS detected: {:?}", os)); }
            else { log_warn("THREAD1", "Failed to detect operating system"); }
            
            log_debug("THREAD1", "Reading kernel version");
            let kernel      = get_kernel();
            if kernel.is_some() { log_debug("THREAD1", &format!("Kernel: {:?}", kernel)); }
            else { log_warn("THREAD1", "Failed to read kernel version"); }
            
            let uptime      = if cfg1.show_uptime    { 
                log_debug("THREAD1", "Calculating system uptime");
                let up = get_uptime();
                if up.is_some() { log_debug("THREAD1", "Uptime calculated successfully"); }
                else { log_warn("THREAD1", "Failed to calculate uptime"); }
                up
            } else { None };
            
            let shell       = if cfg1.show_shell     { 
                log_debug("THREAD1", "Detecting shell");
                let sh = get_shell();
                if sh.is_some() { log_debug("THREAD1", &format!("Shell detected: {:?}", sh)); }
                else { log_warn("THREAD1", "Failed to detect shell"); }
                sh
            } else { None };
            
            let de          = if cfg1.show_de        { 
                log_debug("THREAD1", "Detecting desktop environment");
                let desktop = get_de();
                if desktop.is_some() { log_debug("THREAD1", &format!("DE detected: {:?}", desktop)); }
                else { log_debug("THREAD1", "No desktop environment detected (normal for servers/minimal installs)"); }
                desktop
            } else { None };
            
            let init        = if cfg1.show_init      { 
                log_debug("THREAD1", "Detecting init system");
                get_init()
            } else { None };
            
            let terminal    = if cfg1.show_terminal  { 
                log_debug("THREAD1", "Detecting terminal emulator");
                get_terminal()
            } else { None };
            
            let locale      = if cfg1.show_locale    { 
                log_debug("THREAD1", "Reading locale settings");
                get_locale()
            } else { None };
            
            let model       = if cfg1.show_model     { 
                log_debug("THREAD1", "Reading hardware model information");
                get_model()
            } else { None };
            
            let motherboard = if cfg1.show_motherboard { 
                log_debug("THREAD1", "Reading motherboard information");
                get_motherboard()
            } else { None };
            
            let bios        = if cfg1.show_bios      { 
                log_debug("THREAD1", "Reading BIOS version");
                get_bios()
            } else { None };
            
            log_debug("THREAD1", "Thread 1 completed successfully");
            (user, hostname, os, kernel, uptime, shell, de, init, terminal, locale, model, motherboard, bios)
        });

        // ── Thread 2: cpu, mem+swap (1 read), battery, processes, users, entropy ──
        log_debug("THREAD2", "Starting Thread 2: CPU, memory, battery, and process info");
        let cfg2 = config.clone();
        let t2 = s.spawn(move || {
            log_debug("THREAD2", "Collecting CPU information");
            let cpu_info  = get_cpu_info_combined();
            if cpu_info.name.is_some() { log_debug("THREAD2", &format!("CPU detected: {:?}", cpu_info.name)); }
            else { log_warn("THREAD2", "Failed to detect CPU name"); }
            
            let cpu_temp  = if cfg2.show_cpu_temp && !cfg2.fast_mode { 
                log_debug("THREAD2", "Reading CPU temperature");
                let temp = get_cpu_temp();
                if temp.is_some() { log_debug("THREAD2", &format!("CPU temp: {:?}°C", temp)); }
                else { log_warn("THREAD2", "CPU temperature not available (normal for some systems/VMs)"); }
                temp
            } else { 
                if cfg2.fast_mode { log_debug("THREAD2", "Skipping CPU temperature (fast mode enabled)"); }
                None 
            };
            
            log_debug("THREAD2", "Reading memory and swap information");
            let (memory, swap) = if cfg2.show_memory || cfg2.show_swap { 
                let mem_swap = get_memory_and_swap();
                if mem_swap.0.is_some() { log_debug("THREAD2", "Memory info collected successfully"); }
                else { log_warn("THREAD2", "Failed to read memory information"); }
                mem_swap
            } else { (None, None) };
            
            let battery   = if cfg2.show_battery   { 
                log_debug("THREAD2", "Checking for battery");
                let bat = get_battery();
                if bat.is_some() { log_debug("THREAD2", &format!("Battery found: {:?}", bat)); }
                else { log_debug("THREAD2", "No battery detected (normal for desktops)"); }
                bat
            } else { None };
            
            let processes = if cfg2.show_processes { 
                log_debug("THREAD2", "Counting running processes");
                get_processes()
            } else { None };
            
            let users     = if cfg2.show_users     { 
                log_debug("THREAD2", "Counting logged-in users");
                get_users_count()
            } else { None };
            
            let entropy   = if cfg2.show_entropy   { 
                log_debug("THREAD2", "Reading system entropy");
                get_entropy()
            } else { None };
            
            log_debug("THREAD2", "Thread 2 completed successfully");
            (cpu_info, cpu_temp, memory, swap, battery, processes, users, entropy)
        });

        // ── Thread 3: single lspci -v → gpu names + vram, then gpu temps ──
        log_debug("THREAD3", "Starting Thread 3: GPU detection and information");
        let cfg3 = config.clone();
        let t3 = s.spawn(move || {
            let (gpus, gpu_vram) = if cfg3.show_gpu || cfg3.show_gpu_vram {
                log_debug("THREAD3", "Running lspci to detect GPU(s)");
                let gpu_info = get_gpu_combined();
                if gpu_info.0.is_some() { log_debug("THREAD3", &format!("GPU(s) detected: {:?}", gpu_info.0)); }
                else { log_warn("THREAD3", "No GPU detected or lspci unavailable"); }
                gpu_info
            } else { (None, None) };
            
            let gpu_temps = if cfg3.show_gpu && !cfg3.fast_mode {
                log_debug("THREAD3", "Reading GPU temperature");
                let temps = get_gpu_temp_with_gpus(gpus.as_ref());
                if temps.is_some() { log_debug("THREAD3", &format!("GPU temps: {:?}°C", temps)); }
                else { log_debug("THREAD3", "GPU temperature not available (normal for some GPUs/drivers)"); }
                temps
            } else { 
                if cfg3.fast_mode { log_debug("THREAD3", "Skipping GPU temperature (fast mode enabled)"); }
                None 
            };
            
            log_debug("THREAD3", "Thread 3 completed successfully");
            (gpus, gpu_temps, gpu_vram)
        });

        // ── Thread 4: packages, partitions (statfs), bootloader, wm, failed, theme ──
        log_debug("THREAD4", "Starting Thread 4: Package counts, partitions, bootloader, WM, and theme");
        let cfg4 = config.clone();
        let t4 = s.spawn(move || {
            let packages     = if cfg4.show_packages     { 
                log_debug("THREAD4", "Counting installed packages");
                let pkgs = get_packages();
                if pkgs.is_some() { log_debug("THREAD4", &format!("Packages counted: {:?}", pkgs)); }
                else { log_warn("THREAD4", "Failed to count packages"); }
                pkgs
            } else { None };
            
            let partitions   = if cfg4.show_partitions   { 
                log_debug("THREAD4", "Reading partition information");
                get_partitions_impl()
            } else { None };
            
            let boot_time    = if cfg4.show_boot_time    { 
                log_debug("THREAD4", "Calculating boot time");
                get_boot_time()
            } else { None };
            
            let bootloader   = if cfg4.show_bootloader   { 
                log_debug("THREAD4", "Detecting bootloader");
                get_bootloader()
            } else { None };
            
            let wm           = if cfg4.show_wm           { 
                log_debug("THREAD4", "Detecting window manager");
                let window_mgr = get_wm();
                if window_mgr.is_some() { log_debug("THREAD4", &format!("WM detected: {:?}", window_mgr)); }
                else { log_debug("THREAD4", "No window manager detected (normal for servers)"); }
                window_mgr
            } else { None };
            
            let public_ip    = if cfg4.show_public_ip && !cfg4.fast_mode { 
                log_debug("THREAD4", "Fetching public IP address (may take a moment)");
                let ip = get_public_ip();
                if ip.is_some() { log_debug("THREAD4", "Public IP retrieved"); }
                else { log_warn("THREAD4", "Failed to retrieve public IP (check internet connection)"); }
                ip
            } else { 
                if cfg4.fast_mode { log_debug("THREAD4", "Skipping public IP (fast mode enabled)"); }
                None 
            };
            
            let failed_units = if cfg4.show_failed_units { 
                log_debug("THREAD4", "Checking for failed systemd units");
                get_failed_units()
            } else { None };
            
            let theme_info   = if cfg4.show_theme || cfg4.show_icons || cfg4.show_font {
                log_debug("THREAD4", "Reading desktop theme information");
                get_theme_info()
            } else { ThemeInfo { theme: None, icons: None, font: None } };
            
            log_debug("THREAD4", "Thread 4 completed successfully");
            (packages, partitions, boot_time, bootloader, wm, public_ip, failed_units, theme_info)
        });

        // ── Thread 5: display+resolution (1 xrandr) + prefetch ip for network ──
        log_debug("THREAD5", "Starting Thread 5: Display info and network IP prefetch");
        let cfg5 = config.clone();
        let t5 = s.spawn(move || {
            let (display, resolution) = if cfg5.show_display || cfg5.show_resolution {
                log_debug("THREAD5", "Running xrandr to detect display and resolution");
                let disp_info = get_display_and_resolution();
                if disp_info.0.is_some() || disp_info.1.is_some() { 
                    log_debug("THREAD5", "Display information collected"); 
                } else { 
                    log_debug("THREAD5", "Display info not available (normal for headless/server systems)"); 
                }
                disp_info
            } else { (None, None) };
            
            // Prefetch ip output so network assembly after join has zero extra latency
            let ip_out = if cfg5.show_network { 
                log_debug("THREAD5", "Pre-fetching network IP addresses");
                run_cmd("ip", &["-o", "addr", "show"])
            } else { None };
            
            log_debug("THREAD5", "Thread 5 completed successfully");
            (display, resolution, ip_out)
        });

        // ── join ──
        log_debug("THREADS", "Waiting for all threads to complete");
        let (user, hostname, os, kernel, uptime, shell, de, init, terminal, locale, model, motherboard, bios) = t1.join().unwrap();
        log_debug("THREADS", "Thread 1 joined");
        
        let (cpu_info, cpu_temp, memory, swap, battery, processes, users, entropy) = t2.join().unwrap();
        log_debug("THREADS", "Thread 2 joined");
        
        let (gpu, gpu_temps, gpu_vram) = t3.join().unwrap();
        log_debug("THREADS", "Thread 3 joined");
        
        let (packages, partitions, boot_time, bootloader, wm, public_ip, failed_units, theme_info) = t4.join().unwrap();
        log_debug("THREADS", "Thread 4 joined");
        
        let (display, resolution, ip_out) = t5.join().unwrap();
        log_debug("THREADS", "Thread 5 joined - all threads completed");

        // Network: uses pre-fetched ip output — no spawn on critical path
        log_debug("NETWORK", "Finalizing network statistics");
        let network = if config.show_network {
            let delta = start_time.elapsed().as_secs_f64();
            log_debug("NETWORK", &format!("Network delta time: {:.3}s", delta));
            let net = get_network_final_with_ip(net_start, delta, config.show_network_ping, ip_out);
            if net.is_some() { log_debug("NETWORK", "Network information collected successfully"); }
            else { log_warn("NETWORK", "Failed to collect network information"); }
            net
        } else { None };

        log_info("COLLECTION", "All system information collected successfully");

        Info {
            user, hostname, os, kernel, uptime, shell, de, wm, init, terminal,
            cpu: cpu_info.name,
            cpu_temp,
            cpu_cores: if cpu_info.cores.is_some() && cpu_info.threads > 0 {
                Some((cpu_info.cores.unwrap_or(cpu_info.threads), cpu_info.threads))
            } else { None },
            cpu_cache: cpu_info.cache,
            cpu_freq: cpu_info.freq,
            gpu, gpu_temps, gpu_vram,
            memory, swap, partitions, network, display, battery,
            model, motherboard, bios,
            theme: theme_info.theme, icons: theme_info.icons, font: theme_info.font,
            processes, users, entropy, locale, public_ip, resolution, failed_units,
            boot_time, bootloader, packages,
        }
    });
    
    let elapsed = start_time.elapsed();
    log_info("PERFORMANCE", &format!("Total execution time: {:.3}s", elapsed.as_secs_f64()));
    
    if config.json_output {
        log_debug("OUTPUT", "Rendering output in JSON format");
        println!("{}", info.to_json());
        log_info("OUTPUT", "JSON output rendered successfully");
    } else {
        log_debug("OUTPUT", "Rendering output in standard format");
        render_output(&info, &config);
        log_info("OUTPUT", "Standard output rendered successfully");
    }
    
    // Fire-and-forget cache write — doesn't block exit
    if config.cache_enabled {
        log_debug("CACHE", "Spawning background thread to save cache");
        let info_c = info.clone();
        std::thread::spawn(move || {
            log_debug("CACHE", "Writing cache to disk");
            save_cache(&info_c);
            log_debug("CACHE", "Cache saved successfully");
        });
    } else {
        log_debug("CACHE", "Cache disabled, skipping save");
    }
    
    log_info("SHUTDOWN", "Rustfetch completed successfully");
}

// ============================================================================
// BENCHMARKING
// ============================================================================

fn run_benchmarks(config: &Config) {
    println!("rustfetch {} - Performance Benchmark\n", VERSION);
    
    macro_rules! bench {
        ($name:expr, $func:expr) => {
            let start = std::time::Instant::now();
            let _ = $func;
            let elapsed = start.elapsed();
            println!("{:.<35} {:>10.2?}", $name, elapsed);
        };
    }
    
    bench!("User", get_user());
    bench!("Hostname", get_hostname());
    bench!("OS", get_os());
    bench!("Kernel", get_kernel());
    bench!("Uptime", get_uptime());
    bench!("Boot time", get_boot_time());
    bench!("Bootloader", get_bootloader());
    bench!("Packages", get_packages());
    bench!("Shell", get_shell());
    bench!("DE", get_de());
    bench!("WM", get_wm());
    bench!("Init", get_init());
    bench!("Terminal", get_terminal());
    bench!("CPU (combined)", get_cpu_info_combined());
    bench!("Memory+Swap", get_memory_and_swap());
    bench!("Partitions", get_partitions_impl());
    bench!("Display+Res", get_display_and_resolution());
    bench!("Battery", get_battery());
    bench!("Model", get_model());
    bench!("Motherboard", get_motherboard());
    bench!("BIOS", get_bios());
    bench!("Theme info", get_theme_info());
    bench!("Processes", get_processes());
    bench!("Users", get_users_count());
    bench!("Entropy", get_entropy());
    bench!("Locale", get_locale());
    bench!("Failed units", get_failed_units());
    bench!("GPU+VRAM", get_gpu_combined());
    
    if !config.fast_mode {
        println!("\nExpensive operations (skipped in --fast mode):");
        bench!("CPU temp", get_cpu_temp());
        bench!("Public IP", get_public_ip());
        let (gpus, _) = get_gpu_combined();
        bench!("GPU temps", get_gpu_temp_with_gpus(gpus.as_ref()));
    } else {
        println!("\n(Use without --fast to benchmark expensive operations)");
    }
    
    println!("\nTip: Run 'rustfetch --fast' for ~60% faster execution");
}

// ============================================================================
// RENDERING UTILS
// ============================================================================

fn get_terminal_width() -> usize {
    // $COLUMNS first (shell sets it, fastest)
    if let Some(w) = env::var("COLUMNS").ok().and_then(|s| s.parse::<usize>().ok()) {
        if w > 0 { return w; }
    }
    // ioctl TIOCGWINSZ — zero spawns
    #[repr(C)] struct Winsize { rows: u16, cols: u16, _xp: u16, _yp: u16 }
    extern "C" { fn ioctl(fd: i32, req: u64, ...) -> i32; }
    let mut ws = Winsize { rows: 0, cols: 0, _xp: 0, _yp: 0 };
    if unsafe { ioctl(2, 0x5413, &mut ws) } == 0 && ws.cols > 0 { return ws.cols as usize; }
    80
}

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_ansi = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_ansi = true;
        } else if in_ansi {
            if c.is_ascii_alphabetic() {
                in_ansi = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

fn truncate_ansi(s: &str, max_width: usize) -> String {
    let mut current_width = 0;
    let mut result = String::new();
    let mut in_ansi = false;
    
    for c in s.chars() {
        if c == '\x1b' {
            in_ansi = true;
            result.push(c);
        } else if in_ansi {
            result.push(c);
            if c.is_ascii_alphabetic() {
                in_ansi = false;
            }
        } else {
            if current_width < max_width {
                result.push(c);
                current_width += 1;
            } else {
                break;
            }
        }
    }
    if !result.is_empty() && s.contains('\x1b') {
        result.push_str("\x1b[0m");
    }
    result
}

// ============================================================================
// RENDERING
// ============================================================================

fn render_output(info: &Info, config: &Config) {
    let cs = ColorScheme::new(config);
    let term_width = get_terminal_width();
    
    let logo_lines = if let Some(ref os) = info.os {
        get_logo(os)
    } else {
        get_logo("unknown")
    };
    
    let logo_width = logo_lines.iter().map(|s| visible_len(s.trim_end())).max().unwrap_or(0);
    let available_info_width = term_width.saturating_sub(logo_width + 2).max(60);
    let bar_width = (available_info_width.saturating_sub(40)).clamp(2, 25);
    
    let mut info_lines = Vec::with_capacity(30);
    
    if let (Some(ref user), Some(ref host)) = (&info.user, &info.hostname) {
        let separator = "─".repeat(user.len() + host.len() + 1);
        info_lines.push(format!("{}{}{}@{}", cs.bold, cs.primary, user, host));
        info_lines.push(format!("{}{}{}", cs.muted, separator, cs.reset));
    }
    
    module!(info_lines, config.show_os, "OS", info.os, cs);
    module!(info_lines, config.show_kernel, "Kernel", info.kernel, cs);
    module!(info_lines, config.show_uptime, "Uptime", info.uptime, cs);
    module!(info_lines, config.show_boot_time, "Boot", info.boot_time, cs);
    
    if config.show_failed_units {
        if let Some(failed) = info.failed_units {
            if failed > 0 {
                info_lines.push(format!("{}Failed Units:{} {}", cs.warning, cs.reset, failed));
            }
        }
    }
    
    module!(info_lines, config.show_bootloader, "Bootloader", info.bootloader, cs);
    module!(info_lines, config.show_packages, "Packages", info.packages, cs);
    module!(info_lines, config.show_shell, "Shell", info.shell, cs);
    module!(info_lines, config.show_de, "DE", info.de, cs);
    module!(info_lines, config.show_wm, "WM", info.wm, cs);
    module!(info_lines, config.show_init, "Init", info.init, cs);
    module!(info_lines, config.show_terminal, "Terminal", info.terminal, cs);
    module!(info_lines, config.show_processes, "Processes", info.processes.map(|x| x.to_string()), cs);
    module!(info_lines, config.show_users, "Users", info.users.map(|x| x.to_string()), cs);
    module!(info_lines, config.show_entropy, "Entropy", info.entropy, cs);
    module!(info_lines, config.show_model, "Model", info.model, cs);
    module!(info_lines, config.show_motherboard, "Mobo", info.motherboard, cs);
    module!(info_lines, config.show_bios, "BIOS", info.bios, cs);

    if config.show_cpu {
        if let Some(ref cpu) = info.cpu {
            let mut details = Vec::with_capacity(3);
            if config.show_cpu_freq {
                if let Some(ref f) = info.cpu_freq { details.push(f.clone()); }
            }
            if config.show_cpu_cores {
                if let Some((c, t)) = info.cpu_cores { details.push(format!("{}C/{}T", c, t)); }
            }
            if config.show_cpu_cache {
                if let Some(ref cache) = info.cpu_cache { details.push(format!("{} L3", cache)); }
            }
            
            let detail_str = if details.is_empty() { String::new() } else { format!(" ({})", details.join(", ")) };
            info_lines.push(format!("{}CPU:{} {}{}", cs.primary, cs.reset, cpu, detail_str));
        }
    }
    
    if config.show_cpu_temp {
        if let Some(ref temp) = info.cpu_temp {
            info_lines.push(format!("{}CPU Temp:{} {}", cs.primary, cs.reset, temp));
        }
    }
    
    if config.show_gpu {
        if let Some(ref gpus) = info.gpu {
            let temps = info.gpu_temps.as_ref();
            for (i, gpu) in gpus.iter().enumerate() {
                let mut details = Vec::with_capacity(2);
                if let Some(temps_vec) = temps {
                    if let Some(Some(ref temp)) = temps_vec.get(i) { details.push(temp.clone()); }
                }
                if config.show_gpu_vram {
                    if let Some(ref vram_vec) = info.gpu_vram {
                        if let Some(vram) = vram_vec.get(i) { details.push(vram.clone()); }
                    }
                }
                let detail_str = if details.is_empty() { String::new() } else { format!(" ({})", details.join(", ")) };
                info_lines.push(format!("{}GPU:{} {}{}", cs.primary, cs.reset, gpu, detail_str));
            }
        }
    }
    
    if config.show_memory {
        if let Some((used, total)) = info.memory {
            let percent = ((used / total * 100.0) as u8).min(100);
            let bar = create_bar(percent, &cs.secondary, &cs.muted, config.use_color, bar_width);
            info_lines.push(format!("{}Memory:{} {:.1}GiB / {:.1}GiB {}",
                cs.primary, cs.reset, used, total, bar));
        }
    }
    
    if config.show_swap {
        if let Some((used, total)) = info.swap {
            if total > 0.0 {
                let percent = ((used / total * 100.0) as u8).min(100);
                let bar = create_bar(percent, &cs.warning, &cs.muted, config.use_color, bar_width);
                info_lines.push(format!("{}Swap:{} {:.1}GiB / {:.1}GiB {}",
                    cs.primary, cs.reset, used, total, bar));
            }
        }
    }
    
    if config.show_partitions {
        if let Some(ref parts) = info.partitions {
            for (_, mount, used, total) in parts {
                let percent = if *total > 0.0 { ((used / total * 100.0) as u8).min(100) } else { 0 };
                let bar = create_bar(percent, &cs.secondary, &cs.muted, config.use_color, bar_width);
                info_lines.push(format!("{}Disk ({}):{} {:.1}GiB / {:.1}GiB {}",
                    cs.primary, mount, cs.reset, used, total, bar));
            }
        }
    }
    
    if config.show_network {
        if let Some(ref networks) = info.network {
            for net in networks {
                let mut parts = Vec::with_capacity(4);
                parts.push(net.interface.clone());
                if let Some(ref ip) = net.ipv4 { parts.push(ip.clone()); }
                if let Some(p) = net.ping {
                    let j = net.jitter.map(|j| format!(" | ±{:.1}ms", j)).unwrap_or_default();
                    let l = net.packet_loss.map(|l| format!(" | {:.0}% loss", l)).unwrap_or_default();
                    parts.push(format!("[{:.1}ms{}{}]", p, j, l));
                }
                if let (Some(rx), Some(tx)) = (net.rx_rate_mbs, net.tx_rate_mbs) {
                    if rx > 0.01 || tx > 0.01 { parts.push(format!("↓{:.2}MB/s ↑{:.2}MB/s", rx, tx)); }
                } else if let (Some(rx), Some(tx)) = (net.rx_bytes, net.tx_bytes) {
                    parts.push(format!("↓{} ↑{}", format_bytes(rx), format_bytes(tx)));
                }
                info_lines.push(format!("{}Network:{} {}", cs.primary, cs.reset, parts.join(" ")));
            }
        }
    }

    module!(info_lines, config.show_public_ip, "Public IP", info.public_ip, cs);
    
    if config.show_display {
        if let Some(ref disp) = info.display {
            let res = if config.show_resolution { 
                if let Some(ref r) = info.resolution { 
                    format!(" @ {}", r) 
                } else { 
                    String::new() 
                } 
            } else { 
                String::new() 
            };
            info_lines.push(format!("{}Display:{} {}{}", cs.primary, cs.reset, disp, res));
        }
    }

    module!(info_lines, config.show_locale, "Locale", info.locale, cs);
    module!(info_lines, config.show_theme, "Theme", info.theme, cs);
    module!(info_lines, config.show_icons, "Icons", info.icons, cs);
    module!(info_lines, config.show_font, "Font", info.font, cs);
    
    if config.show_battery {
        if let Some((capacity, ref status)) = info.battery {
            let bar_color = if capacity > 50 { &cs.secondary } else if capacity > 20 { &cs.warning } else { &cs.error };
            let bar = create_bar(capacity, bar_color, &cs.muted, config.use_color, bar_width);
            info_lines.push(format!("{}Battery:{} {}% ({}) {}",
                cs.primary, cs.reset, capacity, status, bar));
        }
    }
    
    if config.show_colors && config.use_color {
        info_lines.push(String::new());
        info_lines.push(format!("{}███{}███{}███{}███{}███{}███{}",
            cs.color1, cs.color2, cs.color3, cs.color4, cs.color5, cs.color6, cs.reset));
    }
    
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut handle = std::io::BufWriter::new(stdout.lock());
    
    let max_lines = std::cmp::max(logo_lines.len(), info_lines.len());
    for i in 0..max_lines {
        let (logo_content, logo_len) = if i < logo_lines.len() {
            (logo_lines[i].as_str(), visible_len(&logo_lines[i]))
        } else {
            ("", 0)
        };
        
        let padding = " ".repeat(logo_width.saturating_sub(logo_len));
        let logo_part = format!("{}{}{}{}", cs.primary, logo_content, cs.reset, padding);
        
        let info_part = if i < info_lines.len() {
            truncate_ansi(&info_lines[i], available_info_width)
        } else {
            String::new()
        };
        
        writeln!(handle, "{}  {}", logo_part, info_part).unwrap_or(());
    }
}

fn create_bar(percent: u8, filled_color: &str, empty_color: &str, use_color: bool, width: usize) -> String {
    let filled = ((percent as usize * width) / 100).min(width);
    let empty = width.saturating_sub(filled);
    
    if use_color {
        format!("[{}{}{}{}{}]",
            filled_color,
            FILLED_CHAR.to_string().repeat(filled),
            empty_color,
            EMPTY_CHAR.to_string().repeat(empty),
            "\x1b[0m")
    } else {
        format!("[{}{}]",
            FILLED_CHAR.to_string().repeat(filled),
            EMPTY_CHAR.to_string().repeat(empty))
    }
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
// SYSTEM INFO GATHERING (OPTIMIZED)
// ============================================================================

fn get_user() -> Option<String> {
    std::env::var("USER").ok()
}

fn get_hostname() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/hostname")
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
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
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
            return Some(format_unix_timestamp(timestamp));
        }
    }
    
    None
}

fn format_unix_timestamp(timestamp: i64) -> String {
    const SECONDS_PER_DAY: i64 = 86400;
    const DAYS_PER_400_YEARS: i64 = 146097;
    const DAYS_SINCE_1970: i64 = 719468;
    
    let days = timestamp / SECONDS_PER_DAY + DAYS_SINCE_1970;
    let time_of_day = timestamp % SECONDS_PER_DAY;
    
    let era = if days >= 0 { days } else { days - 146096 } / DAYS_PER_400_YEARS;
    let doe = (days - era * DAYS_PER_400_YEARS) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    
    let hour = (time_of_day / 3600) % 24;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, m, d, hour, minute, second)
}

fn get_bootloader() -> Option<String> {
    log_debug("BOOTLOADER", "Starting comprehensive bootloader detection");
    
    // ============================================================================
    // METHOD 1: Check EFI Boot Manager entries (Most Reliable for UEFI systems)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking EFI boot manager entries");
    if let Some(output) = run_cmd("efibootmgr", &["-v"]) {
        let lower = output.to_lowercase();
        let lines: Vec<&str> = output.lines().collect();
        
        // Find the current boot entry (marked with *)
        let current_boot = lines.iter()
            .find(|line| line.contains('*'))
            .or_else(|| {
                // If no * found, look for BootCurrent
                if let Some(current_line) = lines.iter().find(|l| l.contains("BootCurrent")) {
                    let boot_num = current_line.split(':').nth(1)?.trim().trim_start_matches('0');
                    lines.iter().find(|l| l.starts_with(&format!("Boot{}", boot_num)))
                } else {
                    lines.first()
                }
            })
            .map(|s| s.to_lowercase());
        
        if let Some(current) = current_boot {
            log_debug("BOOTLOADER", &format!("Current EFI boot entry: {}", current));
            
            // Check current boot entry first (highest priority)
            if current.contains("grub") {
                // Determine GRUB variant from the path
                if current.contains("grub2") {
                    log_info("BOOTLOADER", "Detected GRUB 2 from current EFI boot entry");
                    return Some("GRUB 2".to_string());
                } else {
                    log_info("BOOTLOADER", "Detected GRUB from current EFI boot entry");
                    return Some("GRUB".to_string());
                }
            } else if current.contains("systemd") || current.contains("gummiboot") {
                log_info("BOOTLOADER", "Detected systemd-boot from current EFI boot entry");
                return Some("systemd-boot".to_string());
            } else if current.contains("refind") {
                log_info("BOOTLOADER", "Detected rEFInd from current EFI boot entry");
                return Some("rEFInd".to_string());
            } else if current.contains("limine") {
                log_info("BOOTLOADER", "Detected Limine from current EFI boot entry");
                return Some("Limine".to_string());
            } else if current.contains("clover") {
                log_info("BOOTLOADER", "Detected Clover from current EFI boot entry");
                return Some("Clover".to_string());
            } else if current.contains("opencore") {
                log_info("BOOTLOADER", "Detected OpenCore from current EFI boot entry");
                return Some("OpenCore".to_string());
            } else if current.contains("bootmgfw") || current.contains("windows") {
                // Might be dual boot, continue checking
                log_debug("BOOTLOADER", "Found Windows Boot Manager entry, continuing Linux bootloader detection");
            } else if current.contains("uefi") || current.contains("shell") {
                log_debug("BOOTLOADER", "Found UEFI Shell entry, continuing detection");
            }
        }
        
        // Fallback: Check all entries if current didn't match
        if lower.contains("grub2") {
            log_info("BOOTLOADER", "Detected GRUB 2 from EFI entries");
            return Some("GRUB 2".to_string());
        } else if lower.contains("grub") {
            log_info("BOOTLOADER", "Detected GRUB from EFI entries");
            return Some("GRUB".to_string());
        } else if lower.contains("systemd") || lower.contains("gummiboot") {
            log_info("BOOTLOADER", "Detected systemd-boot from EFI entries");
            return Some("systemd-boot".to_string());
        } else if lower.contains("refind") {
            log_info("BOOTLOADER", "Detected rEFInd from EFI entries");
            return Some("rEFInd".to_string());
        } else if lower.contains("limine") {
            log_info("BOOTLOADER", "Detected Limine from EFI entries");
            return Some("Limine".to_string());
        } else if lower.contains("clover") {
            log_info("BOOTLOADER", "Detected Clover from EFI entries");
            return Some("Clover".to_string());
        } else if lower.contains("opencore") {
            log_info("BOOTLOADER", "Detected OpenCore from EFI entries");
            return Some("OpenCore".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 2: Check bootctl for systemd-boot (before file checks)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking bootctl status for systemd-boot");
    if let Some(output) = run_cmd("bootctl", &["status"]) {
        let lower = output.to_lowercase();
        if lower.contains("systemd-boot") {
            // Try to extract version
            for line in output.lines() {
                if line.to_lowercase().contains("systemd-boot") && line.contains("(") {
                    log_info("BOOTLOADER", &format!("Detected systemd-boot via bootctl: {}", line.trim()));
                    return Some("systemd-boot".to_string());
                }
            }
            log_info("BOOTLOADER", "Detected systemd-boot via bootctl");
            return Some("systemd-boot".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 3: Check for systemd-boot (gummiboot successor)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for systemd-boot configuration files");
    let systemd_paths = [
        "/boot/efi/loader/loader.conf",
        "/boot/loader/loader.conf",
        "/efi/loader/loader.conf",
        "/boot/efi/loader/entries",
        "/boot/loader/entries",
        "/efi/loader/entries",
        "/boot/efi/EFI/systemd/systemd-bootx64.efi",
        "/boot/efi/EFI/BOOT/BOOTX64.EFI",  // Check if it's systemd-boot
    ];
    
    for path in &systemd_paths {
        if Path::new(path).exists() {
            // For BOOTX64.EFI, verify it's systemd-boot
            if path.contains("BOOTX64.EFI") {
                if let Ok(content) = fs::read(path) {
                    let content_str = String::from_utf8_lossy(&content[..content.len().min(8192)]);
                    if content_str.contains("systemd-boot") || content_str.contains("gummiboot") {
                        log_info("BOOTLOADER", "Detected systemd-boot via BOOTX64.EFI signature");
                        return Some("systemd-boot".to_string());
                    }
                }
            } else {
                log_info("BOOTLOADER", &format!("Detected systemd-boot via {}", path));
                return Some("systemd-boot".to_string());
            }
        }
    }
    
    // ============================================================================
    // METHOD 4: Check for GRUB (most common bootloader)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for GRUB configuration files");
    
    // Determine GRUB version through multiple methods
    let mut grub_version = String::new();
    
    // Method 4a: Check GRUB binary version
    if let Some(version_output) = run_cmd("grub-install", &["--version"])
        .or_else(|| run_cmd("grub2-install", &["--version"]))
        .or_else(|| run_cmd("grub-mkconfig", &["--version"])) {
        
        log_debug("BOOTLOADER", &format!("GRUB version check: {}", version_output.lines().next().unwrap_or("")));
        
        if version_output.contains("GRUB 2") || version_output.contains("(GRUB) 2") {
            grub_version = "GRUB 2".to_string();
        } else if version_output.contains("GRUB") {
            grub_version = "GRUB".to_string();
        }
    }
    
    // Method 4b: Check config file for version
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
        "/boot/efi/EFI/opensuse/grub.cfg",
        "/boot/efi/EFI/centos/grub.cfg",
        "/boot/efi/EFI/rhel/grub.cfg",
        "/boot/efi/EFI/gentoo/grub.cfg",
        "/boot/efi/EFI/manjaro/grub.cfg",
        "/boot/efi/EFI/endeavouros/grub.cfg",
        "/boot/efi/EFI/pop/grub.cfg",
        "/boot/efi/EFI/garuda/grub.cfg",
        "/boot/efi/EFI/zorin/grub.cfg",
        "/boot/efi/EFI/mint/grub.cfg",
        "/boot/efi/EFI/elementary/grub.cfg",
        "/boot/efi/EFI/kali/grub.cfg",
        "/boot/efi/EFI/parrot/grub.cfg",
        "/boot/efi/EFI/solus/grub.cfg",
        "/boot/efi/EFI/void/grub.cfg",
        "/boot/efi/EFI/alpine/grub.cfg",
        "/boot/efi/EFI/nixos/grub.cfg",
        "/boot/efi/EFI/slackware/grub.cfg",
        // Legacy BIOS locations
        "/boot/grub/menu.lst",
        "/boot/grub2/menu.lst",
        "/boot/grub/grub.conf",
    ];
    
    for path in &grub_paths {
        if Path::new(path).exists() {
            // Try to determine version from config file if not already known
            if grub_version.is_empty() {
                if path.contains("grub2") {
                    grub_version = "GRUB 2".to_string();
                } else if let Ok(content) = fs::read_to_string(path) {
                    // Read first few lines to determine version
                    let preview = content.lines().take(20).collect::<Vec<_>>().join("\n");
                    if preview.contains("GRUB2") || preview.contains("grub2") || preview.contains("set root") {
                        grub_version = "GRUB 2".to_string();
                    } else if preview.contains("GRUB") {
                        grub_version = "GRUB".to_string();
                    }
                }
            }
            
            // If still unknown, default to GRUB 2 (most common nowadays)
            if grub_version.is_empty() {
                grub_version = "GRUB 2".to_string();
            }
            
            log_info("BOOTLOADER", &format!("Detected {} via {}", grub_version, path));
            return Some(grub_version);
        }
    }
    
    // Method 4c: Check for GRUB in EFI directory (if config files not found)
    let efi_grub_paths = [
        "/boot/efi/EFI/grub/grubx64.efi",
        "/boot/efi/EFI/GRUB/grubx64.efi",
    ];
    
    for path in &efi_grub_paths {
        if Path::new(path).exists() {
            log_info("BOOTLOADER", &format!("Detected GRUB 2 via EFI binary: {}", path));
            return Some("GRUB 2".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 5: Check for rEFInd
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for rEFInd configuration files");
    let refind_paths = [
        "/boot/efi/EFI/refind/refind.conf",
        "/efi/EFI/refind/refind.conf",
        "/boot/efi/EFI/BOOT/refind.conf",
        "/boot/refind/refind.conf",
        "/boot/efi/refind/refind.conf",
        "/boot/efi/EFI/refind/refind_x64.efi",
    ];
    
    for path in &refind_paths {
        if Path::new(path).exists() {
            // Try to get version if it's the config file
            if path.ends_with("refind.conf") {
                if let Ok(content) = fs::read_to_string(path) {
                    for line in content.lines().take(30) {
                        if line.contains("rEFInd") || line.contains("refind") {
                            log_debug("BOOTLOADER", &format!("rEFInd config header: {}", line.trim()));
                            break;
                        }
                    }
                }
            }
            log_info("BOOTLOADER", &format!("Detected rEFInd via {}", path));
            return Some("rEFInd".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 6: Check for Limine
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Limine configuration files");
    let limine_paths = [
        "/boot/limine.cfg",
        "/boot/efi/limine.cfg",
        "/efi/limine.cfg",
        "/boot/limine/limine.cfg",
        "/boot/efi/EFI/limine/limine.cfg",
        "/boot/efi/EFI/BOOT/limine.cfg",
        "/boot/efi/EFI/BOOT/BOOTX64.EFI",
        "/boot/limine.sys",
    ];
    
    for path in &limine_paths {
        if Path::new(path).exists() {
            // For BOOTX64.EFI, verify it's actually Limine
            if path.contains("BOOTX64.EFI") {
                if let Ok(content) = fs::read(path) {
                    let content_str = String::from_utf8_lossy(&content[..content.len().min(8192)]);
                    if content_str.contains("Limine") || content_str.contains("limine") {
                        log_info("BOOTLOADER", "Detected Limine via BOOTX64.EFI signature");
                        return Some("Limine".to_string());
                    }
                }
            } else {
                log_info("BOOTLOADER", &format!("Detected Limine via {}", path));
                return Some("Limine".to_string());
            }
        }
    }
    
    // ============================================================================
    // METHOD 7: Check for Clover
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Clover configuration files");
    let clover_paths = [
        "/boot/efi/EFI/CLOVER/config.plist",
        "/efi/EFI/CLOVER/config.plist",
        "/boot/efi/EFI/CLOVER/CLOVERX64.efi",
    ];
    
    for path in &clover_paths {
        if Path::new(path).exists() {
            if path.contains("config.plist") {
                log_info("BOOTLOADER", &format!("Detected Clover via {}", path));
                return Some("Clover".to_string());
            } else if path.contains("CLOVERX64.efi") {
                log_info("BOOTLOADER", "Detected Clover via EFI binary");
                return Some("Clover".to_string());
            }
        }
    }
    
    // ============================================================================
    // METHOD 8: Check for OpenCore
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for OpenCore configuration files");
    let opencore_paths = [
        "/boot/efi/EFI/OC/config.plist",
        "/efi/EFI/OC/config.plist",
        "/boot/efi/EFI/OC/OpenCore.efi",
    ];
    
    for path in &opencore_paths {
        if Path::new(path).exists() {
            log_info("BOOTLOADER", &format!("Detected OpenCore via {}", path));
            return Some("OpenCore".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 9: Check for LILO (Legacy)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for LILO configuration");
    if Path::new("/etc/lilo.conf").exists() {
        log_info("BOOTLOADER", "Detected LILO via /etc/lilo.conf");
        return Some("LILO".to_string());
    }
    
    // ============================================================================
    // METHOD 10: Check for Syslinux/ISOLINUX/EXTLINUX/PXELINUX
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Syslinux variants");
    let syslinux_paths = [
        "/boot/syslinux/syslinux.cfg",
        "/boot/extlinux/extlinux.conf",
        "/boot/isolinux/isolinux.cfg",
        "/extlinux.conf",
        "/syslinux.cfg",
        "/boot/syslinux.cfg",
        "/boot/pxelinux.cfg/default",
    ];
    
    for path in &syslinux_paths {
        if Path::new(path).exists() {
            let name = if path.contains("extlinux") {
                "EXTLINUX"
            } else if path.contains("isolinux") {
                "ISOLINUX"
            } else if path.contains("pxelinux") {
                "PXELINUX"
            } else {
                "Syslinux"
            };
            log_info("BOOTLOADER", &format!("Detected {} via {}", name, path));
            return Some(name.to_string());
        }
    }
    
    // ============================================================================
    // METHOD 11: Check for U-Boot (ARM devices, embedded systems)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for U-Boot");
    let uboot_paths = [
        "/boot/u-boot.bin",
        "/boot/boot.scr",
        "/boot/uEnv.txt",
        "/boot/uboot.env",
        "/boot/extlinux/extlinux.conf",  // U-Boot can use extlinux
    ];
    
    for path in &uboot_paths {
        if Path::new(path).exists() {
            log_info("BOOTLOADER", &format!("Detected U-Boot via {}", path));
            return Some("U-Boot".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 12: Check for BURG (GRUB fork with themes)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for BURG");
    if Path::new("/boot/burg/burg.cfg").exists() {
        log_info("BOOTLOADER", "Detected BURG via /boot/burg/burg.cfg");
        return Some("BURG".to_string());
    }
    
    // ============================================================================
    // METHOD 13: Check for ELILO (EFI LILO)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for ELILO");
    if Path::new("/boot/efi/EFI/elilo/elilo.conf").exists() || 
       Path::new("/etc/elilo.conf").exists() {
        log_info("BOOTLOADER", "Detected ELILO");
        return Some("ELILO".to_string());
    }
    
    // ============================================================================
    // METHOD 14: Check for GRUB4DOS (DOS/Windows GRUB)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for GRUB4DOS");
    if Path::new("/boot/grub4dos/menu.lst").exists() {
        log_info("BOOTLOADER", "Detected GRUB4DOS");
        return Some("GRUB4DOS".to_string());
    }
    
    // ============================================================================
    // METHOD 15: Check for Petitboot (PlayStation, PowerPC)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Petitboot");
    if Path::new("/etc/petitboot").exists() {
        log_info("BOOTLOADER", "Detected Petitboot");
        return Some("Petitboot".to_string());
    }
    
    // ============================================================================
    // METHOD 16: Check for Raspberry Pi bootloader
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Raspberry Pi bootloader");
    if (Path::new("/boot/config.txt").exists() || Path::new("/boot/firmware/config.txt").exists()) && 
       (Path::new("/boot/start.elf").exists() || Path::new("/boot/firmware/start.elf").exists()) {
        log_info("BOOTLOADER", "Detected Raspberry Pi bootloader");
        return Some("Raspberry Pi Bootloader".to_string());
    }
    
    // ============================================================================
    // METHOD 17: Check MBR/Boot Sector for Legacy BIOS systems
    // ============================================================================
    log_debug("BOOTLOADER", "Checking boot device MBR signature");
    
    // Try to find the boot device
    if let Ok(mounts) = fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && (parts[1] == "/" || parts[1] == "/boot") {
                let boot_device = parts[0];
                // Extract base device (e.g., /dev/sda from /dev/sda1)
                let base_device = boot_device
                    .trim_end_matches(|c: char| c.is_ascii_digit())
                    .trim_end_matches(|c: char| c == 'p');  // Handle /dev/nvme0n1p1
                
                log_debug("BOOTLOADER", &format!("Checking boot device: {}", base_device));
                
                // Read first 512 bytes of boot device (requires root, may fail)
                if let Ok(mbr) = fs::read(base_device) {
                    if mbr.len() >= 512 {
                        let mbr_str = String::from_utf8_lossy(&mbr[0..512]);
                        
                        if mbr_str.contains("GRUB") {
                            log_info("BOOTLOADER", "Detected GRUB from MBR signature");
                            return Some("GRUB".to_string());
                        } else if mbr_str.contains("LILO") {
                            log_info("BOOTLOADER", "Detected LILO from MBR signature");
                            return Some("LILO".to_string());
                        } else if mbr_str.contains("SYSLINUX") {
                            log_info("BOOTLOADER", "Detected Syslinux from MBR signature");
                            return Some("Syslinux".to_string());
                        } else if mbr_str.contains("ISOLINUX") {
                            log_info("BOOTLOADER", "Detected ISOLINUX from MBR signature");
                            return Some("ISOLINUX".to_string());
                        }
                    }
                }
                
                if parts[1] == "/" {
                    break;  // Found root, no need to continue
                }
            }
        }
    }
    
    // ============================================================================
    // METHOD 18: Check kernel command line for bootloader hints
    // ============================================================================
    log_debug("BOOTLOADER", "Checking kernel command line for hints");
    if let Ok(cmdline) = fs::read_to_string("/proc/cmdline") {
        let lower = cmdline.to_lowercase();
        
        log_debug("BOOTLOADER", &format!("Kernel cmdline: {}", cmdline.chars().take(200).collect::<String>()));
        
        if lower.contains("grub") {
            log_info("BOOTLOADER", "Detected GRUB from kernel command line");
            return Some("GRUB".to_string());
        } else if lower.contains("systemd-boot") || lower.contains("gummiboot") {
            log_info("BOOTLOADER", "Detected systemd-boot from kernel command line");
            return Some("systemd-boot".to_string());
        } else if lower.contains("refind") {
            log_info("BOOTLOADER", "Detected rEFInd from kernel command line");
            return Some("rEFInd".to_string());
        } else if lower.contains("bootloader=") {
            // Some systems specify bootloader explicitly
            for param in cmdline.split_whitespace() {
                if param.starts_with("bootloader=") {
                    let bl = param.split('=').nth(1).unwrap_or("").to_string();
                    if !bl.is_empty() {
                        log_info("BOOTLOADER", &format!("Detected {} from kernel parameter", bl));
                        return Some(bl);
                    }
                }
            }
        }
    }
    
    // ============================================================================
    // METHOD 19: Check dmesg for bootloader messages
    // ============================================================================
    log_debug("BOOTLOADER", "Checking dmesg for bootloader hints");
    if let Some(dmesg) = run_cmd("dmesg", &[]) {
        let lower = dmesg.to_lowercase();
        
        if lower.contains("grub") && lower.contains("loading") {
            log_info("BOOTLOADER", "Detected GRUB from dmesg");
            return Some("GRUB".to_string());
        } else if lower.contains("systemd-boot") {
            log_info("BOOTLOADER", "Detected systemd-boot from dmesg");
            return Some("systemd-boot".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 20: Check for UEFI firmware capsule updates (indicates UEFI boot)
    // ============================================================================
    log_debug("BOOTLOADER", "Checking UEFI firmware interface");
    if Path::new("/sys/firmware/efi/efivars").exists() {
        // System is UEFI but bootloader unknown
        log_debug("BOOTLOADER", "UEFI system detected, checking EFI variables");
        
        // Try to read EFI variables for more info
        if let Ok(entries) = fs::read_dir("/sys/firmware/efi/efivars") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("bootloader") || name.contains("loader") {
                    log_debug("BOOTLOADER", &format!("Found EFI variable: {}", name));
                }
            }
        }
    }
    
    // ============================================================================
    // METHOD 21: Check for Coreboot/Libreboot
    // ============================================================================
    log_debug("BOOTLOADER", "Checking for Coreboot/Libreboot");
    if let Ok(dmi_version) = fs::read_to_string("/sys/class/dmi/id/bios_version") {
        let lower = dmi_version.to_lowercase();
        if lower.contains("coreboot") {
            log_info("BOOTLOADER", "Detected Coreboot firmware");
            return Some("Coreboot".to_string());
        } else if lower.contains("libreboot") {
            log_info("BOOTLOADER", "Detected Libreboot firmware");
            return Some("Libreboot".to_string());
        }
    }
    
    // ============================================================================
    // METHOD 22: Final fallback - check if system is UEFI or BIOS
    // ============================================================================
    log_debug("BOOTLOADER", "Performing final UEFI/BIOS check");
    if Path::new("/sys/firmware/efi").exists() {
        log_warn("BOOTLOADER", "UEFI system detected but bootloader could not be identified");
        
        // Last attempt: check if there's ANY EFI file in the ESP
        let efi_check_paths = [
            "/boot/efi/EFI",
            "/boot/EFI",
            "/efi/EFI",
        ];
        
        for esp_path in &efi_check_paths {
            if let Ok(entries) = fs::read_dir(esp_path) {
                let dirs: Vec<_> = entries.flatten().collect();
                if !dirs.is_empty() {
                    log_debug("BOOTLOADER", &format!("Found {} EFI directories in {}", dirs.len(), esp_path));
                }
            }
        }
        
        return Some("Unknown (UEFI)".to_string());
    } else {
        log_warn("BOOTLOADER", "BIOS system detected but bootloader could not be identified");
        return Some("Unknown (BIOS)".to_string());
    }
}

fn get_packages() -> Option<String> {
    let mut counts = Vec::with_capacity(5);
    
    if let Ok(entries) = fs::read_dir("/var/lib/pacman/local") {
        let count = entries.filter_map(Result::ok)
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .count();
        if count > 0 {
            counts.push(format!("{} (pacman)", count));
        }
    }
    
    if Path::new("/var/lib/dpkg/status").exists() {
        if let Some(count) = run_cmd("dpkg", &["-l"]).map(|s| s.lines().filter(|l| l.starts_with("ii")).count()) {
            counts.push(format!("{} (dpkg)", count));
        }
    }
    
    if Path::new("/var/lib/rpm").exists() {
        if let Some(count) = run_cmd("rpm", &["-qa"]).map(|s| s.lines().count()) {
            counts.push(format!("{} (rpm)", count));
        }
    }

    if let Ok(entries) = fs::read_dir("/var/lib/flatpak/app") {
        let count = entries.filter_map(Result::ok).count();
        if count > 0 { counts.push(format!("{} (flatpak)", count)); }
    }
    
    if let Ok(entries) = fs::read_dir("/var/lib/snapd/snaps") {
        let count = entries.filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".snap"))
            .count();
        if count > 0 { counts.push(format!("{} (snap)", count)); }
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

fn get_cpu_info_combined() -> CpuInfo {
    let mut info = CpuInfo {
        name: None,
        threads: 0,
        cores: None,
        cache: None,
        freq: None,
    };
    
    if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
        let mut physical_cores = HashMap::new();
        let mut current_physical_id = 0;
        
        for line in cpuinfo.lines() {
            if line.starts_with("processor") {
                info.threads += 1;
            } else if line.starts_with("model name") && info.name.is_none() {
                if let Some(name) = line.split(':').nth(1) {
                    let name = name.trim();
                    info.name = Some(name.replace("(R)", "")
                                   .replace("(TM)", "")
                                   .replace("Intel Core", "Intel")
                                   .split_whitespace()
                                   .filter(|s| !s.is_empty())
                                   .collect::<Vec<_>>()
                                   .join(" "));
                }
            } else if line.starts_with("physical id") {
                if let Some(id_str) = line.split(':').nth(1) {
                    current_physical_id = id_str.trim().parse::<usize>().unwrap_or(0);
                }
            } else if line.starts_with("cpu cores") {
                if let Some(cores_str) = line.split(':').nth(1) {
                    if let Ok(cores) = cores_str.trim().parse::<usize>() {
                        physical_cores.insert(current_physical_id, cores);
                    }
                }
            } else if line.starts_with("cache size") && info.cache.is_none() {
                if let Some(cache_str) = line.split(':').nth(1) {
                    info.cache = Some(cache_str.trim().to_string());
                }
            }
        }
        
        let total_cores: usize = physical_cores.values().sum();
        info.cores = if total_cores > 0 { Some(total_cores) } else { None };
    }
    
    info.freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .map(|mhz| format!("{:.2} GHz", mhz / 1000000.0));
    
    info
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

/// Single `lspci -v` call. Parses GPU names AND per-GPU VRAM in one pass.
fn get_gpu_combined() -> (Option<Vec<String>>, Option<Vec<String>>) {
    let output = match run_cmd("lspci", &["-v"]) {
        Some(o) => o,
        None    => return (None, None),
    };

    let mut gpus:  Vec<String> = Vec::with_capacity(2);
    let mut vrams: Vec<String> = Vec::with_capacity(2);
    let mut cur_vram: Option<String> = None;
    let mut in_gpu = false;

    for line in output.lines() {
        let lower = line.to_lowercase();

        // Top-level device line (no leading whitespace)
        if !line.starts_with('\t') && !line.starts_with(' ') {
            // flush previous GPU's vram
            if in_gpu { vrams.push(cur_vram.take().unwrap_or_default()); }
            in_gpu = false;

            if lower.contains("bridge") || lower.contains("audio") || lower.contains("usb") { continue; }
            if !((lower.contains("vga") || lower.contains("3d") ||
                  (lower.contains("display") && !lower.contains("audio"))) &&
                 lower.contains("controller")) { continue; }

            if let Some(pos) = line.find("controller:") {
                let mut desc = line[pos + 11..].trim().to_string();
                if let Some(rp) = desc.find(" (rev ") { desc.truncate(rp); }
                desc = desc.replace("Intel Corporation", "Intel")
                           .replace("Advanced Micro Devices, Inc.", "AMD")
                           .replace("[AMD/ATI]", "AMD")
                           .replace("NVIDIA Corporation", "NVIDIA")
                           .replace("Corporation", "");
                let desc = desc.trim().to_string();
                if desc.len() > 10 && !desc.to_lowercase().contains("bridge") && !desc.starts_with("Device ") {
                    gpus.push(desc);
                    in_gpu = true;
                    cur_vram = None;
                }
            }
            continue;
        }

        // Detail line inside a GPU block — look for Memory size=
        if in_gpu && line.contains("Memory at") && line.contains("size=") {
            if let Some(p) = line.find("size=") {
                let rest = &line[p + 5..];
                if let Some(end) = rest.find(']') {
                    let s = &rest[..end];
                    let val = parse_human_size(s).unwrap_or(0.0);
                    let cur = cur_vram.as_ref().and_then(|v| parse_human_size(v)).unwrap_or(0.0);
                    if val > cur { cur_vram = Some(s.to_string()); }
                }
            }
        }
    }
    if in_gpu { vrams.push(cur_vram.unwrap_or_default()); }

    let vrams: Vec<String> = vrams.into_iter().filter(|s| !s.is_empty()).collect();
    (
        if gpus.is_empty()  { None } else { Some(gpus) },
        if vrams.is_empty() { None } else { Some(vrams) },
    )
}

fn get_gpu_temp_with_gpus(gpus: Option<&Vec<String>>) -> Option<Vec<Option<String>>> {
    let gpus = gpus?;
    if gpus.is_empty() {
        return None;
    }
    
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

/// Single read of /proc/meminfo. Returns (memory, swap).
fn get_memory_and_swap() -> (Option<(f64, f64)>, Option<(f64, f64)>) {
    let meminfo = match fs::read_to_string("/proc/meminfo") {
        Ok(s) => s,
        Err(_) => return (None, None),
    };
    let (mut mt, mut ma, mut st, mut sf) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let (mut a, mut b, mut c, mut d) = (false, false, false, false);
    for line in meminfo.lines() {
        if a && b && c && d { break; } // all four found, stop scanning
        if !a && line.starts_with("MemTotal:") {
            if let Some(v) = line.split_whitespace().nth(1).and_then(|s| s.parse::<f64>().ok()) { mt = v / KB_TO_GIB; a = true; }
        } else if !b && line.starts_with("MemAvailable:") {
            if let Some(v) = line.split_whitespace().nth(1).and_then(|s| s.parse::<f64>().ok()) { ma = v / KB_TO_GIB; b = true; }
        } else if !c && line.starts_with("SwapTotal:") {
            if let Some(v) = line.split_whitespace().nth(1).and_then(|s| s.parse::<f64>().ok()) { st = v / KB_TO_GIB; c = true; }
        } else if !d && line.starts_with("SwapFree:") {
            if let Some(v) = line.split_whitespace().nth(1).and_then(|s| s.parse::<f64>().ok()) { sf = v / KB_TO_GIB; d = true; }
        }
    }
    let mem  = if mt  > 0.0 { Some((mt  - ma, mt))  } else { None };
    let swap = if st > 0.0 { Some((st - sf, st)) } else { None };
    (mem, swap)
}

/// Returns (display, resolution). At most one subprocess on x11 (xrandr) or wayland (wlr-randr).
fn get_display_and_resolution() -> (Option<String>, Option<String>) {
    if let Ok(stype) = std::env::var("XDG_SESSION_TYPE") {
        if stype == "wayland" {
            let disp = match std::env::var("WAYLAND_DISPLAY") {
                Ok(wd) => format!("Wayland ({})", wd),
                Err(_) => "Wayland".to_string(),
            };
            let res = run_cmd("wlr-randr", &[]).and_then(|out|
                out.lines().find(|l| l.contains(" px, ") && l.contains(" Hz")).map(|l| l.trim().to_string())
            );
            return (Some(disp), res);
        }
        if stype == "x11" {
            // Single xrandr call serves both display and resolution
            if let Some(out) = run_cmd("xrandr", &["--current"]) {
                let mut res: Option<String> = None;
                for line in out.lines() {
                    if line.contains(" connected") && (line.contains(" primary") || !line.contains(" disconnected")) {
                        for (i, p) in line.split_whitespace().enumerate() {
                            if i > 0 && p.contains('x') && p.as_bytes().first().map_or(false, |b| b.is_ascii_digit()) {
                                res = Some(p.to_string());
                                break;
                            }
                        }
                        break;
                    }
                }
                let disp = match &res {
                    Some(r) => format!("{} (X11)", r),
                    None    => "X11".to_string(),
                };
                return (Some(disp), res);
            }
            return (Some("X11".to_string()), None);
        }
    }
    // Fallback: env vars only, no resolution available
    if std::env::var("DISPLAY").is_ok()          { return (Some("X11".to_string()),      None); }
    if std::env::var("WAYLAND_DISPLAY").is_ok() { return (Some("Wayland".to_string()), None); }
    (None, None)
}

fn get_entropy() -> Option<String> {
    let avail = read_file_trim("/proc/sys/kernel/random/entropy_avail")?;
    let pool = read_file_trim("/proc/sys/kernel/random/poolsize").unwrap_or_else(|| "4096".to_string());
    Some(format!("{}/{}", avail, pool))
}

fn get_users_count() -> Option<usize> {
    log_debug("USERS", "Counting currently logged-in users");
    
    // Try to read /var/run/utmp to count logged in users
    // This is the most accurate method as it shows actual login sessions
    if let Some(output) = run_cmd("who", &[]) {
        let count = output.lines()
            .filter(|line| !line.trim().is_empty())
            .count();
        if count > 0 {
            log_debug("USERS", &format!("Found {} logged-in user(s) via 'who' command", count));
            return Some(count);
        }
    }
    
    // Fallback: try 'users' command which lists logged in users
    if let Some(output) = run_cmd("users", &[]) {
        let count = output.split_whitespace().count();
        if count > 0 {
            log_debug("USERS", &format!("Found {} logged-in user(s) via 'users' command", count));
            return Some(count);
        }
    }
    
    // Last fallback: at least count ourselves as 1 logged-in user
    log_debug("USERS", "Could not determine logged-in users, defaulting to 1 (current user)");
    Some(1)
}

fn get_failed_units() -> Option<usize> {
    run_cmd("systemctl", &["list-units", "--failed", "--no-legend", "--no-pager"])
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
}

fn get_partitions_impl() -> Option<Vec<(String, String, f64, f64)>> {
    // Find device + fstype for "/" from /proc/mounts (zero spawns)
    let mounts = fs::read_to_string("/proc/mounts").ok()?;
    let mut dev = "root";
    let mut fst = "unknown";
    for line in mounts.lines() {
        let mut it = line.splitn(4, ' ');
        let d = it.next().unwrap_or("");
        let mp = it.next().unwrap_or("");
        let f  = it.next().unwrap_or("");
        if mp == "/" { dev = d; fst = f; break; }
    }
    let dev_short = dev.rsplit('/').next().unwrap_or(dev);

    // statfs syscall — no external binary needed
    #[repr(C)]
    struct Statfs { f_type: i64, f_bsize: i64, f_blocks: u64, f_bfree: u64, f_bavail: u64,
                    f_files: u64, f_ffree: u64, f_fsid: [i64; 2], f_flag: i64, f_namelen: i64, _pad: [i64; 4] }
    extern "C" { fn statfs(path: *const u8, buf: *mut Statfs) -> i32; }
    let mut s = Statfs { f_type:0, f_bsize:0, f_blocks:0, f_bfree:0, f_bavail:0,
                         f_files:0, f_ffree:0, f_fsid:[0;2], f_flag:0, f_namelen:0, _pad:[0;4] };
    if unsafe { statfs(b"/\0".as_ptr(), &mut s) } != 0 { return None; }

    let bs    = s.f_bsize as f64;
    let total = s.f_blocks as f64 * bs / (1024.0 * 1024.0 * 1024.0);
    let avail = s.f_bavail as f64 * bs / (1024.0 * 1024.0 * 1024.0);
    if total <= 0.0 { return None; }
    Some(vec![(format!("{} - {}", dev_short, fst), "/".to_string(), total - avail, total)])
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    let args_str = args.join(" ");
    log_debug("COMMAND", &format!("Executing: {} {}", cmd, args_str));
    
    match Command::new(cmd).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(stdout) => {
                        let result = stdout.trim().to_string();
                        log_debug("COMMAND", &format!("Success: {} {} (output length: {} bytes)", 
                            cmd, args_str, result.len()));
                        Some(result)
                    }
                    Err(e) => {
                        log_error("COMMAND", &format!("Failed to parse UTF-8 output from {} {}: {}", 
                            cmd, args_str, e));
                        None
                    }
                }
            } else {
                let exit_code = output.status.code().unwrap_or(-1);
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_warn("COMMAND", &format!("Command failed: {} {} (exit code: {}, stderr: {})", 
                    cmd, args_str, exit_code, stderr.trim()));
                None
            }
        }
        Err(e) => {
            log_error("COMMAND", &format!("Failed to execute {} {}: {} (command may not be installed or available)", 
                cmd, args_str, e));
            None
        }
    }
}

fn read_file_trim(path: &str) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => {
            let trimmed = content.trim().to_string();
            log_debug("FILE", &format!("Successfully read {}: {} bytes", path, trimmed.len()));
            Some(trimmed)
        }
        Err(e) => {
            log_debug("FILE", &format!("Could not read {} (this is normal if file doesn't exist): {}", path, e));
            None
        }
    }
}

fn get_model() -> Option<String> {
    let vendor = read_file_trim("/sys/class/dmi/id/sys_vendor").unwrap_or_default();
    let product = read_file_trim("/sys/class/dmi/id/product_name").unwrap_or_default();
    if vendor.is_empty() && product.is_empty() { return None; }
    Some(format!("{} {}", vendor, product).trim().to_string())
}

fn get_motherboard() -> Option<String> {
    read_file_trim("/sys/class/dmi/id/board_name")
}

fn get_bios() -> Option<String> {
    read_file_trim("/sys/class/dmi/id/bios_version")
}

fn get_processes() -> Option<usize> {
    fs::read_dir("/proc").ok()?.filter_map(|e| e.ok()).filter(|e| {
        e.file_name().to_str().map(|s| s.chars().all(|c| c.is_ascii_digit())).unwrap_or(false)
    }).count().into()
}

fn get_locale() -> Option<String> {
    env::var("LANG").ok()
}

fn get_public_ip() -> Option<String> {
    run_cmd("curl", &["-s", "--connect-timeout", "1", "https://icanhazip.com"])
}

struct ThemeInfo {
    theme: Option<String>,
    icons: Option<String>,
    font: Option<String>,
}

fn get_theme_info() -> ThemeInfo {
    let mut info = ThemeInfo { theme: None, icons: None, font: None };

    // KDE path first — pure file reads, zero spawns.
    if let Ok(home) = env::var("HOME") {
        if let Ok(content) = fs::read_to_string(format!("{}/.config/kdeglobals", home)) {
            let mut in_icons = false;
            for line in content.lines() {
                if line == "[Icons]"  { in_icons = true;  continue; }
                if line.starts_with('[') { in_icons = false; }
                if in_icons && line.starts_with("theme=") && info.icons.is_none() {
                    info.icons = Some(line.split('=').nth(1).unwrap_or("").to_string());
                }
                if line.starts_with("widgetStyle=") && info.theme.is_none() {
                    info.theme = Some(line.split('=').nth(1).unwrap_or("").to_string());
                }
                if line.starts_with("font=") && info.font.is_none() {
                    info.font = Some(line.split('=').nth(1).unwrap_or("").split(',').next().unwrap_or("").to_string());
                }
            }
        }
    }

    // Only spawn gsettings for values still missing (GNOME fallback).
    if info.theme.is_none() {
        if let Some(v) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "gtk-theme"]) {
            let v = v.trim_matches('\''); if !v.is_empty() { info.theme = Some(v.to_string()); }
        }
    }
    if info.icons.is_none() {
        if let Some(v) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "icon-theme"]) {
            let v = v.trim_matches('\''); if !v.is_empty() { info.icons = Some(v.to_string()); }
        }
    }
    if info.font.is_none() {
        if let Some(v) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "font-name"]) {
            let v = v.trim_matches('\''); if !v.is_empty() { info.font = Some(v.to_string()); }
        }
    }
    info
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

fn get_network_final_with_ip(net_start: Option<String>, delta: f64, should_ping: bool, ip_out: Option<String>) -> Option<Vec<NetworkInfo>> {
    let dev1 = net_start?;
    let dev2 = fs::read_to_string("/proc/net/dev").ok()?;
    
    let mut stats1 = HashMap::new();
    for line in dev1.lines().skip(2) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() > 9 { stats1.insert(p[0].trim_end_matches(':').to_string(), (p[1].parse::<u64>().unwrap_or(0), p[9].parse::<u64>().unwrap_or(0))); }
    }

    let mut ip_map = HashMap::new();
    if let Some(output) = ip_out {
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 { continue; }
            let iface = parts[1].to_string();
            let family = parts[2];
            let addr = parts[3].split('/').next().unwrap_or(parts[3]).to_string();
            let entry = ip_map.entry(iface).or_insert((None, None));
            if family == "inet" { entry.0 = Some(addr); }
            else if family == "inet6" && !addr.starts_with("fe80") && addr != "::1" { entry.1 = Some(addr); }
        }
    }

    let mut networks = Vec::with_capacity(4);
    for line in dev2.lines().skip(2) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() < 10 { continue; }
        let interface = p[0].trim_end_matches(':').to_string();
        if interface == "lo" { continue; }
        let (ipv4, ipv6) = ip_map.remove(&interface).unwrap_or((None, None));
        let state = read_file_trim(&format!("/sys/class/net/{}/operstate", interface)).unwrap_or_else(|| "unknown".to_string()).to_uppercase();
        let rx2 = p[1].parse::<u64>().ok();
        let tx2 = p[9].parse::<u64>().ok();
        
        let mut rx_rate = None;
        let mut tx_rate = None;
        if let (Some(r2), Some(t2), Some(&(r1, t1))) = (rx2, tx2, stats1.get(&interface)) {
            rx_rate = Some((r2.saturating_sub(r1) as f64 / (1024.0 * 1024.0)) / delta);
            tx_rate = Some((t2.saturating_sub(t1) as f64 / (1024.0 * 1024.0)) / delta);
        }

        let mut p_stat = None;
        let mut j_stat = None;
        let mut l_stat = None;
        if should_ping && state == "UP" && ipv4.is_some() {
            if let Some(out) = run_cmd("ping", &["-c", "2", "-i", "0.2", "-W", "1", "1.1.1.1"]) {
                for l in out.lines() {
                    if l.contains("packet loss") {
                        if let Some(pos) = l.find('%') {
                            let start = l[..pos].rfind(' ').unwrap_or(0);
                            l_stat = l[start..pos].trim().parse::<f64>().ok();
                        }
                    } else if l.contains("rtt min/avg/max/mdev") {
                        let stats: Vec<&str> = l.split('=').nth(1).unwrap_or("").trim().split('/').collect();
                        if stats.len() >= 4 {
                            p_stat = stats[1].parse::<f64>().ok();
                            j_stat = stats[3].split(' ').next().and_then(|s| s.parse::<f64>().ok());
                        }
                    }
                }
            }
        }

        networks.push(NetworkInfo {
            interface, ipv4, ipv6, mac: None, state, rx_bytes: rx2, tx_bytes: tx2,
            rx_rate_mbs: rx_rate, tx_rate_mbs: tx_rate, ping: p_stat, jitter: j_stat, packet_loss: l_stat,
        });
    }

    networks.sort_by(|a, b| {
        let a_up = a.state == "UP";
        let b_up = b.state == "UP";
        if a_up != b_up { b_up.cmp(&a_up) } else { a.interface.cmp(&b.interface) }
    });

    if networks.is_empty() { None } else { Some(networks) }
}

// ============================================================================
// ASCII LOGOS
// ============================================================================

fn get_logo(os: &str) -> Vec<String> {
    let ol = os.to_lowercase();
    
    let lines: &[&str] = if ol.contains("cachy") {
        &[
            r#"           .-------------------------:"#,
            r#"          .+=========================."#,
            r#"         :++===++==================-       :++-"#,
            r#"        :*++====+++++=============-        .==:"#,
            r#"       -*+++=====+***++==========:"#,
            r#"      =*++++========------------:"#,
            r#"     =*+++++=====-                     ..."#,
            r#"   .+*+++++=-===:                    .=+++=:"#,
            r#"  :++++=====-==:                     -*****+"#,
            r#" :++========-=.                      .=+**+."#,
            r#".+==========-.                          ."#,
            r#" :+++++++====-                                .--==-."#,
            r#"  :++==========.                             :+++++++:"#,
            r#"   .-===========.                            =*****+*+"#,
            r#"    .-===========:                           .+*****+:"#,
            r#"      -=======++++:::::::::::::::::::::::::-:  .---:"#,
            r#"       :======++++====+++******************=."#,
            r#"        :=====+++==========++++++++++++++*-"#,
            r#"         .====++==============++++++++++*-"#,
            r#"          .===+==================+++++++:"#,
            r#"           .-=======================+++:"#,
            r#"             .........................."#,
        ]
    } else if ol.contains("bazzite") {
        &[
            r#"         ,....,          "#,
            r#"       ,::::::<          "#,
            r#"      ,::/^\/::.         "#,
            r#"     ,::/   \::.         "#,
            r#"    ,::/     \::.        "#,
            r#"   ,::/       \::.       "#,
            r#"   :::         :::       "#,
            r#"   `::.       .::'       "#,
            r#"     `::.   .::'         "#,
            r#"       `:::::'           "#,
            r#"         `'''            "#,
        ]
    } else if ol.contains("arch") || ol.contains("artix") || ol.contains("arco") {
        &[
            r#"                   -`                    "#,
            r#"                  .o+`                   "#,
            r#"                 `ooo/                   "#,
            r#"                `+oooo:                  "#,
            r#"               `+oooooo:                 "#,
            r#"               -+oooooo+:                "#,
            r#"             `/:-:++oooo+:               "#,
            r#"            `/++++/+++++++:              "#,
            r#"           `/++++++++++++++:             "#,
            r#"          `/+++ooooooooooooo/`           "#,
            r#"         ./ooosssso++osssssso+`          "#,
            r#"        .oossssso-````/ossssss+`         "#,
            r#"       -osssssso.      :ssssssso.        "#,
            r#"      :osssssss/        osssso+++.       "#,
            r#"     /ossssssss/        +ssssooo/-       "#,
            r#"   `/ossssso+/:-        -:/+osssso+-     "#,
            r#"  `+sso+:-`                 `.-/+oso:    "#,
            r#" `++:.                           `-/+/   "#,
            r#" .`                                 `/   "#,
        ]
    } else if ol.contains("ubuntu") || ol.contains("kubuntu") || ol.contains("xubuntu") || ol.contains("lubuntu") {
        &[
            r#"            .-/+oossssoo+/-.               "#,
            r#"        `:+ssssssssssssssssss+:`           "#,
            r#"      -+ssssssssssssssssssyyssss+-         "#,
            r#"    .ossssssssssssssssssdMMMNysssso.       "#,
            r#"   /ssssssssssshdmmNNmmyNMMMMhssssss/      "#,
            r#"  +ssssssssshmydMMMMMMMNddddyssssssss+     "#,
            r#" /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/    "#,
            r#" .ssssssssdMMMNhsssssssssshNMMMdssssssss.   "#,
            r#" +sssshhhyNMMNyssssssssssssyNMMMysssssss+   "#,
            r#" ossyNMMMNyMMhsssssssssssssshmmmhssssssso   "#,
            r#" ossyNMMMNyMMhsssssssssssssshmmmhssssssso   "#,
            r#" +sssshhhyNMMNyssssssssssssyNMMMysssssss+   "#,
            r#" .ssssssssdMMMNhsssssssssshNMMMdssssssss.   "#,
            r#"  /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/    "#,
            r#"   +ssssssssshmydMMMMMMMNddddyssssssss+     "#,
            r#"    /ssssssssssshdmmNNmmyNMMMMhssssss/      "#,
            r#"     .ossssssssssssssssssdMMMNysssso.       "#,
            r#"       -+ssssssssssssssssssyyssss+-         "#,
            r#"         `:+ssssssssssssssssss+:`           "#,
            r#"             .-/+oossssoo+/-.               "#,
        ]
    } else if ol.contains("debian") || ol.contains("raspberry") || ol.contains("raspbian") {
        &[
            r#"       _,met$$$$$gg.           "#,
            r#"    ,g$$$$$$$$$$$$$$$P.        "#,
            r#"  ,g$$P"     """Y$$. ".     "#,
            r#" ,$$P'              `$$$.      "#,
            r#"',$$P       ,ggs.     `$$b:    "#,
            r#"`d$$'     ,$P"'   .    $$$     "#,
            r#" $$P      d$'     ,    $$P     "#,
            r#" $$:      $$.   -    ,d$$'     "#,
            r#" $$;      Y$b._   _,d$P'       "#,
            r#" Y$$.    `.`"Y$$$$P"'          "#,
            r#" `$$b      "-.__               "#,
            r#"  `Y$$                         "#,
            r#"   `Y$$.                       "#,
            r#"     `$$b.                     "#,
            r#"       `Y$$b.                  "#,
            r#"          `"Y$b._              "#,
            r#"              `"""             "#,
        ]
    } else if ol.contains("fedora") {
        &[
            r#"          /:-------------:\          "#,
            r#"       :-------------------::        "#,
            r#"     :-----------/shhOHbmp---:\      "#,
            r#"   /-----------omMMMNNNMMD  ---:     "#,
            r#"  :-----------sMMMMNMNMP.    ---:    "#,
            r#" :-----------:MMMdP-------    ---\   "#,
            r#",------------:MMMd--------    ---:   "#,
            r#":------------:MMMd-------    .---:   "#,
            r#":----    oNMMMMMMMMMNho     .----:   "#,
            r#":--     .+shhhMMMmhhy++   .------/   "#,
            r#":-    -------:MMMd--------------:    "#,
            r#":-   --------/MMMd-------------;     "#,
            r#":-    ------/hMMMy------------:      "#,
            r#":-- :dMNdhhdNMMNo------------;       "#,
            r#":---:sdNMMMMNds:------------:        "#,
            r#":------:://:-------------::          "#,
            r#":---------------------://            "#,
        ]
    } else if ol.contains("manjaro") {
        &[
            r#"██████████████████  ████████   "#,
            r#"██████████████████  ████████   "#,
            r#"██████████████████  ████████   "#,
            r#"██████████████████  ████████   "#,
            r#"████████            ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
            r#"████████  ████████  ████████   "#,
        ]
    } else if ol.contains("mint") {
        &[
            r#" MMMMMMMMMMMMMMMMMMMMMMMMMmds+.        "#,
            r#" MMm----::-://////////////oymNMd+`     "#,
            r#" MMd      /++                -sNMd:    "#,
            r#" MMNso/`  dMM    `.::-. .-::.` .hMN:   "#,
            r#" ddddMMh  dMM   :hNMNMNhNMNMNh: `NMm   "#,
            r#"     NMm  dMM  .NMN/-+MMM+-/NMN` dMM   "#,
            r#"     NMm  dMM  -MMm  `MMM   dMM. dMM   "#,
            r#"     NMm  dMM  -MMm  `MMM   dMM. dMM   "#,
            r#"     NMm  dMM  .mmd  `mmm   yMM. dMM   "#,
            r#"     NMm  dMM`  ..`   ...   ydm. dMM   "#,
            r#"     hMM- +MMd/-------...-:sdds  dMM   "#,
            r#"     -NMm- :hNMNNNmdddddddddy/`  dMM   "#,
            r#"      -dMNs-``-::::-------.``    dMM   "#,
            r#"       `/dMNmy+/:.............:/yMMM   "#,
            r#"          ./ydNMMMMMMMMMMMMMMMMMMMMM   "#,
            r#"             \.MMMMMMMMMMMMMMMMMMM     "#,
        ]
    } else if ol.contains("pop") {
        &[
            r#"             /////////////                "#,
            r#"         /////////////////////            "#,
            r#"      ///////*767////////////////         "#,
            r#"    //////7676767676*//////////////       "#,
            r#"   /////76767//7676767//////////////      "#,
            r#"  /////767676///*76767///////////////     "#,
            r#" ///////767676///76767.///7676*///////    "#,
            r#"/////////767676//76767///767676////////   "#,
            r#"//////////76767676767////76767/////////   "#,
            r#"///////////76767676//////7676//////////   "#,
            r#"////////////,7676,///////767///////////   "#,
            r#"/////////////*7676///////76////////////   "#,
            r#"///////////////7676////////////////////   "#,
            r#" ///////////////7676///767////////////    "#,
            r#"  //////////////////////'////////////     "#,
            r#"   //////.7676767676767676767//////       "#,
            r#"    //////767676767676767676//////        "#,
            r#"      ///////////////////////////         "#,
            r#"         /////////////////////            "#,
            r#"             /////////////                "#,
        ]
    } else if ol.contains("gentoo") {
        &[
            r#"         -/oyddmdhs+:.                "#,
            r#"     -odNMMMMMMMMNNmhy+.              "#,
            r#"   -yNMMMMMMMMNmhhyhs+:`              "#,
            r#" -oNMMMMMMMMMNne`                     "#,
            r#" `oNMMMMMMMMN- `                      "#,
            r#"   `+yMMMMMMMm-                       "#,
            r#"     `+hMMMMMMMc                      "#,
            r#"       `oNMMMMMMd-                    "#,
            r#"         `sNMMMMMMm+`                 "#,
            r#"           `+dMMMMMMNho:              "#,
            r#"             `+hMMMMMMMMNds+.         "#,
            r#"               `+hNMMMMMMMMMMmy-      "#,
            r#"                 `/dNMMMMMMMMMMMy`    "#,
            r#"                   `:yNMMMMMMMMMMMs   "#,
            r#"                     `:hNMMMMMMMMMM+  "#,
        ]
    } else if ol.contains("nixos") || ol.contains("nix") {
        &[
            r#"          \\  \\ //          "#,
            r#"         ==\\__\\/ //        "#,
            r#"           //   \\//         "#,
            r#"        ==//     //==        "#,
            r#"         //\\___//           "#,
            r#"        // /\\  \\==         "#,
            r#"          // \\              "#,
        ]
    } else if ol.contains("void") {
        &[
            r#"                __.,,------.._     "#,
            r#"             ,'"   _      _   "`.  "#,
            r#"            /.__, ._  -=- _"`    Y "#,
            r#"           (.____.-.`      ""`   j "#,
            r#"           VvvvvvV`.Y,.    _.,-'`  "#,
            r#"              Y    ||,   '"\       "#,
            r#"              |    ,'  ,     `-..  "#,
            r#"              |  ,    o  ,  ,.'    "#,
            r#"              | ;       /   ;      "#,
            r#"              |  _  ,  /   ,       "#,
            r#"              |,' .   :  ,         "#,
            r#"              `--..__  `._`.._     "#,
            r#"                     `--..____,    "#,
        ]
    } else if ol.contains("alpine") {
        &[
            r#"       .hddddddddddddddddddddddh.          "#,
            r#"      :dddddddddddddddddddddddddd:         "#,
            r#"     /dddddddddddddddddddddddddddd/        "#,
            r#"    +dddddddddddddddddddddddddddddd+       "#,
            r#"  `sdddddddddddddddddddddddddddddddds`     "#,
            r#" `ydddddddddddd++hdddddddddddddddddddy`    "#,
            r#" .hddddddddddd+`  `+ddddh:-sdddddddddddh.   "#,
            r#" hdddddddddd+`      `+y:    .sddddddddddh   "#,
            r#" ddddddddh+`   `//`   `.`     -sddddddddd   "#,
            r#" ddddddh+`   `/hddh/`   `:s-    -sddddddd   "#,
            r#" ddddh+`   `/+/dddddh/`   `+s-    -sddddd   "#,
            r#" ddd+`   `/o` :dddddddh/`   `oy-    .yddd   "#,
            r#" hdddyo+ohddyosdddddddddho+oydddy++ohdddh   "#,
            r#" .hddddddddddddddddddddddddddddddddddddh.   "#,
            r#"  `yddddddddddddddddddddddddddddddddddy`    "#,
            r#"   `sdddddddddddddddddddddddddddddddds`     "#,
            r#"     +dddddddddddddddddddddddddddddd+       "#,
            r#"      /dddddddddddddddddddddddddddd/        "#,
            r#"       :dddddddddddddddddddddddddd:         "#,
            r#"        .hddddddddddddddddddddddh.          "#,
        ]
    } else if ol.contains("endeavour") || ol.contains("eos") {
        &[
            r#"                     ./o.                  "#,
            r#"                   ./sssso-                "#,
            r#"                 `:osssssss+-              "#,
            r#"               `:+sssssssssso/.            "#,
            r#"             `-/ossssssssssssso/.          "#,
            r#"           `-/+sssssssssssssssso+:`        "#,
            r#"         `-:/+sssssssssssssssssso+/.       "#,
            r#"       `.://osssssssssssssssssssswo++-     "#,
            r#"      .://+ssssssssssssssssssssssso++:     "#,
            r#"    .:///ossssssssssssssssssssssssso++:    "#,
            r#"  `:////ssssssssssssssssssssssssssso+++.   "#,
            r#" `-////+ssssssssssssssssssssssssssso++++-   "#,
            r#"  `..-+oosssssssssssssssssssssssso+++++/`   "#,
            r#"    ./++++++++++++++++++++++++++++++/:.     "#,
            r#"   `:::::::::::::::::::::::::------``       "#,
        ]
    } else if ol.contains("zorin") {
        &[
            r#"        `.:/++++++/-.`             "#,
            r#"      .:/++++++++++++/:-           "#,
            r#"    `:/++++++++++++++/++/.         "#,
            r#"   `:/++++++++++++++//++/+`        "#,
            r#"  .://++++++++++++++//++ /+        "#,
            r#"  :://++++++++++++++/++  :+        "#,
            r#"  /://+++++++++++++/++   :+        "#,
            r#"  /://++++++++++++/++    :+        "#,
            r#"  /://+++++++++++/++     :+        "#,
            r#"  /://++++++++++/++      :+        "#,
            r#"  /://+++++++++/++       :+        "#,
            r#"  /://++++++++/++        :+        "#,
            r#"  /://+++++++/++         :+        "#,
            r#"  /://++++++/++          :+        "#,
            r#"  /://+++++/++           :+        "#,
            r#"  /://++++/++            :+        "#,
            r#"  /://+++/++             :+        "#,
            r#"  /://++/++              :+        "#,
            r#"  /://+/++               :+        "#,
            r#"  /://++`                .+        "#,
            r#"   ++`                    `        "#,
        ]
    } else if ol.contains("kali") {
        &[
            r#"      ..............           "#,
            r#"    ..`  `......`  `..         "#,
            r#"  ..`  `.`......`.`  `..       "#,
            r#" ..  `.`  `....`  `.`  ..      "#,
            r#"..  `.` .` ... `. `.`  ..      "#,
            r#"..  `.` `.`...`.` `.`  ..      "#,
            r#" ..  `.`  `...`  `.`  ..       "#,
            r#"  ..`  `.` `.` `. `  ..        "#,
            r#"    ..`  `.` `.` ` ..          "#,
            r#"      ..`  `.` `. `            "#,
            r#"        ..`  ` .               "#,
            r#"          ..`                  "#,
            r#"            .                  "#,
        ]
    } else if ol.contains("garuda") {
        &[
            r#"             .           "#,
            r#"           .d8l          "#,
            r#"         .d8888l         "#,
            r#"        .d888888l        "#,
            r#"       .d88888888l       "#,
            r#"      .d8888888888l      "#,
            r#"     .d888888888888l     "#,
            r#"    .d88888888888888l    "#,
            r#"   .d8888888888888888l   "#,
            r#"  .d888888888888888888l  "#,
            r#" .d88888888888888888888l "#,
            r#".d8888888888888888888888l"#,
        ]
    } else if ol.contains("elementary") {
        &[
            r#"         eeeeeeeeeeeeeeeee         "#,
            r#"      eeeeeeeeeeeeeeeeeeeeeee      "#,
            r#"    eeeee  eeeeeeeeeeee   eeeee    "#,
            r#"  eeee   eeeee       eee     eeee  "#,
            r#" eeee   eeee          eee     eeee "#,
            r#"eee    eee            eee       eee"#,
            r#"eee   eee            eee        eee"#,
            r#"ee    eee           eeee       eeee"#,
            r#"ee    eee         eeeee      eeeeee"#,
            r#"ee    eee       eeeee      eeeee ee"#,
            r#"eee   eeee   eeeeee      eeeee  eee"#,
            r#"eee    eeeeeeeeee     eeeeee    eee"#,
            r#" eeeeeeeeeeeeeeeeeeeeeeee    eeeee "#,
            r#"  eeeeeeee eeeeeeeeeeee      eeee  "#,
            r#"    eeeee                 eeeee    "#,
            r#"      eeeeeee         eeeeeee      "#,
            r#"         eeeeeeeeeeeeeeeee         "#,
        ]
    } else if ol.contains("solus") {
        &[
            r#"             `.-:-.`             "#,
            r#"           ./++++++/-.           "#,
            r#"         .:/+++++++++/-          "#,
            r#"        -/++++++++++++/-         "#,
            r#"      `./+++++++++++++++/.       "#,
            r#"     .://+++++++++++++++//:.     "#,
            r#"    .:/+++++++++++++++++++//:.   "#,
            r#"   -///+++++++++++++++++++///-   "#,
            r#"  `////+++++++++++++++++++////`  "#,
            r#"  -////+++++++++++++++++++////-  "#,
            r#"   -///+++++++++++++++++++///-   "#,
            r#"    `://+++++++++++++++++//:`    "#,
            r#"      `-://+++++++++++//:-`      "#,
            r#"         `.-://///:-.`           "#,
        ]
    } else if ol.contains("centos") || ol.contains("rocky") || ol.contains("alma") || ol.contains("rhel") || ol.contains("red hat") {
        &[
            r#"           .          "#,
            r#"          ..          "#,
            r#"         .=.          "#,
            r#"       .=: .          "#,
            r#"     .==:  .=|.       "#,
            r#"    .===:  .===.      "#,
            r#"  .====:   .====.     "#,
            r#" .=====.   .=====.    "#,
            r#".======.   .======.   "#,
            r#".======.   .======.   "#,
            r#".======.   .======.   "#,
            r#".======.   .======.   "#,
            r#" .=====.   .=====.    "#,
            r#"  .====:   .====.     "#,
            r#"    .===:  .===.      "#,
            r#"     .==:  .=|.       "#,
            r#"       .=: .          "#,
            r#"         .=.          "#,
            r#"          ..          "#,
            r#"           .          "#,
        ]
    } else if ol.contains("windows") || ol.contains("wsl") {
        &[
            r#"                                ..,  "#,
            r#"                    ....,,:;+ccllll  "#,
            r#"      ...,,+:;  cllllllllllllllllll  "#,
            r#",cclllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"                                     "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"llllllllllllll  lllllllllllllllllll  "#,
            r#"`'ccllllllllll  lllllllllllllllllll  "#,
            r#"       `' \*::  :ccllllllllllllllll  "#,
            r#"                       ````''*::cll  "#,
        ]
    } else if ol.contains("android") || ol.contains("termux") {
        &[
            r#"      -o          o-       "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
            r#"     +hyd.      .dhy+      "#,
        ]
    } else if ol.contains("freebsd") {
        &[
            r#"   /\,-''''-.    "#,
            r#"  \_)       \   "#,
            r#"  |         |   "#,
            r#"  |  FreeBSD|   "#,
            r#"   ;        /    "#,
            r#"    '-....--'    "#,
        ]
    } else {
        &[
            r#"         _nnnn_        "#,
            r#"        dGGGGMMb       "#,
            r#"       @p~qp~~qMb      "#,
            r#"       M|@||@) M|      "#,
            r#"       @,----.JM|      "#,
            r#"      JS^\__/  qKL     "#,
            r#"     dZP        qKRb   "#,
            r#"    dZP          qKKb  "#,
            r#"   fZP            SMMb "#,
            r#"   HZM            MMMM "#,
            r#"   FqM            MMMM "#,
            r#" __| ".        |\dS"qML"#,
            r#" |    `.       | `' \Zq"#,
            r#"_)      \.___.,|     .'"#,
            r#"\____   )MMMMMP|   .'  "#,
            r#"     `-'       `--'    "#,
        ]
    };
    
    lines.iter().map(|&s| s.to_string()).collect()
}
