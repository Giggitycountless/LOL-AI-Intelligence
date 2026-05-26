# Promotion Copy — LoL Desktop Assistant

---

## 🟠 Reddit — r/leagueoflegends

**Title:** I built a free desktop app for League — AI coach, chat presets, match analysis, auto-accept

**Body:**

Hey summoners 👋

I built a Windows desktop app that sits alongside the League client with some features I always wanted:

**What it does:**
- 🤖 **AI Coach** — analyzes your last ~100 ranked games, identifies strengths/weaknesses. 3 styles (objective / roast / praise). Works with any OpenAI-format API.
- 💬 **Chat Presets** — 9 hotkey-bound messages (Ctrl+Shift+1~9). Press a key, it types in chat for you. Supports Chinese.
- 👁️ **In-game Overlay** — 10-player stats panel auto-shows when game starts, Shift+Tab to toggle
- 🛎️ **Auto-accept queue** + pick/ban automation
- 📊 **Match history** with detailed scoreboard + AI post-game analysis
- 🔒 **100% local** — reads your local League Client only, no Riot API, no cloud

**Tech:** Tauri 2 + React 19 + Rust + Vite 8

**Download:** [GitHub Releases](https://github.com/Giggitycountless/LOL-Desktop-Assistant/releases)
**Source:** [github.com/Giggitycountless/LOL-Desktop-Assistant](https://github.com/Giggitycountless/LOL-Desktop-Assistant)

⚠️ Not affiliated with Riot. No gameplay automation — this is a companion tool. Chat presets need admin rights (Windows UIPI). Vanguard on CN server may flag synthetic input — use at your own risk.

---

## 🟣 Reddit — r/tauri

**Title:** I shipped a Tauri 2 League of Legends desktop app — 29 Rust commands, multi-window, global shortcuts

**Body:**

Just released v0.2.0 of my League of Legends desktop assistant, built entirely with Tauri 2.

**Tech highlights:**
- 5 Rust crates (domain, application, adapters, storage, platform)
- 29 Tauri commands
- React 19 + Vite 8 + Tailwind CSS 4 frontend
- SQLite via rusqlite
- NSIS installer for Windows
- Multi-window: main window + frameless overlay for in-game stats
- tauri-plugin-global-shortcut for Shift+Tab overlay toggle
- HTTP adapter to League Client's local HTTPS with self-signed cert handling
- Exponential backoff retry on all LCU HTTP calls
- RAII guards to prevent automation from getting stuck on panic
- Chat presets via Windows SendInput (requires admin for UIPI bypass)

**Repo:** [github.com/Giggitycountless/LOL-Desktop-Assistant](https://github.com/Giggitycountless/LOL-Desktop-Assistant)

Happy to answer questions about Tauri 2 multi-window patterns, global shortcuts, or LCU integration.

---

## ✍️ Dev.to Article

**Title:** Building a League of Legends Desktop Assistant with Tauri 2, React 19, and Rust

**Tags:** rust, tauri, react, gamedev, typescript

**Body (outline):**

### Why I built this
During champ select I'd constantly alt-tab to check match history. Web tools were slow and needed API keys. I wanted something fast, local, and open source.

### Features
- AI Coach with multi-model support
- 9 chat presets with global hotkeys
- In-game overlay with 10-player stats
- Auto-accept and pick/ban automation
- Match history + AI post-game analysis

### Tech stack
Tauri 2 · React 19 · Rust · Vite 8 · Tailwind CSS 4 · SQLite

### How it talks to League Client
The LCU runs a local HTTPS server with a self-signed cert. The Rust backend reads the lockfile, makes authenticated requests, and never exposes tokens to the frontend.

### Multi-window with global shortcuts
The in-game overlay is a separate frameless Tauri window, toggled by Shift+Tab via tauri-plugin-global-shortcut.

### Chat presets via SendInput
Windows UIPI blocks synthetic keyboard events to elevated windows (League runs as admin). The chat preset feature uses SendInput and requires admin rights to bypass this.

**Check it out:** [github.com/Giggitycountless/LOL-Desktop-Assistant](https://github.com/Giggitycountless/LOL-Desktop-Assistant)

---

## 🇨🇳 Bilibili / NGA 文案

**标题：** 我用 Tauri 2 写了个英雄联盟桌面助手——AI教练/聊天预设/战绩复盘/自动接受

**正文：**

兄弟们好，在新西兰留学的 CS 学生。打 LOL 的时候老想有个工具能直接在游戏里看数据，干脆自己写了一个。

**功能：**
- 🤖 AI 教练——分析近 100 场排位，三种风格（客观/怒喷/夸夸）随心切
- 💬 聊天预设——Ctrl+Shift+1~9 一键发消息，支持中文
- 👁️ 游戏内悬浮窗——进游戏自动显示十人战绩面板
- 🛎️ 自动接受对局 + 选人/禁人自动化
- 📊 战绩复盘 + AI 赛后分析
- 🔒 全本地数据，不上传、不调 Riot API

**技术栈：** Tauri 2 + React 19 + Rust + Vite 8

**下载：** [GitHub Releases](https://github.com/Giggitycountless/LOL-Desktop-Assistant/releases)
**源码：** [github.com/Giggitycountless/LOL-Desktop-Assistant](https://github.com/Giggitycountless/LOL-Desktop-Assistant)

⚠️ 聊天预设需要管理员权限运行（Windows UIPI 限制）。国服 Vanguard 可能会标记合成输入，自行评估风险。

开源 AGPL-3.0，求 Star 求反馈 🙏
