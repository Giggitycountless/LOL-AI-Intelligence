# 项目记忆

## 仓库信息
- 仓库名: Giggitycountless/LOL-Desktop-Assistant
- 累计反思次数: 3

## 常见代码问题与审查要点

### Rust 后端
- **N+1 HTTP 调用**：轮询循环中对每个实体顺序发起 HTTP 请求 → 要求并行化或批量接口
- **静默失败**：`if let Ok(_) = fallible()` 无日志 → 必须添加 `log::warn!` 或 `tracing::debug!`
- **缓存克隆**：`Mutex<HashMap<_, T>>` 中频繁 `.clone()` → 建议使用 `Arc<T>` 共享只读
- **外部依赖脆弱性**：硬编码 CDN URL（如 `/latest/`）无版本协商 → 要求本地回退或版本固定
- **序列化兼容性**：`payload_json` 无 `version` 字段 → 必须伴随 `schema_version`

### 前端 TypeScript/React
- **i18n 缺失**：新增 UI 硬编码字符串 → 必须使用 `t()` 函数
- **竞态未清理**：`setTimeout`/`fetch` 在组件卸载后执行 → 使用 `wasCancelled` 标志或 AbortController
- **Effect 重复加载**：依赖数组变更导致重复请求 → 使用 ref 保存最新值避免重注册
- **浮点格式化**：`NaN`/`Infinity` 暴露给用户 → 必须添加占位符或边界检查

### 测试与迁移
- **测试名实不符**：声称测试特殊字符但未传入真实特殊值 → 审查输入数据
- **异步 rejection 测试**：用 `expect(() => fn()).not.toThrow()` 测试 Promise → 改为 `await expect(fn()).resolves.not.toThrow()`
- **迁移无回滚**：数据库迁移缺少 `down` 脚本 → 要求提供回滚计划

## 近期审查模式总结

| PR | 核心发现 | 规则提炼 |
|----|---------|---------|
| #2 (9/10) | `formatDuration` 未处理 `undefined`；测试名实不符 | 工具函数参数必须覆盖 `null/undefined`；异步 rejection 必须用异步断言 |
| #3 (7/10) | 多词英雄名称映射缺失；缓存 `Clone` 低效 | 手工维护名称列表必检查遗漏；缓存模式区分共享只读 vs 独占修改 |
| #4 (7/10) | 轮询内 N+1 调用；i18n 缺失；无回滚脚本 | 轮询×串行调用优先审查；MVP 阶段 i18n 可降级；迁移强制 `down` |

## 规范建议

### 新增审查规则
- **PERF-N+1** (major)：检测轮询/循环中对每个实体的顺序 HTTP 调用
- **ERROR-SILENT** (major)：检测 `if let Ok` 无日志的静默失败
- **I18N-MISSING** (minor-MVP / major-正式)：新增 UI 硬编码字符串
- **MIGRATION-ROLLBACK** (minor)：数据库迁移缺少 `down` 脚本
- **FLOAT-NAN** (minor)：浮点数输出未处理 `NaN/Inf`

### 强化规则
- 并行化改造后检查 `reqwest::Client` 连接池竞争
- 前端异步操作取消/清理（AbortController / `wasCancelled`）
- JSON 存储结构必须包含 `version` 字段

## 经验教训

**未来审查关注点**：
1. 轮询 × 串行调用 = 性能隐患，优先审查并发模型
2. 手工维护英雄/物品名称列表 → 检查空格、连字符、罗马数字
3. 外部 CDN 依赖 → 要求本地回退或版本固定
4. 行为变更（如 `0` 秒显示）必须明确标记并确认

**值得肯定的实践**：
- 严格 DDD 分层，降低认知负担
- `normalize_advisor_entry` 对每个字段做范围校验 → 可作模板
- 测试 fixture 丰富，保证第三方数据兼容性

## 需要特别关注的领域
- `adapters` 层过重（`lib.rs` 单次 +1068/-24 行）→ 建议拆分模块
- `payload_json` 无版本字段 → 未来结构变更需全量迁移
- 轮询刷新逻辑分散在 `AppStateProvider.tsx` 和 `Advisor.tsx` → 状态同步复杂
- CommunityDragon 解析失败对核心功能的影响边界未评估