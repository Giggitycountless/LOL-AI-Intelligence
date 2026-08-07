---
target: 游戏内悬浮窗 SelfHistoryOverlay
total_score: 27
p0_count: 1
p1_count: 1
timestamp: 2026-08-07T04-05-04Z
slug: src-pages-selfhistoryoverlay-tsx
---
#### Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | 不变——刷新转圈、超时兜底、失败提示仍在 |
| 2 | Match System / Real World | 3 | 不变 |
| 3 | User Control and Freedom | 3 | 不变 |
| 4 | Consistency and Standards | 2 | 色彩语义迁移只覆盖了上次点名的那几处，`ChampionDetailsPanel.tsx`/`MatchDetailPanel.tsx` 的错误态还是 red-*，现在反而是"两种红"并存 |
| 5 | Error Prevention | 3 | 不变 |
| 6 | Recognition Rather Than Recall | 3 ↑ | 熟练度、评分含义现在都有 tooltip 解释，比上次单纯留白好 |
| 7 | Flexibility and Efficiency | 3 | 不变 |
| 8 | Aesthetic and Minimalist Design | 3 ↑ | side-stripe 和三处死代码清理完，密度虽高但不再有多余装饰 |
| 9 | Error Recovery | 2 ↓ | 两个详情浮层的错误态用纯红，跟主界面已经改成 rose 的刷新失败提示视觉上不是一套语言——半迁移比不迁移多制造了一种新的不一致 |
| 10 | Help and Documentation | 2 ↑ | tooltip 从"零解释"变成"有实质内容"，但组队字母、段位含义仍未覆盖 |
| **Total** | | **27/40** | **可接受，比上轮 25/40 有真实进步，但没有想象中多——一个 P0 是这轮改动自己漏下的尾巴，一个新 P1 是评分改版力度过猛的副作用** |

#### Anti-Patterns Verdict

**不算 AI 味，但"半迁移"本身开始有 AI 辅助重构常见的那种味道。**

**LLM 评审**：五项修复在各自动的文件里都落地干净——评分行是真实功能而不是装饰（宽度按本局最高分归一化，不是固定刻度）、战绩胜负配色的"胜=蓝"问题真的没了不是换皮、死代码清理逐一核实过没有残留。但色彩语义这项只改了上次评审明确点名的那几行，`ChampionDetailsPanel.tsx:71`、`MatchDetailPanel.tsx:62`、以及技能描述解析失败兜底色（`ChampionDetailsPanel.tsx:109` 的 `unresolved` span）还是纯红——同一个"数据不可用"概念，现在在悬浮窗的不同浮层里显示成两种不同色相，这正是"没做完的代币迁移"这种半吊子感的来源。

**确定性扫描**：对 `SelfHistoryOverlay.tsx`、`selfHistoryOverlayUtils.ts`、`components/overlay/` 全目录重新扫了一遍，**0 处命中**（exit code 0）。上一轮点名的 `border-l-4` 已经彻底清除。

**可视化叠加层**：走的是本地源码 + 实机截图路线（空数据态），没有注入浏览器叠加层。截图确认了新增的"评分"行渲染正常，并肉眼验证了段位文字（"未定级"）确实比评分数字小、淡得多——印证了下面的 P1。

#### Overall Impression

五个问题里，三个（评分锚点、胜负配色主干、死代码+文档）是扎实完成的真修复，不是表面掩盖。剩下两个信号说明"改一处、忘一片"和"矫枉过正"是这轮返工里真实存在的风险：色彩语义只改了上次评审精确点名的坐标，没有推广到同一模式在其他文件里的其他实例；评分从"完全没权重"直接跳到"比段位显眼一大截"，PRODUCT.md 说的是两个锚点并列，不是评分压过段位。这两条都不难修，但值得下一轮认真处理，而不是又只改点名的那几行。

#### What's Working

- **评分强度条是真功能，不是好看的装饰**：宽度按本局十人最高分动态归一化（`scoreWidth`），弱势玩家的条子会明显短，不是固定刻度上凑数字。
- **胜负配色的核心 bug 真的没了**：战绩行背景色、结果文字色都验证过是 emerald/rose，不是换了个名字的旧色值。
- **死代码清理可验证地彻底**：`advisorTagClass`、`resultClass`、`playerBadge`、`PlayerView.badge` 全部搜索确认零残留，不是"删了调用点、类型定义还留着"那种半吊子清理。

#### Priority Issues

- **[P0] 色彩语义迁移只做了一半，两个详情浮层还是纯红。** 为什么重要：`ChampionDetailsPanel.tsx:71`、`MatchDetailPanel.tsx:62` 的错误提示框仍是 `border-red-700 bg-red-950 text-red-400`，跟两步之前刚改对的 `SelfHistoryOverlay.tsx:356` 刷新失败提示（现在是 `rose-700/950/400`）用的不是同一种红。用户点开英雄技能遇到网络问题，看到的警示色跟刷新失败时看到的不是同一套视觉语言，没有任何功能上的理由。修复：把这两处以及 `ChampionDetailsPanel.tsx:109` 的 `unresolved` span 兜底色一并并回 rose-*。建议命令：`/impeccable colorize`
- **[P1] 段位在这轮改版里被评分甩开了，PRODUCT.md 说的是两个并列锚点。** 为什么重要：评分行现在是 17px `font-extrabold` + 独立标签 + 强度条 + tooltip；段位（`RankEntry`）还是原来的 `text-[11px] text-white/80`，无加粗、无强调，挤在名字下面半宽的位置。文档写的是"段位与 Scout Score 是锚点"，两者并列，不是评分优先——这轮修复把"评分完全没权重"这个问题矫枉过正成了"评分权重远超段位"。修复：至少把段位文字加粗、放大到 12-13px，让两个锚点视觉上打平。建议命令：`/impeccable layout`
- **[P2] 强度条本身偏细，"几秒扫十张卡"这个场景里余光很难辨认。** `h-1`（4px）配 `bg-white/10` 的轨道，紧贴在 17px 文字下面。既然这条信号存在的目的就是"不用读数字也能比出强弱"，4px 在密集的十卡网格里容易被忽略。建议命令：`/impeccable layout`
- **[P3] 强度条的颜色没有跟着 `scoreToneClass` 走，只有数字在变色。** 一个评分很低、数字显示暗灰色的玩家，条子却还是固定的琥珀渐变，只是短——同一个数值的两种呈现方式互相矛盾。建议命令：`/impeccable polish`

#### Persona Red Flags

**习惯先看段位的老玩家**：段位是 LoL 玩家的肌肉记忆式扫读起点，现在视觉上要先被更显眼的评分吸引，再手动找回段位——两步扫读，不是一眼扫读（P1）。

**点开技能面板遇到网络问题的玩家**：看到的错误提示是纯红而不是悬浮窗其他地方已经统一的玫红，视觉上像是另一套更严重的报警（P0）。

#### Minor Observations

`MatchDetailPanel.tsx` 和战绩行背景色继续用 `bg-[rgba(...)]` 而不是 Tailwind 的 `emerald`/`rose` alpha 类（两行之外的结果文字就在用），是三套标签样式系统这个已知、还没排期的问题在色彩这条线上的又一个实例；熟练度 tooltip 是简单字符串拼接（`${t("profile.mastery")} ${masteryLevel}`），中英文都读得通，但如果这个模式要推广，值得换成带占位符的模板字符串。

#### Questions to Consider

评分和段位要不要干脆合并成一个视觉单元（比如段位徽标下面直接挂评分条），而不是两行分别抢主视觉？半迁移这件事本身，是不是说明需要一条 lint 规则或者把颜色收进 token 表，而不是每轮评审揪出几行就改几行——这次"改完"的色彩语义问题，会不会下一轮又冒出新的漏网实例？
