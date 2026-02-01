use std::{
    env,
    fs,
    path::Path,
    process::Command,
    thread,
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

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
    let config = match parse_args() {
        Some(cfg) => cfg,
        None => return,
    };
    
    if config.benchmark {
        run_benchmarks(&config);
        return;
    }
    
    let start_time = std::time::Instant::now();
    let net_start = if config.show_network {
        read_file_trim("/proc/net/dev")
    } else {
        None
    };

    let info = thread::scope(|s| {
        let cfg1 = config.clone();
        let t1 = s.spawn(move || {
            let user = get_user();
            let hostname = get_hostname();
            let os = get_os();
            let kernel = get_kernel();
            let uptime = if cfg1.show_uptime { get_uptime() } else { None };
            let shell = if cfg1.show_shell { get_shell() } else { None };
            let de = if cfg1.show_de { get_de() } else { None };
            let init = if cfg1.show_init { get_init() } else { None };
            let terminal = if cfg1.show_terminal { get_terminal() } else { None };
            let display = if cfg1.show_display { get_display() } else { None };
            let model = if cfg1.show_model { get_model() } else { None };
            let motherboard = if cfg1.show_motherboard { get_motherboard() } else { None };
            let bios = if cfg1.show_bios { get_bios() } else { None };
            let locale = if cfg1.show_locale { get_locale() } else { None };
            let theme_info = if cfg1.show_theme || cfg1.show_icons || cfg1.show_font {
                get_theme_info()
            } else {
                ThemeInfo { theme: None, icons: None, font: None }
            };
            let resolution = if cfg1.show_resolution { get_resolution() } else { None };
            (user, hostname, os, kernel, uptime, shell, de, init, terminal, display, 
             model, motherboard, bios, locale, theme_info, resolution)
        });
        
        let cfg2 = config.clone();
        let t2 = s.spawn(move || {
            let cpu_info = get_cpu_info_combined();
            let cpu_temp = if cfg2.show_cpu_temp && !cfg2.fast_mode { 
                get_cpu_temp() 
            } else { 
                None 
            };
            let memory = if cfg2.show_memory { get_memory() } else { None };
            let swap = if cfg2.show_swap { get_swap() } else { None };
            let battery = if cfg2.show_battery { get_battery() } else { None };
            let processes = if cfg2.show_processes { get_processes() } else { None };
            let users = if cfg2.show_users { get_users_count() } else { None };
            let entropy = if cfg2.show_entropy { get_entropy() } else { None };
            (cpu_info, cpu_temp, memory, swap, battery, processes, users, entropy)
        });
        
        let cfg3 = config.clone();
        let t3 = s.spawn(move || {
            let gpus = if cfg3.show_gpu { get_gpu() } else { None };
            let gpu_temps = if cfg3.show_gpu && !cfg3.fast_mode {
                get_gpu_temp_with_gpus(gpus.as_ref())
            } else {
                None
            };
            let gpu_vram = if cfg3.show_gpu_vram { get_gpu_vram() } else { None };
            (gpus, gpu_temps, gpu_vram)
        });
        
        let cfg4 = config.clone();
        let t4 = s.spawn(move || {
            let packages = if cfg4.show_packages { get_packages() } else { None };
            let partitions = if cfg4.show_partitions { get_partitions_impl() } else { None };
            let boot_time = if cfg4.show_boot_time { get_boot_time() } else { None };
            let bootloader = if cfg4.show_bootloader { get_bootloader() } else { None };
            let wm = if cfg4.show_wm { get_wm() } else { None };
            let public_ip = if cfg4.show_public_ip && !cfg4.fast_mode { 
                get_public_ip() 
            } else { 
                None 
            };
            let failed_units = if cfg4.show_failed_units { get_failed_units() } else { None };
            (packages, partitions, boot_time, bootloader, wm, public_ip, failed_units)
        });
        
        let (user, hostname, os, kernel, uptime, shell, de, init, terminal, display, 
             model, motherboard, bios, locale, theme_info, resolution) = t1.join().unwrap();
        let (cpu_info, cpu_temp, memory, swap, battery, processes, users, entropy) = t2.join().unwrap();
        let (gpu, gpu_temps, gpu_vram) = t3.join().unwrap();
        let (packages, partitions, boot_time, bootloader, wm, public_ip, failed_units) = t4.join().unwrap();
        
        let delta = start_time.elapsed().as_secs_f64();
        let network = if config.show_network {
            get_network_final(net_start, delta, config.show_network_ping)
        } else {
            None
        };

        Info {
            user, hostname, os, kernel, uptime, shell, de, wm, init, terminal,
            cpu: cpu_info.name,
            cpu_temp,
            cpu_cores: if cpu_info.cores.is_some() && cpu_info.threads > 0 {
                Some((cpu_info.cores.unwrap_or(cpu_info.threads), cpu_info.threads))
            } else {
                None
            },
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
    
    if config.json_output {
        println!("{}", info.to_json());
    } else {
        render_output(&info, &config);
    }
    
    if config.cache_enabled {
        save_cache(&info);
    }
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
    bench!("Memory", get_memory());
    bench!("Swap", get_swap());
    bench!("Partitions", get_partitions_impl());
    bench!("Display", get_display());
    bench!("Battery", get_battery());
    bench!("Model", get_model());
    bench!("Motherboard", get_motherboard());
    bench!("BIOS", get_bios());
    bench!("Theme info", get_theme_info());
    bench!("Processes", get_processes());
    bench!("Users", get_users_count());
    bench!("Entropy", get_entropy());
    bench!("Locale", get_locale());
    bench!("Resolution", get_resolution());
    bench!("Failed units", get_failed_units());
    bench!("GPU", get_gpu());
    
    if !config.fast_mode {
        println!("\nExpensive operations (skipped in --fast mode):");
        bench!("CPU temp", get_cpu_temp());
        bench!("Public IP", get_public_ip());
        let gpus = get_gpu();
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
    env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            run_cmd("tput", &["cols"]).and_then(|s| s.parse().ok())
        })
        .unwrap_or(80)
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
    
    None
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

fn get_gpu() -> Option<Vec<String>> {
    let mut gpus = Vec::with_capacity(2);
    
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
    if total > 0.0 { Some((total - free, total)) } else { None }
}

fn get_gpu_vram() -> Option<Vec<String>> {
    if let Some(output) = run_cmd("lspci", &["-v"]) {
        let mut vrams: Vec<String> = Vec::with_capacity(2);
        let mut current_gpu_vram: Option<String> = None;
        
        for line in output.lines() {
            if line.contains("VGA compatible controller") || line.contains("Display controller") || line.contains("3D controller") {
                if let Some(vram) = current_gpu_vram { vrams.push(vram); }
                current_gpu_vram = None;
            }
            if line.contains("Memory at") && line.contains("size=") {
                if let Some(pos) = line.find("size=") {
                    let size_part = &line[pos+5..];
                    if let Some(end) = size_part.find(']') {
                        let size_str = size_part[..end].to_string();
                        let size_val = parse_human_size(&size_str).unwrap_or(0.0);
                        let current_val = current_gpu_vram.as_ref().and_then(|v| parse_human_size(v)).unwrap_or(0.0);
                        if size_val > current_val {
                            current_gpu_vram = Some(size_str);
                        }
                    }
                }
            }
        }
        if let Some(vram) = current_gpu_vram { vrams.push(vram); }
        if !vrams.is_empty() { return Some(vrams); }
    }
    None
}

fn get_resolution() -> Option<String> {
    if let Some(out) = run_cmd("wlr-randr", &[]) {
        for line in out.lines() {
            if line.contains(" px, ") && line.contains(" Hz") { return Some(line.trim().to_string()); }
        }
    }
    if let Some(out) = run_cmd("xrandr", &["--current"]) {
        for line in out.lines() {
            if line.contains(" connected") && (line.contains(" primary") || !line.contains(" disconnected")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, &p) in parts.iter().enumerate() {
                    if p.contains('x') && p.chars().next().unwrap_or(' ').is_ascii_digit() && i > 0 {
                        return Some(p.to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_entropy() -> Option<String> {
    let avail = read_file_trim("/proc/sys/kernel/random/entropy_avail")?;
    let pool = read_file_trim("/proc/sys/kernel/random/poolsize").unwrap_or_else(|| "4096".to_string());
    Some(format!("{}/{}", avail, pool))
}

fn get_users_count() -> Option<usize> {
    if let Some(out) = run_cmd("who", &[]) {
        let count = out.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 { return Some(count); }
    }
    
    if let Some(out) = run_cmd("w", &["-h"]) {
        let count = out.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 { return Some(count); }
    }

    if let Some(out) = run_cmd("users", &[]) {
        let count = out.split_whitespace().count();
        if count > 0 { return Some(count); }
    }

    if let Some(out) = run_cmd("loginctl", &["list-users", "--no-legend"]) {
        let count = out.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 { return Some(count); }
    }

    Some(1)
}

fn get_failed_units() -> Option<usize> {
    run_cmd("systemctl", &["list-units", "--failed", "--no-legend", "--no-pager"])
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
}

fn get_partitions_impl() -> Option<Vec<(String, String, f64, f64)>> {
    if let Some(output) = run_cmd("df", &["-hT", "/"]) {
        for line in output.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 6 {
                let source = fields[0];
                let fstype = fields[1];
                
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
    
    if let Some(theme) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "gtk-theme"]) {
        info.theme = Some(theme.trim_matches('\'').to_string());
    }
    if let Some(icons) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "icon-theme"]) {
        info.icons = Some(icons.trim_matches('\'').to_string());
    }
    if let Some(font) = run_cmd("gsettings", &["get", "org.gnome.desktop.interface", "font-name"]) {
        info.font = Some(font.trim_matches('\'').to_string());
    }

    if info.theme.is_none() || info.font.is_none() {
        if let Ok(home) = env::var("HOME") {
            let kdeglobals = format!("{}/.config/kdeglobals", home);
            if let Ok(content) = fs::read_to_string(kdeglobals) {
                for line in content.lines() {
                    if line.starts_with("widgetStyle=") && info.theme.is_none() {
                        info.theme = Some(line.split('=').nth(1).unwrap_or("").to_string());
                    } else if line.starts_with("font=") && info.font.is_none() {
                        info.font = Some(line.split('=').nth(1).unwrap_or("").split(',').next().unwrap_or("").to_string());
                    }
                }
            }
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

fn get_network_final(net_start: Option<String>, delta: f64, should_ping: bool) -> Option<Vec<NetworkInfo>> {
    let dev1 = net_start?;
    let dev2 = fs::read_to_string("/proc/net/dev").ok()?;
    
    let mut stats1 = HashMap::new();
    for line in dev1.lines().skip(2) {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.len() > 9 { stats1.insert(p[0].trim_end_matches(':').to_string(), (p[1].parse::<u64>().unwrap_or(0), p[9].parse::<u64>().unwrap_or(0))); }
    }

    let mut ip_map = HashMap::new();
    if let Some(output) = run_cmd("ip", &["-o", "addr", "show"]) {
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
                    .find(|w| w.contains('x') && w.chars().next().unwrap_or('a').is_numeric())
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
