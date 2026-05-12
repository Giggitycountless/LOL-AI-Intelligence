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

## 3. 架构设计与关键决策

**分层架构**：严格遵循 DDD 分层 —— domain → application → adapters/storage → platform，React 前端仅负责视图渲染，不携带业务逻辑。

**安全边界**：仅允许本地 League Client 访问，LCU 认证凭据、PUUID 不得暴露给前端/日志/导出；禁止接入 Riot 远程 API。

**数据存储**：SQLite 仅存储用户设置、活动记录和笔记，League Client 快照不持久化。

## 4. 已知问题与注意事项

- **N+1 性能隐患**：轮询周期内对每个实体发起顺序 HTTP 调用，可能导致延迟积压，需并行化或批量请求
- **静默失败倾向**：多处 `if let Ok(_) = ...` 无日志记录，建议显式 `log::warn!`
- **外部依赖脆弱性**：CommunityDragon CDN 无版本协商，需设计本地回退机制
- **前端竞态风险**：轮询刷新时需处理组件卸载后的 `setState`
- **存储兼容性**：`payload_json` 字段缺少 `version`，未来结构变更需全量迁移

## 5. 审查中发现的重要模式

| 模式 | 评价 | 建议 |
|------|------|------|
| 手动 URL 编码 / 英雄名称映射 | 🟡 造轮子 | 抽取公共 `normalize_champion_name` |
| `Mutex<HashMap>` + `Clone` | 🟡 性能隐患 | 改用 `Arc` 共享只读缓存 |
| React Ref 避免 Effect 重跑 | ✅ 优秀 | 推广到类似场景 |
| 测试 fixture 丰富 | ✅ 可推广 | 要求故障注入测试 |

## 6. 团队约定与规范

**代码质量**：
- 所有新增 adapter 必须与现有错误处理风格一致（返回 `Option`/`Result` 并记录日志）
- 静默失败在模块边界必须有 `log::warn!` 或 `tracing::debug!`
- 前端新增 UI 组件必须使用 `t()` 国际化（正式版强制）

**审查规则**（已纳入）：
- PERF-N+1：检测轮询内循环顺序 HTTP 调用
- ERROR-SILENT：检测无日志的 `if let Ok` 静默失败
- MIGRATION-ROLLBACK：迁移文件需提供 `down` 脚本
- 并发缓存：检查 `Clone` vs `Arc` 使用场景

**测试要求**：单元测试覆盖 application/platform 核心模块；故障场景需注入测试（服务不可用、数据格式异常等）。

## 7. 仓库信息

- 仓库名：Giggitycountless/LOL-Desktop-Assistant
- 语言统计：Rust: 433481, TypeScript: 247276
- 累计反思次数：3