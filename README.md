# rustfetch

A minimal system information fetcher written in Rust. Fast, dependency-free, and highly customizable.

## Features

- **System Info**: Hostname, OS, kernel, CPU, memory, uptime, boot time
- **GPU Detection**: Supports Linux, BSD, macOS
- **Bootloader**: Detects Limine, GRUB, systemd-boot, and others
- **Storage**: Disk usage and partitions with progress bars
- **Network**: Current IP address and interface
- **Battery**: Percentage and charging status
- **Colors**: RGB color support with toggle
- **Fast**: Caching and parallel processing
- **No Dependencies**: Pure Rust, standard library only

## Install

```bash
# Download
git clone https://github.com/akirathelinuxguy/rustfetch.git
cd rustfetch

# Build
rustc -C opt-level=3 rustfetch.rs

# Run
./rustfetch
Or with Cargo:

bash
cargo build --release
./target/release/rustfetch
Configuration
Edit the constants in rustfetch.rs:

rust
// Toggle features
const USE_COLOR: bool = true;
const SHOW_GPU: bool = true;
const SHOW_BOOTLOADER: bool = true;
const SHOW_BATTERY: bool = true;
// ... more options at top of file
ASCII Art
Add your own logos:

Create /usr/share/rustfetch/logos/your-os.ascii

Or edit the get_logo() function

Built-in logos for:

Arch Linux / CachyOS

Ubuntu

Debian

Fedora

Windows

macOS

Linux (generic)

And more

Supported Systems
Linux: Arch, Ubuntu, Debian, Fedora, etc. (I use CachyOS)

BSD: FreeBSD, OpenBSD, NetBSD

macOS: Limited GPU support

Package Managers
pacman (Arch)

dpkg (Debian/Ubuntu)

rpm (Fedora/RHEL)

xbps (Void)

And others

Example
text
      /\        user@host
     /  \       ─────────
    /\   \      OS: CachyOS
   /  \   \     Kernel: 6.18.7
  /    \   \    Uptime: 1h 30m
 /______\___\   Bootloader: Limine
                CPU: Intel Core i7 (8 cores)
                GPU: NVIDIA RTX 3060
                Memory: 4.2/15.6 GiB [████░░░░░░░░░░░░░░] 27%
No Dependencies
This tool uses only Rust's standard library. No external crates.

Building for Speed
bash
rustc -C opt-level=3 -C target-cpu=native rustfetch.rs
Contributing
Keep it:

Dependency-free

Under 2000 lines

Fast and simple

License
MIT License

Contact
GitHub Issues

Email: reubenpercival14@gmail.com
