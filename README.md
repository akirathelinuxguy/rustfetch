# rustfetch

A minimal yet highly customizable system information fetcher written in Rust. Supports Linux, BSD(should do but havent tested personally), and macOS ( not tested as im to broke for mac os), with GPU detection and OS-specific icons. No external dependencies, easy to extend and configure.

---


## Features
- **System Info**:  OS, kernel, CPU, memory, uptime, boot time
- **GPU Detection**: Supports Linux/with temperature monitoring
- **Bootloader Detection**: Limine, GRUB, systemd-boot  
- **Storage**: Disk usage and partitions with progress bars
- **Network**: IP address and interface
- **Battery**: Percentage and charging status
- **DE/WM**: Desktop environment and window manager
- **Packages**: Count for various package managers
- **RGB Colors**: Colorized output with toggle
- **ASCII Art**: OS-specific logos with external file support
- **Fast**: Caching and parallel processing
- **No Dependencies**: Pure Rust standard library only

---

## Usage

1. Download the source code:

```bash
git clone https://codeberg.org/akirathelinuxguy/rustfetch.git
cd rustfetch
```

2. Build the project: with this exact prompts for its optmisations

```bash
rustc -C opt-level=3 -C target-cpu=native -C lto=fat rustfetch.rs
```

3. Run the binary:

```bash
./rustfetch
```

---



## Configuration
Edit constants in rustfetch.rs top of file

// Display
const USE_COLOR: bool = true;
const PROGRESSIVE_DISPLAY: bool = false;
const CACHE_ENABLED: bool = true;

// Information toggles
const SHOW_GPU: bool = true;
const SHOW_BOOTLOADER: bool = true;
const SHOW_BATTERY: bool = true;
const SHOW_CPU_TEMP: bool = true;
const SHOW_GPU_TEMP: bool = true;
const SHOW_DISKS_DETAILED: bool = true;
// ... more options at top of file

You can also add or modify OS logos in the `get_os_icon()` function for more personalized icons. because im to lazy to add every distro myself

---

## Supported Platforms

- Linux (including various distros) i personally use cachy os therefore it works on arch based
- FreeBSD, OpenBSD, NetBSD
- macOS

*Note:* Uptime and GPU detection may be limited or unavailable on some platforms. ( mac os )

---

## License

This project is licensed under the MIT License idk why it just seems better as its mainly hobby code . See `LICENSE` for details.

---

## Acknowledgements

- Inspired by `neofetch` and `fastfetch` and rusthead propaganda
- Uses native system commands and files for maximum compatibility without dependencies.

---

## Contributing

Contributions are welcome! Feel free to fork and extend the project. but keep it pure rust no libs and sub 10,000 lines if possible

---

## Contact Me 
Do it via issues like a normal person or email me at reubenpercival14@gmail.com i always respond


