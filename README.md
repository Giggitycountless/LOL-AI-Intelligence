# LoL Desktop Assistant

> 基于 Tauri 的英雄联盟桌面助手 · AI 教练 / 聊天预设 / 战绩复盘 / 自动接受 · 全本地数据

![GitHub release](https://img.shields.io/github/v/release/Giggitycountless/LOL-Desktop-Assistant)
![License](https://img.shields.io/github/license/Giggitycountless/LOL-Desktop-Assistant)
![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-blue)
![Tauri](https://img.shields.io/badge/tauri-2.0-%23FFC131?logo=tauri)
![React](https://img.shields.io/badge/react-19-%2361DAFB?logo=react)
![Rust](https://img.shields.io/badge/rust-%F0%9F%A6%80-orange?logo=rust)
![Stars](https://img.shields.io/github/stars/Giggitycountless/LOL-Desktop-Assistant?style=social)

🌐 **[宣传网站](https://giggitycountless.github.io/LOL-Desktop-Assistant/landing)** · **[英文推广文案](docs/promotion-copy.md#-reddit--rleagueoflegends)**

---

## 📥 下载安装

**[👉 最新版本下载 (Releases)](https://github.com/Giggitycountless/LOL-Desktop-Assistant/releases/latest)**

下载 `LoL Desktop Assistant_x.y.z_x64-setup.exe` 双击安装。**Windows 10 / 11 x64** only。

> 聊天预设功能需要**以管理员身份运行**（UIPI 限制），其他功能正常用户权限即可。

---

## ✨ 功能

### 🤖 AI 教练
- **数据分析**：基于近 100 场排位（单双排 + 灵活组排），AI 分析强弱项与改进方向，支持按位置筛选（上 / 打野 / 中 / 下 / 辅）
- **赛后复盘**：单局深度分析，包含双方阵容、伤害构成、视野、经济等全维度数据对比
- **三种风格**：客观分析 / 极端怒喷 / 专业夸夸（可切换）
- 兼容任何 **OpenAI 格式 API**：OpenAI、DeepSeek、通义千问、Moonshot 等

### 💬 聊天预设
- 9 条预设消息绑定 `Ctrl+Shift+1` ~ `Ctrl+Shift+9`
- 游戏内按热键自动打开聊天框并键入消息（**支持中文**）
- 由你按 `Enter` 发送（可编辑、可 `Esc` 取消、`Shift+Enter` 改全聊）

### 🛎️ 自动化
- 自动接受对局（可配延迟）
- 按位置自动选英雄 / Ban
- 锁英雄时自动套用 Tencent 推荐符文页

### 👁️ 游戏内悬浮窗
- 进对局自动显示十人战绩面板（KDA、近期胜率、Scout Score）
- `Shift+Tab` 切换显示 / 隐藏

### 📊 战绩
- 自己近期对局列表 + 详细计分板 + AI 复盘窗口

---

## 🔒 数据与隐私

- 所有数据存在本地 SQLite (`%APPDATA%/com.local.lol-desktop-assistant`)
- 仅访问本机 League Client (LCU)，**不调用 Riot 官方 API、不上传任何数据**
- AI 功能你自己配 API Key，请求走你配置的 endpoint，不经过任何第三方

---

## ⚠️ 已知限制 / 风险提示

- **仅支持 Windows**（macOS / Linux 没有 League 客户端）
- **聊天预设需管理员权限**：否则 Windows UIPI 会拦截向 League 窗口的合成键盘事件
- **国服 Vanguard 反作弊会标记合成输入**：本功能只是代你打字，不影响游戏机制，但风险自担
- **未签名安装包**：Windows SmartScreen 会警告，点"仍要运行"即可

---

## 🛠️ Built With

- [Rust](https://www.rust-lang.org/) — 后端
- [Tauri 2](https://tauri.app/) — 桌面框架（Windows 用 WebView2 + Rust 二进制）
- [React 19](https://react.dev/) + [TypeScript 5.9](https://www.typescriptlang.org/) — 前端
- [Vite 8](https://vitejs.dev/) — 前端构建
- [Tailwind CSS 4](https://tailwindcss.com/) — 样式
- [rusqlite](https://github.com/rusqlite/rusqlite)（bundled SQLite）— 本地存储

---

## 🧱 开发

### 依赖

- Node.js 20+ / npm
- Rust stable MSVC toolchain
- Microsoft C++ Build Tools / Visual Studio Build Tools
- WebView2 Runtime（Windows 11 自带）

### 命令

```powershell
npm install                  # 装前端依赖
npm run dev                  # 开发模式（Vite 热重载 + Rust 后端）
npm run build                # 出 release 安装包到 target/release/bundle/nsis/
npm run typecheck            # TS 类型检查
cargo check --workspace      # Rust 编译检查
cargo test --workspace       # Rust 全部测试
```

### 仓库结构

```text
.
├─ crates/
│  ├─ domain/       # 共享 DTO / enum / 纯模型类型
│  ├─ application/  # use cases、验证、命令编排
│  ├─ adapters/     # 本地 LCU 适配器、外部数据源
│  ├─ storage/      # SQLite 连接、迁移、设置、活动、笔记
│  └─ platform/     # Tauri 命令边界、AppState、命令 DTO
├─ src-tauri/       # Tauri 可执行壳子 + Windows 打包配置
└─ src/
   ├─ backend/      # Tauri 命令的 TS 封装
   ├─ components/   # 通用 React 组件
   ├─ pages/        # Dashboard / Profile / Matches / Advisor / ChatPresets / ...
   ├─ state/        # AppStateProvider + 前端状态边界
   └─ windows/      # Tauri 子窗口辅助函数
```

### 架构分层

- **Domain** 不依赖任何外部 crate
- **Application** 负责验证和业务逻辑
- **Storage** 拥有 SQLite 和迁移
- **Adapters** 拥有外部 / 本地客户端集成细节
- **Platform** 拥有 Tauri 命令和命令 DTO
- **Frontend** 只调用 backend 命令封装，不写业务逻辑

领域术语见 [`CONTEXT.md`](./CONTEXT.md)。

### 发新版本

```powershell
npm run build
gh release create v0.x.0 "target/release/bundle/nsis/LoL Desktop Assistant_0.x.0_x64-setup.exe" `
  --title "v0.x.0" --notes-file release-notes.md
```

---

## 📄 License

[AGPL-3.0-or-later](./LICENSE)

简单说：你可以自由用、改、分发，但**如果你修改后分发（包括作为网络服务）**，必须以同样的 AGPL 协议开源你的修改。
