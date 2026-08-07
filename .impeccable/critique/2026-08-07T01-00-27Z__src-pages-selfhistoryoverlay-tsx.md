---
target: 游戏内悬浮窗 SelfHistoryOverlay
total_score: 25
p0_count: 1
p1_count: 2
timestamp: 2026-08-07T01-00-27Z
slug: src-pages-selfhistoryoverlay-tsx
---
#### Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | 刷新转圈、8s 超时兜底、失败提示自动消失（2.5s），状态反馈到位 |
| 2 | Match System / Real World | 3 | 段位文案正确；"评分"（Scout Score）全程没有任何解释来源的公式或含义 |
| 3 | User Control and Freedom | 3 | Esc / 点击空白关闭浮层，退出路径清楚 |
| 4 | Consistency and Standards | 2 | PRODUCT.md 明确写"rose 永远是敌方"，代码里敌方却全线用 red-400/500/700；胜局用蓝色而非"emerald=正向"的既定规则 |
| 5 | Error Prevention | 3 | 8s 加载超时避免无限转圈 |
| 6 | Recognition Rather Than Recall | 2 | M 徽章、评分、组队字母的含义全靠鼠标 hover 的 title 提示——2 秒扫读场景里根本用不上 |
| 7 | Flexibility and Efficiency | 3 | Shift+Tab 切换、一键刷新 |
| 8 | Aesthetic and Minimalist Design | 2 | 三套互不统一的徽章/标签样式系统（内联 hex、硬编码 arbitrary-value、Tailwind 暗色类）叠在一张卡片上 |
| 9 | Error Recovery | 3 | 不同失败态有不同文案（战绩不可用/身份不可用/详情不可用），不是笼统一句 |
| 10 | Help and Documentation | 1 | 评分公式、熟练度门槛、组队字母含义，产品内零解释，完全依赖用户已经懂 League Akari 的约定 |
| **Total** | | **25/40** | **可接受（Acceptable）—— 距离"好"还差几处具体修复，不是推倒重来** |

#### Anti-Patterns Verdict

**先说结论：不算 AI 味，但代码里有"三只手做的"痕迹。**

**LLM 评审**：整体不是通用生成感——代码里反复注明是在照抄真实存在的参考物（"League Akari's PlayerInfoCardHeader/Stats/MatchHistory"），信息密度（5×2 网格、每卡迷你战绩条、双段位并列）也太具体、太懂这个领域，不像是套模板套出来的。但三处"AI 味"症状还是在的：
1. 同一张卡片上，三套标签/徽标用了三套完全不同的样式机制——组队角标是内联 `style=` hex（`PREMADE_GROUP_STYLES`），连胜连败是硬编码 Tailwind arbitrary-value（`bg-[#18571c]` / `bg-[#893b3b]`），AI 顾问标签又是另一套 Tailwind 暗色类（`advisorTagClassDark`）。视觉上"同一类东西"，实现上三套逻辑。
2. 光主题时代留下的死代码：`advisorTagClass`（`selfHistoryOverlayUtils.ts:271-280`，`bg-emerald-100 text-emerald-800`）和 `resultClass`（同文件 458-467，`bg-emerald-50`）现在没有任何地方 import——正好印证 `docs/adr/overlay-stays-light.md` 那份文档已经过期，代码走了深色但没人回去清理浅色分支。
3. 无 LCU 数据时的空态，实测截图看到的就是 10 张一模一样的灰卡片网格——正好撞上 `PRODUCT.md` 自己明令禁止的"同尺寸卡片网格无限重复"，虽然这是空数据下的副作用而不是主动设计。

**确定性扫描**：`detect.mjs` 命中 1 处 —— `ChampionDetailsPanel.tsx:137`，`border-l-4` 侧边条纹（点开英雄头像后的技能卡片，Q/W/E/R 各用一条彩色左边框区分）。这正是 Impeccable 明令禁止的 side-stripe 模式。级别不高（只在次级浮层里，不在主扫读界面），但确实是个该改的硬伤。跟上一份快照（2026-07-12）比对，同一处、同一模式，行号从 124 挪到 137——问题本身没被处理，只是代码往下挪了几行。

**未见可视化叠加层**：本轮走的是本地源码 + 实机截图路线，没有注入浏览器内叠加层，所以没有 [Human] 标签页可看；上面两条就是全部证据。

#### Overall Impression

这不是一个"看起来随便糊的"悬浮窗——密度、双段位、按局内最高分归一化的评分条这些设计决定都挺讲究，PRODUCT.md 里写的设计原则也基本对得上。但最大的问题是：**文档里点名的"主锚点"，代码里恰恰是权重最低的那个**。评分格（Scout Score）被文档定义为跟段位并列的核心扫读锚点，实现里却是三格数据里唯一不染色、不突出的一个；连给它配的强度条（`scorePct`）都算出来了却根本没渲染。再加上三套标签系统各自为政、色彩语义跟自己文档对不上，整体读起来像是好几个人分几次做的，没有一次通盘对齐。

#### What's Working

- **团队标题行是真正的设计判断，不是套模板**：不给标题行套卡片框，让内容自己决定高度，两块 5 人板才能不强制滚动地堆叠——这个密度取舍是照着"几秒扫读"这个真实场景做的。
- **战绩缺失被当一等状态处理**：`notRequested`/`missingIdentity`/`unavailable` 分别给了不同文案（`recentStatsStatusMessage`），没有笼统塞一句"无数据"，跟 PRODUCT.md 里"数据缺失是常态"的原则对得上。
- **评分条按本局十人最高分归一化（而非固定量表）**：这是一个不显眼但正确的设计决定——只是目前完全没接到界面上（见 P0）。

#### Priority Issues

- **[P0] 评分（Scout Score）——文档点名的主锚点，实现里权重最低**
  **为什么重要**：PRODUCT.md 明确说"段位与 Scout Score 是锚点，其余为次级信息"，但 `PlayerTrack.tsx:157-162` 里评分格固定 `text-white/80`，不像胜率/KDA 那样有 `winRateToneClass`/`kdaToneClass` 染色；专门为它算好的强度条 `scorePct`（`selfHistoryOverlayUtils.ts:45-46,113-114`）压根没在 `PlayerTrack.tsx` 里渲染出来。本该最先被看到的数字，视觉上跟另外两格没有任何差别。
  **修复**：把 `scorePct` 接到评分格下面/背后做成强度条，配一套独立色阶，让它在视觉上明显压过胜率和 KDA。
  **建议命令**：`/impeccable layout` 或 `/impeccable colorize`

- **[P1] 语义色跟自己写的文档对不上**
  **为什么重要**：PRODUCT.md 写"emerald 永远是我方/胜/正向，rose 永远是敌方/负/警示，不在别处挪用"——但敌方队伍、敌方文字全线用的是 `red-400/500/700`（`SelfHistoryOverlay.tsx:474,485,570,603`），只有标题栏那个跟敌我毫无关系的小圆点用了 `rose-700`；胜局战绩行用的是蓝色底（`PlayerTrack.tsx:32`）而不是 emerald。一个已经在主界面习惯了 emerald=我方/rose=敌方的用户，到悬浮窗这里看到的是另一套色相。
  **修复**：要么把悬浮窗这套 League Akari 派生的例外正式写进 PRODUCT.md（承认它是有意为之的独立视觉语言），要么把颜色并回文档定义的 emerald/rose。两者选一个，不要让文档和代码继续各说各话。
  **建议命令**：`/impeccable document`（先把例外写清楚）或 `/impeccable colorize`（真要统一颜色）

- **[P1] 关键含义全靠 hover tooltip，2 秒扫读场景里用不上**
  **为什么重要**：M 徽章等级门槛、评分含义、组队字母，全部只有原生 `title=` 提示能看到。目标用户是"对局中瞥一眼就要关掉"的场景，没有停留时间去 hover。
  **修复**：把关键含义变成可见的短文字/图例，tooltip 只留给次要细节。
  **建议命令**：`/impeccable clarify`

- **[P2] 三套死代码 + 一份过期 ADR，容易被将来的人"修复"回错误方向**
  **为什么重要**：`advisorTagClass`、`resultClass`（浅色版函数）、`playerBadge`（连带它的 `Math.max(1, wins)` 0 胜显示成 1 的老毛病）都已经没有任何地方引用了，但还留在 `selfHistoryOverlayUtils.ts` 里；`docs/adr/overlay-stays-light.md` 仍然写着"悬浮窗保持浅色"，跟实际的深色实现直接矛盾。上一份评审快照（2026-07-12）里点名的"0 胜显示成 1"问题，现在的真实情况是：那个徽章根本没被渲染了，bug 还在代码里，只是不再对用户可见——这是"删掉未接线的死代码"没做到位，不是真正修复。
  **修复**：删掉三个未使用的函数；重写或标注废弃那份 ADR，避免有人对着旧文档把悬浮窗"修复"回浅色。
  **建议命令**：`/impeccable harden`

- **[P2] `border-l-4` 侧边条纹（ChampionDetailsPanel 技能卡片）**
  **为什么重要**：确定性扫描命中的唯一一条，是 Impeccable 明令禁止的 side-stripe 模式（Q/W/E/R 技能卡片左边一条彩色粗边框）。级别不算高——只在点开头像后的次级浮层里，不在主扫读界面——但这是个一眼就能认出来的"AI 味"标志。
  **修复**：换成整框描边、背景色块，或者槽位字母角标，不要用左边框做强调。
  **建议命令**：`/impeccable quieter`

#### Persona Red Flags

**Alex（没耐心的老玩家，中路对线时瞥一眼，预算 2 秒）**：本该第一眼看到的评分（Scout Score）没有视觉权重（见 P0），意味着他要真的读数字而不是识别颜色/形状——这恰好增加了这个界面存在的唯一理由（快速判断威胁）所需要的认知负担。M 徽章、组队字母含义都锁在 hover 提示里，Alex 不会为了看懂一个徽章去悬停鼠标。

**Sam（依赖辅助功能，需要高对比度）**：`text-white/50`（顾问摘要）、`text-zinc-500/600`（空态胜率/KDA 占位）几档文字，直接低于 PRODUCT.md 自己写的"悬浮窗覆盖在高饱和游戏画面上，正文对比度要求高于常规桌面应用"这条硬性要求；含义锁在 `title=` hover 提示里的内容，键盘/屏幕阅读器用户根本拿不到。

#### Minor Observations

- 顶部标题栏的 `bg-rose-700` 圆点跟敌我语义毫无关系，纯粹是一个没解释的品牌点缀，容易让人误以为它代表敌方。
- `SummaryBar` 的 `+`/`-` 前缀渲染出了一个多余空格（实机截图里是"+ 0/0"），破坏了别处都对得很整齐的 tabular-nums 排版。
- 空数据状态下，评分格保持亮白色，胜率/KDA 却变暗成 zinc-500——同一行三格降级方式不一致。
- `Icons.tsx` 里的 SVG 都是干净的自制图标，没有引入图标库/图标字体，克制得住。
- 英雄头像点击热区 42×42，对局中快速点击这个尺寸够用。

#### Questions to Consider

- 如果把评分格删掉、只留那条早就算好但没用上的强度条，卡片会不会扫得更快——四个数字挤在一起，是不是本来就多了一个？
- 要不要干脆让界面自己把"本局评分最高的敌方"高亮出来，而不是把"谁是威胁"这个推理过程留给用户在 2 秒内自己做？
- 组队角标、连胜连败、AI 顾问标签三套样式系统合并成一套统一的 chip 组件/色板，会不会一次性解决"看起来像几个人分头做的"这种感觉？
