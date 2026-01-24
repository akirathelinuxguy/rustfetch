
# rustfetch

A minimal yet highly customizable system information fetcher written in Rust. Supports Linux, BSD, and macOS, with GPU detection and OS-specific icons. No external dependencies, easy to extend and configure. WHICH USES RUST BTW

---

## Features

- Retrieves hostname, OS, CPU (model, cores, threads), memory, uptime.
- Detects GPU (Linux/BSD/macOS).
- Supports modern distros and custom OS logos.
- Colorized output with toggle.
- Simple configuration via constants.
- No external libraries; fully self-contained.

---

## Usage

1. Download the source code:

```bash
git clone https://codeberg.org/akirathelinuxguy/rustfetch.git
cd rustfetch
```

2. Build the project:

```bash
rustc -C opt-level=3 -C target-cpu=native -C lto=fat rustfetch.rs
```

3. Run the binary:

```bash
./rustfetch
```

---

## Customization

You can tweak the behavior and appearance by editing the constants at the top of `rustfetch.rs`:

```rust
// ==== CONFIGURATION ====
// Toggle GPU detection
const ENABLE_GPU_DETECTION: bool = true;
// Toggle colored output
const USE_COLOR_OUTPUT: bool = true;
```

You can also add or modify OS logos in the `get_os_icon()` function for more personalized icons. because im to lazy to add every distro myself

---

## Supported Platforms

- Linux (including various distros) i personally use cachy os therefore it works on arch based
- FreeBSD, OpenBSD, NetBSD
- macOS

*Note:* Uptime and GPU detection may be limited or unavailable on some platforms. ( mac os )

---

## License

This project is licensed under the MIT License idk why it just seems better. See `LICENSE` for details.

---

## Acknowledgements

- Inspired by `neofetch` and `fastfetch` and rusthead propaganda
- Uses native system commands and files for maximum compatibility without dependencies.

---

## Contributing

Contributions are welcome! Feel free to fork and extend the project.

---

## Contact Me 
Do it via issues like a normal person 
