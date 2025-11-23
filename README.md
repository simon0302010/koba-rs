# koba-rs

A fast Rust implementation of [koba](https://github.com/simon0302010/koba) — a terminal image renderer that converts images into ASCII/Unicode art.

## Installation

```bash
cargo install koba-rs
```

Or build from source:

```bash
git clone https://github.com/simon0302010/koba-rs
cd koba-rs
cargo build --release
./target/release/koba-rs image.png
```

## Quick Start

```bash
koba-rs image.png
```

## Features

- Color and grayscale rendering
- Custom character sets (ASCII, Unicode, Braille, geometric shapes)
- Custom OpenType and TrueType font support
- Adjustable image scaling and block sizing
- GIF animation support

## Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `IMAGE_PATH` | Path to input image (PNG, JPG, GIF) | Required |
| `-c, --char-range` | Unicode character range (e.g., 32-126) | 32-126 |
| `-s, --scale` | Scale factor for output | 1.0 |
| `--font` | Path to custom OpenType or TrueType font | Unifont |
| `--no-color` | Render in grayscale | false |
| `--no-invert` | Don't invert image for processing | false |
| `--min-size` | Minimum block size for processing | 2.5 |
| `--debug` | Print debug messages | false |

## Credits

- [GNU Unifont](https://unifoundry.com/unifont/index.html) is used for character comparison.

## License

This project is licensed under the GNU General Public License Version 3. See [LICENSE](LICENSE) for details.