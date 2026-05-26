/**
 * Extra search keywords per champion ID, beyond the Chinese name and English
 * alias already provided by the LCU. Contains pinyin abbreviations (initials
 * of each Chinese character) and common player-coined nicknames from the CN
 * server community.
 *
 * Format: space-separated tokens. Search performs substring matching against
 * the full string, so "九尾" matches both "九尾" and "九尾狐".
 */
export const CHAMPION_EXTRA_KEYWORDS: Record<number, string> = {
  1: "an 黑暗女孩",               // Annie 安妮
  2: "olf",                        // Olaf 奥拉夫
  3: "jl 石像鬼",                  // Galio 加里奥
  4: "myzl tf 老鸨 牌手",          // Twisted Fate 命运之轮
  5: "zx",                         // Xin Zhao 赵信
  6: "ejt 螃蟹",                   // Urgot 厄加特
  7: "lbl 双子",                   // LeBlanc 勒布朗
  8: "fljml 吸血鬼 血魔",           // Vladimir 弗拉基米尔
  9: "fdtk 稻草人",                // Fiddlesticks 费德提克
  10: "kl 天使",                   // Kayle 凯尔
  11: "yds 瞎子",                  // Master Yi 易大师
  12: "alst 牛头",                 // Alistar 阿利斯塔
  13: "lz 秃头",                   // Ryze 莱兹
  14: "sn 铁男",                   // Sion 赛恩
  15: "xve 飞镖",                  // Sivir 希维尔
  16: "slk 香蕉 奶妈",             // Soraka 索拉卡
  17: "tm 蘑菇 狗",                // Teemo 提莫
  18: "tstn 小炮",                 // Tristana 崔斯特纳
  19: "hw 狼人",                   // Warwick 华威
  20: "nn 雪人 大雪球",            // Nunu & Willump 努努和威朗普
  21: "mf 双枪",                   // Miss Fortune 财富之拳
  22: "ah 弓女",                   // Ashe 艾希
  23: "tst 蛮王",                  // Tryndamere 崔斯特
  24: "jks 武器大师",              // Jax 杰克斯
  25: "mgn 黑天使 铁链",           // Morgana 莫甘娜
  26: "zln 时钟",                  // Zilean 泽利安
  27: "xjd 毒药 毒圈",             // Singed 辛吉德
  28: "yfl 魅惑",                  // Evelynn 伊芙琳
  29: "tq 鼠鼠 老鼠",              // Twitch 图奇
  30: "klss 卡萨",                 // Karthus 卡尔萨斯
  31: "kjs 大虫子",                // Cho'Gath 科加斯
  32: "amm 木乃伊 哭哭",           // Amumu 阿木木
  33: "lms 球球",                  // Rammus 拉莫斯
  34: "ynwy 凤凰 冰鸟",            // Anivia 艾尼维亚
  35: "sk 小丑",                   // Shaco 萨科
  36: "mdys 蒙多",                 // Dr. Mundo 蒙多医生
  37: "sona 琴女",                 // Sona 娑娜
  38: "ksd 空手道",                // Kassadin 卡萨丁
  39: "yrly 刀妹",                 // Irelia 艾瑞利亚
  40: "jn 风女",                   // Janna 迦娜
  41: "plk 船长",                  // Gangplank 普朗克
  42: "kj 飞机",                   // Corki 科基
  43: "klm 业力",                  // Karma 卡尔玛
  44: "tlk 宝石",                  // Taric 塔里克
  45: "wj 小法",                   // Veigar 维迦
  48: "tl 巨魔",                   // Trundle 图勒
  50: "sw 大帅",                   // Swain 斯温
  51: "ktl 警长 胡椒",             // Caitlyn 凯特琳
  53: "blz 机器人 抓钩",           // Blitzcrank 布里茨
  54: "mft 石头人",                // Malphite 墨菲特
  55: "ktln 刀妹",                 // Katarina 卡特琳娜
  56: "my 屠夫",                   // Nocturne 梦魇
  57: "mk 树人",                   // Maokai 茂凯
  58: "lkt 鳄鱼",                  // Renekton 雷克顿
  59: "jwss jw j4 嘉文",           // Jarvan IV 嘉文四世
  60: "yls 蜘蛛",                  // Elise 伊莱斯
  61: "olnn 发条",                 // Orianna 奥丽安娜
  62: "swk 猴子 悟空",             // Wukong 孙悟空
  63: "bld 火男",                  // Brand 布兰德
  64: "lq ls 瞎子 盲僧",           // Lee Sin 李青
  67: "wen 暗影猎手",              // Vayne 薇恩
  68: "lb 炮车",                   // Rumble 兰博
  69: "ksopy 蛇女 蛇姐",           // Cassiopeia 卡西奥佩娅
  72: "skn 天蝎 蝎子",             // Skarner 斯卡纳
  74: "hmdg 海默 炮台",            // Heimerdinger 海默丁格
  75: "nss 狗头",                  // Nasus 纳沙斯
  76: "ndl 豹女",                  // Nidalee 奈达丽
  77: "wdl 熊 熊爷",               // Udyr 乌迪尔
  78: "bb 小锤",                   // Poppy 波比
  79: "gljs 酒桶",                 // Gragas 古拉加斯
  80: "pns 小兵",                  // Pantheon 潘森
  81: "yzrl ez",                   // Ezreal 伊泽瑞尔
  82: "mdks 莫德",                 // Mordekaiser 莫德凯撒
  83: "ylk 掘墓",                  // Yorick 约里克
  84: "akl 阿卡",                  // Akali 阿卡丽
  85: "kn 忍者 电忍",              // Kennen 凯南
  86: "gl 德玛 德玛西亚",           // Garen 盖伦
  89: "lon 太阳 烈日",             // Leona 雷欧娜
  90: "mzh 先知",                  // Malzahar 玛扎哈
  91: "tln 爪子",                  // Talon 泰隆
  92: "rw",                        // Riven 瑞文
  96: "kjm 寇格",                  // Kog'Maw 寇格魔
  98: "sh 影忍",                   // Shen 申
  99: "lks 光辉",                  // Lux 拉克丝
  101: "zls 炮台 石棺",            // Xerath 泽拉斯
  102: "xwn 龙女",                 // Shyvana 希瓦娜
  103: "al 九尾 狐狸",             // Ahri 阿狸
  104: "glfs 男枪",                // Graves 格雷福斯
  105: "fz 鱼人",                  // Fizz 菲兹
  106: "zhxw 雷熊 电熊",           // Volibear 战熊先锋
  107: "lejl 豹子 雷恩",           // Rengar 雷恩加尔
  110: "wls 弓箭男",               // Varus 维鲁斯
  111: "ntls 螃蟹 船锚",           // Nautilus 诺提勒斯
  112: "wkt 机器 进化",            // Viktor 维克托
  113: "szn 猪妹 猪女",            // Sejuani 瑟庄妮
  114: "fon 剑姬",                 // Fiora 菲奥娜
  115: "jgs 炸弹",                 // Ziggs 吉格斯
  117: "ll 仙灵",                  // Lulu 璐璐
  119: "dlw 德莱",                 // Draven 德莱文
  120: "hklm 马哥 马",             // Hecarim 赫卡里姆
  121: "kzk 蚱蜢 跳虫",            // Kha'Zix 卡兹克
  122: "dls 大魔王",               // Darius 达瑞斯
  126: "js 大炮",                  // Jayce 杰斯
  127: "lsz 冰女",                 // Lissandra 丽桑卓
  131: "dnn 月女神 月亮",          // Diana 黛安娜
  133: "qyn 鹰女",                 // Quinn 奎因
  134: "xdl 球女",                 // Syndra 辛德拉
  136: "tyl 龙爸 龙",              // Aurelion Sol 太阳龙
  141: "ky 镰刀",                  // Kayn 凯隐
  142: "zy 泡泡",                  // Zoe 佐伊
  143: "ql 花草",                  // Zyra 齐拉
  145: "ks 蚕宝宝",                // Kai'Sa 卡莎
  147: "rl 歌姬",                  // Seraphine 芮兰
  150: "jn 恐龙 小恐龙",           // Gnar 基纳
  154: "zc 果冻",                  // Zac 扎克
  157: "ys 剑圣 亚索",             // Yasuo 亚索
  161: "wlm 触手 眼球",            // Vel'Koz 韦鲁斯
  163: "tly 石女 滑板",            // Taliyah 塔莉垭
  164: "kml 钢脚 蜘蛛",            // Camille 卡蜜尔
  166: "aks",                      // Akshan 阿克尚
  200: "blws 女皇",                // Bel'Veth 贝尔维斯
  202: "jue 四 钢琴 珏",           // Jhin 珏
  203: "sjly 羊狼 双子",           // Kindred 双界灵鸦
  222: "jks 疯女 金克",            // Jinx 金克斯
  223: "tmkq 鲶鱼 大嘴",          // Tahm Kench 塔姆·肯奇
  234: "vg 吸血鬼 吸血",          // Viego 维艾戈 (note: actual id is 234?)
  235: "xn",                       // Senna 辛纳
  236: "lxa 鬼步",                 // Lucian 卢锡安
  238: "ylzz 影流 男刀",           // Zed 影流之主
  240: "kl 骑士",                  // Kled 克烈
  245: "ak 时光",                  // Ekko 艾克
  246: "qyn 元素 女皇",            // Qiyana 奇亚娜
  254: "wei 警察",                 // Vi 蔚
  266: "ytkx 战争之王 亚托",       // Aatrox 亚托克斯
  267: "nm 鱼奶",                  // Nami 娜美
  268: "azl 沙皇 鸟人",            // Azir 阿兹尔
  350: "ym 猫咪 猫 悠",            // Yuumi 悠米
  360: "sml 沙漠玫瑰",             // Samira 萨米拉
  369: "rntg",                     // Renata Glasc 瑞纳塔·格拉斯
  372: "wyg 丧夫",                 // Viego 维艾戈
  412: "tsr 门神 钩子",            // Thresh 锤石
  420: "yld 章鱼 触手",            // Illaoi 伊莱冬
  421: "lksi 地鼠 地道",           // Rek'Sai 雷克赛
  427: "ayw 树爸",                 // Ivern 艾翁
  429: "klst 鬼女 长矛",           // Kalista 卡莉丝塔
  432: "bd 旅行者 奶爸",           // Bard 巴德
  497: "luo 旅者",                 // Rakan 洛
  498: "xia 霞",                   // Xayah 霞
  516: "on 铁匠 山羊",             // Ornn 奥恩
  517: "sls 铁链男",               // Sylas 塞拉斯
  518: "nk 变色龙",                // Neeko 尼可
  523: "aflos 月枪",               // Aphelios 阿飞利欧斯
  526: "re 钢铁",                  // Rell 蕾尔
  555: "pk 派克 暗杀者",           // Pyke 派克
  711: "vex 悲观 哥特",            // Vex 薇古丝
  777: "yw 剑鬼",                  // Yone 永恩
  875: "st 赛特",                  // Sett 赛特
  876: "lly 莉莉娅 梦",            // Lillia 莉莉娅
  887: "gw 女武神",                // Gwen 格温
  888: "rn 黄金",                  // Renata Glasc — see 369
  895: "nji 尼菈",                 // Nilah 尼菈
  897: "kpw 琼恩",                 // K'Sante 克桑特
  901: "smy 斯莫德",               // Smolder 斯莫德
  902: "mlo 米利欧",               // Milio 米利欧
  910: "hr 赫莉卡",                // Hwei 黑维
  950: "nar 纳尔",                 // Naafiri 纳菲利
};

/**
 * Returns extra search keywords for a champion, or an empty string if none
 * are registered.
 */
export function extraKeywords(championId: number): string {
  return CHAMPION_EXTRA_KEYWORDS[championId] ?? "";
}
