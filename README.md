# Gelbooru Downloader

A desktop application for searching and downloading images from Gelbooru.

## Features

- Search images by tags using Gelbooru's API
- Download images with concurrent download control
- Local gallery browser with folder tree navigation
- Favorite tags management with hierarchy support
- Configurable download path and proxy settings
- Responsive dark theme UI (naive-ui)

## Prerequisites

- [Node.js](https://nodejs.org/) 18 or higher
- [Rust](https://www.rust-lang.org/) 1.70 or higher
- [pnpm](https://pnpm.io/) 8 or higher

## Installation

### Build from source

```bash
# Clone the repository
git clone <repository-url>
cd gelbooru

# Install frontend dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### Build release

```bash
pnpm tauri build
```

The installer will be created at:
`src-tauri/target/release/bundle/nsis/Gelbooru Downloader 1.0.0.exe`

## Usage

### First launch

On first launch, set your preferred download path in **Settings**.

### Search and download

1. Enter tags in the search bar (e.g., `rating:safe solo`)
2. Browse results and click the download button on images you want
3. Downloads appear in the **Downloads** tab

### Local gallery

Switch to the **Gallery** tab to browse your downloaded images. Use the folder tree to navigate, or search within your local collection.

### Favorite tags

Use the **Favorite Tags** tab to save commonly used tag combinations. Hierarchical tags are supported for organizing tag groups.

## Configuration

Settings are accessible via the Settings tab:

| Option | Default | Description |
|--------|---------|-------------|
| Download path | (empty) | Where images are saved |
| Concurrent downloads | 3 | Max simultaneous downloads |
| Proxy enabled | true | Use system proxy |
| Proxy host | 127.0.0.1 | Proxy server address |
| Proxy port | 7897 | Proxy server port |
| Theme | dark | UI color theme |

## Contributing

### Development setup

```bash
# Install dependencies
pnpm install

# Run linter
pnpm lint

# Run tests
pnpm test        # frontend tests
cargo test       # backend tests

# Run full test suite
pnpm vitest run && cargo test

# Build for release
pnpm tauri build
```

### Code style

- Frontend: ESLint + Prettier (run `pnpm lint:fix` to auto-fix)
- Backend: rustfmt + clippy (run `cargo fmt && cargo clippy -- -D warnings`)

## License

MIT
