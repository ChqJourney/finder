# Finder

A powerful and fast file search application built with Tauri, SvelteKit, and TypeScript. This desktop application allows you to quickly search through your files with advanced features and customizable search scenarios.

## Features

### Core Search Functionality
- 🔍 Fast file search across directories
- 📁 Customizable search scenarios with configurable paths
- 🎯 Multiple search targets (filename, content)
- 📊 Search depth level control
- 🗃️ File extension filtering

### User Interface
- 💫 Modern and responsive UI built with SvelteKit
- 🎨 Clean and intuitive search interface
- ⚡ Real-time search results
- 📌 Always-on-top window option
- 🔄 Search history tracking

### Advanced Features
- ⌨️ Global keyboard shortcuts support
- 📋 Clipboard integration
- 🏃‍♂️ Parallel search processing for better performance
- ⏱️ Search timeout protection (30 seconds)
- 🔍 Search cancellation support

### System Integration
- 🖥️ System tray integration
- 🚀 Native performance with Tauri
- 📱 Cross-platform support

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/)

### Installation

1. Clone the repository
```bash
git clone [your-repo-url]
cd finder
```

2. Install dependencies
```bash
npm install
```

3. Run in development mode
```bash
npm run tauri dev
```

4. Build for production
```bash
npm run tauri build
```

## Tech Stack

- **Frontend**: SvelteKit + TypeScript
- **Backend**: Rust + Tauri
- **Build Tool**: Vite
- **Package Manager**: npm

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) with the following extensions:
- [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

[MIT](LICENSE)
