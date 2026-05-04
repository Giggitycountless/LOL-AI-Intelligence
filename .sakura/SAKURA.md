# LoL Desktop Assistant 项目概述

## 1. 项目简介

一款面向 Windows 平台的英雄联盟桌面辅助工具，提供本地客户端状态查看、对局分析、选人辅助和自动化勾选等安全可控的功能。

## 2. 技术栈

- 后端核心：Rust
- 桌面框架：Tauri 2
- 前端框架：React 19 + TypeScript 5.9
- 构建工具：Vite 8
- 样式方案：Tailwind CSS 4
- 本地数据库：SQLite（通过 rusqlite 绑定）

## 3. 项目结构

核心目录采用 Rust 工作区组织，前后端职责清晰分离：

crates/
- domain/：共享领域模型、DTO、枚举和纯数据类型
- application/：用例实现、数据校验、业务编排和安全的命令行为
- adapters/：只读的本地 League Client 适配器及平台数据读取
- storage/：SQLite 连接管理、数据库迁移、配置存储、活动日志和用户笔记
- platform/：Tauri 应用初始化、应用状态管理、命令边界定义与命令 DTO

src-tauri/：Tauri 可执行程序的宿主目录，包含资源配置、能力声明和主入口

src/
- App.tsx：React 根组件
- backend/：前端与 Tauri 命令的交互封装
- components/：可复用的 UI 组件
- pages/：主要功能页面
- state/：前端状态管理
- utils/：工具函数
- i18n.ts：国际化配置

tools/ranked-data/：段位数据生成工具

data/ranked-champions/：排位英雄静态数据

docs/：架构设计文档

## 4. 开发约定

安全边界约束：
- 仅允许本地 League Client 访问，严禁前端代码读取锁文件或直接调用 LCU
- 所有 LCU 认证凭据、原始 URL 和 PUUID 不得暴露给 React、DOM、日志、导出或 UI 状态
- 禁止接入 Riot 远程 API
- 自动化功能仅限于用户配置的确认就绪和选人偏好，不包含队列操作、匹配控制、游戏自动化或远程机器人
- League Client 快照不作为产品数据进行持久化

数据存储规范：
- SQLite 仅存储应用自有状态，如用户设置、活动记录和用户创建的游戏笔记/标签

架构分层：
- 后端严格遵循领域驱动分层：domain → application → adapters/storage → platform
- React 前端保持轻量，不携带业务逻辑，仅负责视图渲染和用户交互
- Tauri 命令层作为前后端边界，所有数据交互都通过类型安全的命令 DTO 完成

代码质量：
- 通过 TypeScript 类型检查和 Cargo 工作区单元测试保障质量
- 单元测试覆盖 application 和 platform 核心模块

平台限定：
- 目前以 Windows 为优先平台，构建产物为 NSIS 安装包