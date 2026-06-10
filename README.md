# 🌌 OmniDesk: The Ultimate Hacker Desktop Engine

<div align="center">
  <img src="./example2.gif" alt="OmniDesk Dynamic Demo" width="800" style="border-radius: 12px; box-shadow: 0 8px 24px rgba(0,0,0,0.5); margin-bottom: 20px;">
  <p><em>一个高度可定制、纯本地驱动、基于沙盒隔离的极客桌面微件引擎 (Widget Desktop Engine)</em></p>
</div>

OmniDesk 旨在打破传统桌面的枯燥与死板。通过 **Tauri 2.0 + Rust + React** 的强强联合，它允许你在桌面上挂载无限可能的生产力工具与信息面板。轻量、极速、无边无际。

---

## 📸 效果展示 (Showcase)

<div align="center">
  <img src="./example1.png" alt="OmniDesk Showcase" width="800" style="border-radius: 12px; box-shadow: 0 8px 24px rgba(0,0,0,0.5);">
</div>

---

## 🌟 核心特性 (Core Features)

- 🧩 **全自适应网格系统 (Adaptive Grid Engine)**：摆脱固定宽高比！支持任意拖拽、调整大小，组件间自动碰撞与避让，完美契合你的显示器分辨率，100% 榨干桌面空间。
- 🛡️ **沙盒架构 (Sandbox Isolation)**：每一个 Widget 都在独立的 `iframe` 中运行，互相绝对隔离，彻底杜绝全局 CSS/JS 污染，甚至支持完全不同的技术栈混编！
- 🌐 **底层网络穿透 (CORS-Free Proxy)**：通过 Rust 封装的跨域代理，前端 Widget 可以无视浏览器的 CORS 限制，直击接口，轻松抓取全网数据。
- 🎬 **动态航拍壁纸引擎 (Continuous Streaming Engine)**：内置 Apple TV 官方超清航拍库。采用纯在线流媒体无限轮播模式，配合 WebKitGTK 硬件加速底层优化，0 带宽抢占，纵享世界级航拍大片。
- 🎨 **极致的毛玻璃美学 (Glassmorphism)**：专门为现代混合器 (Compositors) 优化的暗黑模式与高级背景实时模糊效果，每一帧都是壁纸级别。
- 💾 **系统级文件持久化 (Local Persistence)**：提供沙盒级的文件读写接口，让 Widget 都能安全、持久地将用户的隐私配置（如 API Token）保存在系统物理硬盘中。
- 🧘 **禅模式 (Zen Mode)**：在桌面空白处双击鼠标，可通过平滑渐隐动画一键隐藏所有组件及遮罩，瞬间进入沉浸式赏图模式。
- 🖱️ **沉浸式右键设置菜单**：随叫随到的高斯模糊设置中心，自由定制你的专属桌面、微件排列与壁纸。
- 🗂️ **原生桌面集成 (XDG Desktop Integration)**：直读 Linux `~/Desktop` 的文件、文件夹与应用图标（基于图标主题的真实图标 + 图片缩略图），双击启动本地应用 / 打开文件，右键「在文件管理器中显示 / 复制路径 / 移到回收站 / 删除」，并通过 `notify` 实时监听桌面变化自动刷新。

## 📦 内置小组件 (Pre-built Widgets)

> 19 个开箱即用的小组件，涵盖系统监控、开发者工具、AI 资讯、效率提升、桌面集成五大类。每个组件独立沙盒运行，支持热插拔。

### 🖥️ 系统监控

| 组件 | 说明 | 尺寸 |
|------|------|------|
| **System Monitor** | 双环形仪表盘，实时显示 CPU/RAM 占用率 + 网络上下行速率 + Top 进程，支持内联配置 | 4×2 |
| **CPU Top** | CPU 占用最高的进程列表，支持一键结束进程 | 4×4 |
| **Disk Storage** | 环形进度条展示各磁盘使用率，实时监控磁盘 I/O 读写速率 | 6×4 |
| **Network Monitor** | 面积图实时展示网络上下行速率 | 4×2 |

### 🤖 AI & 开发者

| 组件 | 说明 | 尺寸 |
|------|------|------|
| **GitHub PR 监控** | 多仓库 PR 自动轮播，分 Tab 展示「需 Review」和「我提交的」，基于 PAT 防限流 | 4×4 |
| **SSH 连接管理** | 服务器连接管理器，支持分组、一键打开终端 SSH 连接，可配置密码和自定义终端路径 | 4×4 |
| **AI 资讯** | 聚合 HuggingFace 等 AI 行业实时资讯与热门论文 | 4×4 |
| **DeepSeek 余额** | 实时监控大模型 API 余额与用量 | 2×2 |
| **Linux 资讯** | Linux 生态、Qt/KDE 更新动态聚合 | 4×4 |

### 🔥 热点资讯

| 组件 | 说明 | 尺寸 |
|------|------|------|
| **全网热搜** | 微博热搜 + 百度热搜双数据源，Tab 一键切换，实时更新 | 4×4 |

### ⏱️ 效率工具

| 组件 | 说明 | 尺寸 |
|------|------|------|
| **时钟** | 极简数字时钟 | 4×4 |
| **天气预报** | 实时天气信息 | 2×2 |
| **倒数日** | 重要日期倒计时，支持设置每月发工资日自动计算 | 2×2 |
| **番茄钟** | 专注工作计时器 | 2×2 |
| **便签** | 桌面待办便签 | 4×4 |
| **习惯打卡** | 每日习惯追踪与打卡 | 2×2 |
| **快捷指令** | 常用快捷操作面板 | 2×2 |
| **每日提示词** | 每日灵感提示词 | 2×2 |

### 🗂️ 桌面集成

| 组件 | 说明 | 尺寸 |
|------|------|------|
| **桌面文件** | 读取 `~/Desktop` 的文件 / 文件夹 / 应用并按类分组展示，真实主题图标 + 图片缩略图；双击打开文件或启动应用，右键支持「在文件管理器中显示 / 复制路径 / 移到回收站 / 删除（永久, 二次确认）」，`notify` 实时同步桌面增删 | 4×4 |

## 🛠️ 技术栈 (Tech Stack)

- **核心框架**: [Tauri 2.0](https://tauri.app/) (极限的低内存占用与系统原生能力)
- **后端架构**: Rust (`reqwest` 代理, `sysinfo` 系统探针)
- **前端渲染**: React 18 + TypeScript + Vite
- **视觉构建**: Vanilla CSS + 原生 SVG 计算交互

## 🚀 快速启动 (Getting Started)

### 环境要求
- Node.js >= 18
- Rust 工具链 (`rustup`, `cargo`)
- Linux 系统依赖 (如 `libwebkit2gtk-4.1-dev`)

### 运行步骤
```bash
# 1. 安装前端依赖
npm install

# 2. 启动开发服务器与 Rust 后端
npm run tauri dev
```

## 📦 构建 Debian 安装包

```bash
# 构建 release 二进制、生成 deb 包，并规整系统集成文件
npm run build:deb
```

构建产物默认输出到：

```bash
src-tauri/target/release/bundle/deb/tauri-app_0.1.0_amd64.deb
```

安装本地 deb 包：

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/tauri-app_0.1.0_amd64.deb
```

当前 deb 包会安装：

- 主程序：`/usr/bin/tauri-app`
- 用户级 systemd 服务：`/usr/lib/systemd/user/tauri-app.service`
- Deepin 用户会话启动软链接：`/usr/lib/systemd/user/dde-session-core.target.wants/tauri-app.service -> ../tauri-app.service`

OmniDesk 是桌面层组件，不作为普通应用出现在应用启动器里。因此 deb 后处理脚本会移除 Tauri 默认生成的 `/usr/share/applications/tauri-app.desktop`。

安装后，`tauri-app.service` 会通过包内的 `dde-session-core.target.wants` 软链接接入 Deepin 用户会话。用户重启或注销后重新登录时，程序会由 `systemd --user` 随 `dde-session-core.target` 启动。

## 🧩 Deepin 桌面服务接管

Deepin 系统上，如果要先屏蔽系统自带桌面服务，再安装并启动 OmniDesk，可以按下面流程操作。以下命令以当前机器上的桌面服务名为例：

```bash
SYSTEM_DESKTOP_SERVICE='dde-shell-plugin@org.deepin.ds.desktop.service'
OUR_SERVICE='tauri-app.service'
```

先停止当前会话中的系统桌面服务，并全局 mask，避免下次登录时再次启动：

```bash
systemctl --user stop "$SYSTEM_DESKTOP_SERVICE" || true
sudo systemctl --global mask "$SYSTEM_DESKTOP_SERVICE"
```

然后安装 deb 包：

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/tauri-app_0.1.0_amd64.deb
```

在当前已登录会话中立即启动 OmniDesk：

```bash
sudo systemctl --global unmask "$OUR_SERVICE" || true
systemctl --user daemon-reload
systemctl --user start "$OUR_SERVICE"
systemctl --user status "$OUR_SERVICE"
```

重启或注销后重新登录时，`tauri-app.service` 会随 Deepin 用户会话自动启动。

### 恢复系统桌面服务

如果要停用 OmniDesk，并恢复 Deepin 原生桌面服务：

```bash
SYSTEM_DESKTOP_SERVICE='dde-shell-plugin@org.deepin.ds.desktop.service'
OUR_SERVICE='tauri-app.service'
```

先停止并全局 mask OmniDesk：

```bash
systemctl --user stop "$OUR_SERVICE" || true
sudo systemctl --global disable "$OUR_SERVICE" || true
sudo systemctl --global mask "$OUR_SERVICE"
```

恢复系统桌面服务并启动：

```bash
sudo systemctl --global unmask "$SYSTEM_DESKTOP_SERVICE"
systemctl --user daemon-reload
systemctl --user start "$SYSTEM_DESKTOP_SERVICE"
systemctl --user status "$SYSTEM_DESKTOP_SERVICE"
```

## 📂 项目结构指南

- `src-tauri/src/lib.rs`：核心的 Rust 业务逻辑，包含 HTTP Proxy、文件系统挂载与监控数据采集。
- `src/components/GridEngine.tsx`：自适应网格排版系统核心计算引擎。
- `src/App.tsx`：桌面的主入口、路由与状态分发中枢。
- `public/widgets/`：**所有 Widget 的生态老巢！** 
  > 想要开发一个新组件？只需要在这里新建一个文件夹，放入 `manifest.json` 和 `index.html`，即刻获得全套的底层系统能力，无需重新编译主程序即可热插拔生效！

## 📝 License
MIT License
