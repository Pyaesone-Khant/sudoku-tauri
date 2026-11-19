# Use Tauri v2 (Rust + React/TypeScript) for the macOS Desktop App

We chose Tauri v2 with a Rust backend and React/TypeScript frontend instead of native SwiftUI or Electron. Tauri leverages macOS's built-in WKWebView to keep the app bundle small (~6–10 MB vs Electron's ~150+ MB) while letting us build in our primary languages (Rust and TypeScript) and preserve cross-platform optionality for Windows/Linux later.

## Considered Options

- **Swift + SwiftUI**: Smallest possible bundle (~1–3 MB) and lowest RAM, but locks the codebase to Apple platforms and requires authoring outside our primary Rust/TypeScript stack.
- **Electron**: Familiar web stack, but rejected due to ~150+ MB bundle size and Chromium memory overhead.
