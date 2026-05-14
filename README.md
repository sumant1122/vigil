# 👁️ Vigil

**The Universal Supply Chain Health Dashboard.**

> "Is your dependency tree a house of cards? Find out before it collapses."

**Vigil** is a high-fidelity terminal UI (TUI) that gives you an instant, holistic view of your project's supply chain risk. It doesn't just find vulnerabilities; it measures the **vitality** of your dependencies.

---

## ✨ Features

- **🌍 Universal Support**: Analyze Rust (`Cargo.lock`), Node.js (`package-lock.json`), Python (`requirements.txt`), and Go (`go.mod`) in one tool.
- **🩺 Vitality Scoring**: Goes beyond CVEs. Vigil measures maintenance health:
    - **Bus Factor**: How many people are actually maintaining this?
    - **Staleness**: When was the last heartbeat (commit)?
    - **Unsafe Density**: (Rust-specific) How much `unsafe` code are you inheriting?
- **🛡️ Security First**: Direct integration with **OSV.dev** (Open Source Vulnerabilities) for real-time security alerts.
- **📊 Gorgeous TUI**: A premium, interactive terminal dashboard built with `ratatui`.
- **🚀 Zero Config**: Run `vigil` in any repo, and it automatically detects your stack.

---

## 🚀 Installation

### From Source
Ensure you have Rust and Cargo installed, then run:

```bash
git clone https://github.com/sumant1122/vigil.git
cd vigil
cargo install --path .
```

---

## ⚡ Usage

Vigil is designed to be **Zero-Config**. Just navigate to your project's root and run:

```bash
vigil
```

### Advanced Usage

Analyze a specific project directory:
```bash
vigil --path /path/to/your/project
```

Check the version:
```bash
vigil --version
```

---

## 💡 How to Read the Dashboard

When you run Vigil, it scans your lockfiles and presents a table with the following columns:

- **Dependency**: The name of the package/crate.
- **Version**: The specific version currently locked in your project.
- **Ecosystem**: Which language ecosystem it belongs to (Cargo, NPM, Pip, Go).
- **Score**: A composite health score (0-100).
    - **Green (80-100)**: Healthy, active, and secure.
    - **Yellow (50-79)**: Minor concerns (e.g., slightly stale or low bus factor).
    - **Red (0-49)**: **Critical Risk**. Usually indicates a known security vulnerability (CVE) or an abandoned project.

---

## 🛠️ Supported Ecosystems

| Ecosystem | Detected File |
| :--- | :--- |
| **Rust** | `Cargo.lock` |
| **Node.js** | `package-lock.json` (v2+) |
| **Python** | `requirements.txt` |
| **Go** | `go.mod` |

---

## 🛠️ Why Vigil?

In 2024, a "secure" dependency isn't enough. A library might have zero CVEs today but be completely abandoned by its maintainers, leaving you with a ticking time bomb.

Vigil treats your supply chain like a living organism. It monitors the "pulse" of your dependencies so you can make informed decisions about your stack.

---

## 🤝 Contributing

We are in early development! If you want to help build the future of supply chain security, check out our [Contributing Guide](CONTRIBUTING.md).

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.
