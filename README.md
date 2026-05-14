# 👁️ Vigil

**The Universal Supply Chain Health Dashboard.**

> "Is your dependency tree a house of cards? Find out before it collapses."

**Vigil** is a high-fidelity terminal UI (TUI) that gives you an instant, holistic view of your project's supply chain risk. It doesn't just find vulnerabilities; it measures the **vitality** of your dependencies.

---

## ✨ Features

- **🌍 Universal Support**: Analyze Rust (`Cargo.lock`), Node.js (`package-lock.json`), Python (`requirements.txt`), and Go (`go.mod`) in one tool.
- **⚡ Blazing Fast**: 
    - **OSV Batching**: One single network request to check your entire dependency tree for security vulnerabilities.
    - **Persistent Cache**: Subsequent runs are near-instant thanks to a local cache (`~/.cache/vigil`).
- **🩺 Vitality Scoring**: Goes beyond CVEs. Vigil measures maintenance health:
    - **Bloat Index**: Visualize the transitive weight of your dependencies (how many hidden packages are being dragged in).
    - **Staleness**: Real-time "heartbeat" monitoring via crates.io and npm registry APIs.
    - **License Vigilance**: Track and flag restrictive licenses (MIT, Apache, GPL, etc.) across your stack.
- **🛡️ Security First**: Direct, high-speed integration with **OSV.dev** (Open Source Vulnerabilities).
- **📊 Gorgeous TUI**: A premium, interactive terminal dashboard with a dual-pane drill-down view.
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

---

## 💡 How to Read the Dashboard

When you run Vigil, it scans your lockfiles and presents an interactive dual-pane dashboard:

### 1. The Inventory (Left Pane)
- **Dependency**: The name of the package/crate.
- **Version**: The specific version currently locked in your project.
- **Score**: A composite health score (0-100).
    - **Green (80-100)**: Healthy, active, and secure.
    - **Yellow (50-79)**: Minor concerns (e.g., slightly stale or low bus factor).
    - **Red (0-49)**: **Critical Risk**. Usually indicates a known security vulnerability (CVE) or an abandoned project.

### 2. The Drill-Down (Right Pane)
- **Security Status**: Real-time status from OSV database.
- **Dependency Breakdown**: Direct vs. Transitive dependency counts (The Bloat Index).
- **Maintenance Signals**: Live data from registries (Last updated date, total downloads, etc.).
- **License**: Legal status of the dependency.

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

In **2026**, "zero vulnerabilities" is the bare minimum, not the goal. A library with no CVEs can still be a **liability** if it was last updated three years ago or is maintained by a single, overwhelmed individual.

Vigil treats your supply chain like a living organism. It monitors the **vitality** of your dependencies—staleness, bloat, and bus factor—giving you the insight to cut out dead weight before it becomes a crisis. Don't just scan for the past; audit for the future.

---

## 🤝 Contributing

We are in early development! If you want to help build the future of supply chain security, check out our [Contributing Guide](CONTRIBUTING.md).

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for more information.
