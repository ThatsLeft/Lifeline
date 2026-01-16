# Lifeline Timeline

A beautiful, interactive timeline application built with Rust, egui, and WebAssembly. Features animated cosmic backgrounds with stars, galaxies, and nebulas, along with burning star markers for timeline events.

![Lifeline Timeline](screenshot.png)

## Features

- **Interactive Timeline**: Add events with full date stamps (day, month, year)
- **Image Support**: Attach images to events (file picker on both native and web)
- **Animated Background**: Beautiful cosmic scenery with parallax effects
- **Camera Controls**: 
  - WASD for panning
  - Mouse wheel for zooming
  - Click events to freeze their animation
- **Cross-Platform**: Runs as native desktop app or in web browsers

## Live Demo

Visit the live demo at: `https://<your-username>.github.io/lifeline`

## Running Locally

### Native Desktop App

Requirements:
- Rust (latest stable)

```bash
# Run in development mode
cargo run

# Build release version
cargo build --release
./target/release/lifeline
```

### Web Version (WASM)

Requirements:
- Rust (latest stable)
- Trunk (`cargo install trunk`)

```bash
# Serve with hot reload
trunk serve

# Build for production
trunk build --release
```

The built files will be in the `dist/` directory.

## Deploying to GitHub Pages

1. Push your code to GitHub
2. Go to repository Settings → Pages
3. Set Source to "GitHub Actions"
4. Push to `master` or `main` branch
5. GitHub Actions will automatically build and deploy

The workflow is configured in `.github/workflows/deploy.yml`

## Usage

### Adding Events

1. Click "Add Event ▲" at the bottom
2. Fill in:
   - Title
   - Description
   - Day (1-31)
   - Month (1-12)
   - Year
   - Optional: Image (click "Browse..." to upload)
3. Click "Add to Timeline" or "Today" for current date

### Navigation

- **WASD**: Pan the camera (disabled when typing)
- **Mouse Wheel**: Zoom in/out
- **Click Event**: Freeze/unfreeze animation
- **Hover Event**: View details in tooltip

## Technology Stack

- **Language**: Rust
- **GUI Framework**: egui + eframe
- **Graphics**: OpenGL via glow
- **WASM**: wasm-bindgen
- **Build Tool**: Trunk
- **Image Processing**: image crate
- **File Picker**: rfd (native and WASM)

## Project Structure

```
lifeline/
├── src/
│   ├── main.rs           # App entry point and UI
│   ├── timeline.rs       # Event data structures
│   ├── event_renderer.rs # Event rendering and animation
│   └── stars.rs          # Cosmic background effects
├── index.html            # HTML template for WASM
├── Cargo.toml           # Dependencies
└── .github/
    └── workflows/
        └── deploy.yml   # GitHub Pages deployment
```

## License

MIT

## Credits

Built with ❤️ using Rust and egui
