<div align="center">

# 🛡️ CodePack

### Your Local AI Code Auditor

**Stop pasting code manually. Audit before you push.**

*The desktop app that turns your codebase into AI-ready context — with built-in secret scanning, smart Git integration, and reviewer personas.*

[![Built with Tauri](https://img.shields.io/badge/Built%20with-Tauri%20v2-blue?logo=tauri)](https://tauri.app)
[![Rust Backend](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)](https://www.rust-lang.org)
[![Vue 3 Frontend](https://img.shields.io/badge/Frontend-Vue%203-green?logo=vuedotjs)](https://vuejs.org)

</div>

---

## Why CodePack?

Every developer copy-pastes code into ChatGPT. But doing it manually is **slow**, **dangerous** (you might leak API keys), and **wasteful** (you send irrelevant files that burn tokens).

CodePack solves all three:

| Problem | CodePack Solution |
|---------|-------------------|
| 🐌 Manually selecting & copying files | **Drag-and-drop** your project, check the files you need, one-click export |
| 🔑 Accidentally leaking secrets to AI | **Secret Scanner** detects API keys before export, one-click redaction |
| 💸 Wasting tokens on unchanged code | **Git Changed Only** mode — review just what you wrote |
| 🎯 AI gives generic, unfocused reviews | **Reviewer Personas** — Security Expert, Performance Optimizer, Clean Code |
| 🌐 Data sent through third-party servers | **100% local processing** — your code never leaves your machine* |

<sub>*API calls go directly from your machine to the AI provider. No middleman.</sub>

---

## ✨ Core Features

### 🛡️ Secret Scanning — Audit Before You Send

CodePack scans your code for **API keys, passwords, private keys, and tokens** before export.

- **6 detection rules** — AWS Keys, OpenAI Keys, GitHub PATs, Google API Keys, SSH Private Keys, hardcoded passwords
- **Visual warnings** — ⚠️ badges on risky files in the tree, red line highlights in preview
- **One-click redaction** — mask secrets as `AKI******` before sending to AI
- **Export interception** — blocking dialog prevents accidental leaks: *"Auto-redact and copy"* / *"Copy anyway"* / *"Cancel"*

### 🔀 Smart Git Integration — Review What Matters

- **`[Changed]` toggle** — one click to select only Git-modified files
- **Include Diff** checkbox — embed unified diffs in your export so AI sees *what changed*, not just *all the code*
- Branch name and change count displayed in the header

### 🎭 Reviewer Personas — Focus the AI's Attention

Pre-built review instructions that append to your export:

| Persona | Focus |
|---------|-------|
| 🔒 **Security Expert** | Injection risks, auth vulnerabilities, hardcoded secrets, input validation |
| ⚡ **Performance Optimizer** | Algorithm complexity, memory leaks, N+1 queries, caching opportunities |
| 🧹 **Clean Code** | SOLID principles, code smells, naming, DRY, error handling |

Create your own custom personas with any instruction text.

### 📦 Intelligent Packing

- **15+ project types** auto-detected (Rust, Go, Python, Node.js, Flutter, Android, Java, C++, Swift...)
- **Smart exclusions** — `node_modules`, `build`, `dist`, `.git`, `__pycache__`, `target`, `venv` etc.
- **3 export formats** — Plain Text, Markdown, XML
- **Token estimation** — real-time count with context limit warnings
- **Syntax highlighting** — Shiki-powered code preview
- **File presets** — save/load different file selections per project
- **Plugin system** — extend project detection with custom JSON rules
- **Statistics panel** — language distribution, line counts, size breakdown

---

## 🖥️ Screenshots

> *Screenshots coming soon — the app features a dark-themed UI with:*
> - *Left panel: file tree with checkboxes, ⚠️ secret badges, and Git status*
> - *Top bar: reviewer persona selector (🔒 ⚡ 🧹)*
> - *Right panel: syntax-highlighted code preview with secret line highlighting*
> - *Bottom bar: token count, format selector, Diff toggle, copy/export buttons*
> - *Security dialog: blocking prompt before exporting files with detected secrets*

---

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)

### Install & Run

```bash
git clone https://github.com/ramyaramyarao11278-cmd/codepack_92.git
cd codepack_92
npm install
npx tauri dev
```

### Build for Production

```bash
npx tauri build
```

---

## 🗺️ Roadmap

- [x] Secret scanning with one-click redaction
- [x] Git integration (changed files toggle + diff embedding)
- [x] Reviewer personas (builtin + custom)
- [x] Multi-format export (Plain / Markdown / XML)
- [x] Token estimation with context limit warnings
- [x] File presets and plugin system
- [ ] **Direct AI API integration** — call OpenAI / DeepSeek / Anthropic from the app
- [ ] **Streaming Review UI** — render AI review results as Markdown in-app
- [ ] **API Key management** — secure storage in OS Keychain
- [ ] **Code compression** — Tree-sitter AST skeleton mode to reduce tokens
- [ ] **Review history** — save and compare past reviews

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────┐
│                   Vue 3 + Tailwind               │
│  FileTree │ CodePreview │ ReviewPromptBar │ ...  │
├─────────────────────────────────────────────────┤
│               Pinia State Management             │
│  useProjectStore │ useUIStore │ useToast         │
├─────────────────────────────────────────────────┤
│                 Tauri v2 Bridge                   │
├─────────────────────────────────────────────────┤
│                   Rust Backend                    │
│  scanner │ security │ packer │ git │ config      │
│  metadata │ stats │ plugins │ watcher            │
└─────────────────────────────────────────────────┘
```

- **Rust backend** — file scanning, secret detection (regex), Git operations (libgit2), packing, token counting (tiktoken)
- **Vue 3 frontend** — reactive file tree, syntax highlighting (Shiki), real-time preview
- **Tauri v2** — native desktop performance, no Electron bloat

---

## 📄 License

MIT
