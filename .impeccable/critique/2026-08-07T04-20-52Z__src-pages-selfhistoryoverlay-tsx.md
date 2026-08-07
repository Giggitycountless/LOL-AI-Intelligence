---
target: 游戏内悬浮窗 SelfHistoryOverlay
total_score: 30
p0_count: 0
p1_count: 1
timestamp: 2026-08-07T04-20-52Z
slug: src-pages-selfhistoryoverlay-tsx
---
#### Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | 不变 |
| 2 | Match System / Real World | 3 | 不变 |
| 3 | User Control and Freedom | 3 | Esc/点击空白关闭正常，但战绩行的键盘操作有缺口拖了后腿 |
| 4 | Consistency and Standards | 3 ↑ | 色彩语义在内容区域验证是彻底完成的，但标题栏那个圆点又开了一个新缺口 |
| 5 | Error Prevention | 3 | 不变 |
| 6 | Recognition Rather Than Recall | 4 ↑ | 段位、评分、胜率、KDA 现在全部常驻可见，tooltip 只是补充不是必需 |
| 7 | Flexibility and Efficiency | 3 | Shift+Tab、刷新都在，但没有键盘路径能打开战绩/技能浮层 |
| 8 | Aesthetic and Minimalist Design | 3 | 不变 |
| 9 | Error Recovery | 3 | 不变 |
| 10 | Help and Documentation | 2 | 不变 |
| **Total** | | **30/40** | **四轮下来 25→25→27→30，这轮没有 P0，是纯粹的干净进展** |

#### Anti-Patterns Verdict

**不算 AI 味。** 独立复核用 grep 核实过：`components/overlay/` 目录里没有 `gradient`/`backdrop-blur`/`animate-pulse` 这类装饰性滥用；`ChampionDetailsPanel.tsx`、`MatchDetailPanel.tsx` 里除了两处明确排除的例外（R 技能徽标、图标 hover 色），已经找不到残留的 `red-*`。

**确定性扫描**：`detect.mjs` 对整个悬浮窗目录重新扫描，**0 处命中**。

**可视化叠加层**：走本地源码 + 实机截图路线。这轮复核额外做了一件事——注入了一段临时 DOM，实测了上一轮修的评分强度条三档颜色（`amber-300`/`zinc-300`/`zinc-500`），确认在真实页面里三档确实清楚可辨，不是只在代码里看着对。

#### Overall Impression

前三轮改的东西这轮逐条验证过，站得住：色彩语义是真的改完了，不是又漏了几行；评分强度条三档在浏览器里真的能看清楚。这轮新发现的问题不再是"改一半"或"矫枉过正"这种返工味，而是三处之前四轮都没扫到的角落——一处是真实的键盘可访问性 bug（不是风格问题），一处是标题栏一个装饰性圆点意外撞上了刚刚才理顺的语义色系统，还有一处是发现 `PRODUCT.md` 自己前后不一致（写着"蓝=胜/红=负"，隔了两行又写"emerald=我方/rose=敌方"）。这说明色彩语义那条线已经收敛得差不多了，接下来值得往功能性缺口（键盘操作）和文档准确性上转移注意力。

#### What's Working

- **评分强度条的三档颜色是真解决了，不是纸面上解决了**：独立复核实测截图确认 `bg-amber-300`/`bg-zinc-300`/`bg-zinc-500` 三档在深色卡片背景上一眼可辨，包括最容易失效的"弱势玩家"那一档。
- **色彩语义收尾是彻底的**：不是又改了评审点名的那几行，是整个 `components/overlay/` 目录都 grep 验证过，只剩两处明确该保留原色的例外。
- **空数据态的克制延续得很好**：`未选择`/`未定级`/`战绩不可用` 这套降级展示走的是中性 zinc 色，不是报警红，符合"数据缺失是常态"这条原则——四轮改动都没有破坏这一点。

#### Priority Issues

- **[P1] 战绩行挂了键盘操作的空壳。** `PlayerTrack.tsx` 的 `MatchRow`（约 376-386 行）设置了 `role="button"` 和 `tabIndex={0}`，视觉上也有 `cursor-pointer` 和 hover 高亮，看起来完全可以用键盘操作——但没有任何 `onKeyDown` 处理，Tab 键能聚焦上去，Enter/Space 按下去没有任何反应。这是一个真实存在、可复现的功能缺口，不是风格建议。修复：加 `onKeyDown` 处理 Enter/Space 调用跟 `onClick` 一样的 `onSelect`，或者干脆换成原生 `<button>` 元素省掉这些手动补的 ARIA 属性。建议命令：`/impeccable harden`
- **[P2] 标题栏那个圆点，撞上了刚理顺的语义色系统。** `SelfHistoryOverlay.tsx:341` 的 `bg-rose-700` 圆点是静态装饰，不随任何状态变化——但 `Dashboard.tsx` 里同样的视觉写法（`h-2.5 w-2.5 rounded-full`）是真的状态指示灯（emerald/sky/amber = 就绪/部分/等待）。悬浮窗内容区两行之后就是货真价实的敌我语义圆点（emerald 我方/rose 敌方），标题栏这个圆点用同样的圆点视觉语言却啥也不代表，容易被当成"出问题了"的信号。修复：要么让它真的接上状态（比如数据新鲜度：绿=刚刷新、琥珀=刷新中、玫红=刷新失败），要么换成不是圆点/不是 rose 的纯装饰形式。建议命令：`/impeccable clarify`
- **[P3] 窗口标题在对局进行中依然叫"英雄选择情报"。** `overlay.windowTitle`（i18n.ts）翻成"英雄选择情报"，但这个窗口在 InProgress 阶段（金币/事件/对局时钟都在显示）照样叫这个名字，跟它此刻实际在做的事不符。优先级不高，标题栏不是主扫读区。建议命令：`/impeccable clarify`
- **[P3] `PRODUCT.md` 自己前后矛盾。** 文档"品牌个性"那段写的是"蓝=胜/红=负"，两行之后的设计原则又写"emerald=我方/rose=敌方"——代码已经全面走后者，是文档没跟上自己的另一段话，不是代码问题。建议命令：`/impeccable document`

#### Persona Red Flags

**反复 Alt+Tab 看一眼的排位玩家**：每次打开悬浮窗，第一眼捕捉到的颜色是那个没有意义的玫红圆点，比真正的敌我圆点先进入视野半拍，制造一次不必要的"是不是出事了"的短暂疑惑。

**用键盘/辅助工具操作的玩家**：Tab 到一条战绩行，按 Enter 期待打开对局详情——所有视觉提示都在暗示这会发生——结果什么都没有，只能重新抓鼠标。

#### Minor Observations

评分行依然享有整行的结构性空间（标签+17px数字+6px条），段位还是挤在名字下面的双列窄条里——字号差距这轮补上了，但版面分配的差距还在，不紧急，值得留意。`scoreToneClass` 的中间档文字色（`text-white/80`）跟 `scoreBarClass` 的中间档条色（`bg-zinc-300`）是两种字面上不同的浅灰，视觉上读起来是同一档，纯粹是命名层面的小瑕疵。

#### Questions to Consider

标题栏那个圆点要不要干脆接上真实状态，复用 Dashboard 页面用户已经学会的那套红黄绿语义，而不是白白借用一个没有意义的颜色？战绩行要不要直接换成原生 `<button>`，一次性把这类"看起来能操作但键盘按不动"的缺口在悬浮窗其他地方也筛一遍？
