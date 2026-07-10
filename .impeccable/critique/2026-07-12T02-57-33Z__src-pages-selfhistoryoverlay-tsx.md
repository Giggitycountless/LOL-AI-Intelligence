---
target: 悬浮窗 SelfHistoryOverlay
total_score: 25
p0_count: 0
p1_count: 4
timestamp: 2026-07-12T02-57-33Z
slug: src-pages-selfhistoryoverlay-tsx
---
# SelfHistoryOverlay 评审快照

总分 25/40（可接受）。检测器命中 1 处：ChampionDetailsPanel.tsx:124 border-l-4 side-stripe。

## 优先问题
- [P1] 评分为绝对数字+固定刻度条(score/220)，核心任务「找最弱敌人」需人工心算排名。修复：按本局10人归一化条形或标注每队最高/最低。
- [P1] playerBadge 中 Math.max(1, wins) 使 0 胜显示为 1（数据造假）；我方徽章=胜场、敌方=败场，同位置语义翻转且无标签。
- [P1] 悬浮窗无鼠标关闭途径：Hide 按钮已从 header 消失（overlay.hide i18n 死键，"shows hide button" 测试长期失败即回归证据）。
- [P1] 红绿为胜负/敌我唯一编码，色盲不可读。战绩行加胜/负文字，队伍板加我方/敌方标题。
- [P2] 主窗口全线支持 dark: 变体，悬浮窗零支持；夜间为白色巨窗覆盖游戏。可考虑悬浮窗默认深色。

## 次要
- 玩家卡内嵌套四层圆角边框盒；评分盒/顾问盒降级为无边框分区。
- 空位卡 text-zinc-400 on zinc-50 约 2.7:1，低于 4.5:1。
- 9px 字号（等级、M 徽章）在覆盖游戏场景过小。
- 评分盒角标与底部 W/G 重复。
- 段位 tooltip 可扩展为胜负场次（wins/losses 字段未使用）。
- S/F 单字母标签易与 S 评价混淆，建议 单双/灵活。
- LiveOverlayBar 直出 ChampionKill 原始事件名，未本地化。
- 顾问盒条件渲染导致 10 列战绩行起始高度不齐。
- header rose-700 圆点与 rose=敌方语义轻微冲突。

## 启发式得分
状态可见3 / 真实世界2 / 用户控制2 / 一致性2 / 错误预防3 / 识别2 / 灵活3 / 极简3 / 恢复3 / 帮助2 = 25/40
