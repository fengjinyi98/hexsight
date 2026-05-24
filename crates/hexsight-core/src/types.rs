use serde::{Deserialize, Serialize};

/// 识别结果 —— 从单帧画面中提取的所有结构化数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecognizedFrame {
    /// 当前金币
    pub gold: u32,
    /// 当前血量
    pub hp: u32,
    /// 人口等级
    pub level: u32,
    /// 经验值 (0-100)
    pub exp: u32,
    /// 当前回合标识 (e.g. "2-1", "4-7")
    pub round: String,
    /// 回合阶段
    pub phase: GamePhase,
    /// 连胜/连败数（正=连胜, 负=连败）
    pub streak: i32,
    /// 自身场上棋子列表
    pub own_heroes: Vec<Hero>,
    /// 装备席 + 棋子穿戴装备列表
    pub own_equipment: Vec<Equipment>,
    /// 已选海克斯列表
    pub hextechs: Vec<Hextech>,
    /// 对手基本信息（侧边栏可见部分）
    pub opponents: Vec<OpponentInfo>,
}

/// 对局阶段
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum GamePhase {
    #[default]
    Unknown,
    /// PVE 野怪回合
    PvE,
    /// PVP 对战回合
    PvP,
    /// 选秀阶段
    Carousel,
    /// 海克斯选择
    HextechSelect,
    /// 结算/等待
    Idle,
}

/// 棋子信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Hero {
    /// 英雄名称 (e.g. "金克丝", "盖伦")
    pub name: String,
    /// 费用 (1-5)
    pub cost: u32,
    /// 星级 (1-3)
    pub star: u32,
    /// 所在位置 (行, 列)
    pub position: (u32, u32),
    /// 穿戴装备列表
    pub items: Vec<Equipment>,
}

/// 装备信息
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Equipment {
    /// 装备名称
    pub name: String,
    /// 装备类型
    pub equip_type: EquipmentType,
    /// 是否已合成（false=散件）
    pub completed: bool,
}

/// 装备类型
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum EquipmentType {
    #[default]
    Unknown,
    /// 攻击类
    Offensive,
    /// 防御类
    Defensive,
    /// 功能类
    Utility,
    /// 转职纹章
    Emblem,
}

/// 海克斯信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Hextech {
    /// 海克斯名称
    pub name: String,
    /// 海克斯等级 (1-3)
    pub tier: u32,
}

/// 对手基本信息（侧边栏）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpponentInfo {
    /// 对手索引 (0-6)
    pub index: u32,
    /// 当前血量
    pub hp: u32,
    /// 人口等级
    pub level: u32,
    /// 羁绊标识列表（从侧边栏图标识别）
    pub traits: Vec<String>,
}

/// 对局全局状态 —— 内存记忆系统的核心数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    /// 当前帧识别结果
    pub current: RecognizedFrame,
    /// 历史帧序列（最近N帧关键数据）
    pub history: Vec<RecognizedFrame>,
    /// 当前推荐阵容
    pub active_lineup: Option<LineupRef>,
    /// 同行数量
    pub rival_count: u32,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 对局开始时间戳
    pub started_at: u64,
    /// 上次更新帧序号
    pub frame_seq: u64,
}

/// 阵容引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupRef {
    /// 阵容名称
    pub name: String,
    /// 强度评级 (T0/T1/T2)
    pub tier: String,
    /// 运营类型
    pub playstyle: Playstyle,
}

/// 运营类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Playstyle {
    /// 标准运营
    Standard,
    /// 赌狗
    Reroll,
    /// 九五
    Fast9,
}

/// 风险等级
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// 决策输出 —— 推送到悬浮窗的最终建议
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Decision {
    /// 推荐阵容名称
    pub lineup_name: String,
    /// 操作建议列表
    pub suggestions: Vec<String>,
    /// 同行数量
    pub rival_count: u32,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 核心装备合成路线
    pub equip_route: Vec<String>,
    /// 过渡推荐
    pub transition: Vec<String>,
    /// LLM 补充建议（V1.5 启用）
    pub llm_advice: Option<String>,
    /// 决策来源
    pub source: DecisionSource,
}

/// 决策来源
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum DecisionSource {
    #[default]
    RuleEngine,
    LLM,
    Hybrid,
}

/// 阵容配置 —— 从 JSON 加载的版本阵容数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupConfig {
    /// 阵容名称
    pub name: String,
    /// 强度评级
    pub tier: String,
    /// 运营类型
    pub playstyle: String,
    /// 核心主C英雄名
    pub main_carry: Vec<String>,
    /// 副C英雄名
    pub sub_carry: Vec<String>,
    /// 主坦英雄名
    pub main_tank: Vec<String>,
    /// 完整羁绊列表
    pub traits: Vec<String>,
    /// 主C装备优先级
    pub carry_items: Vec<String>,
    /// 副C装备优先级
    pub sub_carry_items: Vec<String>,
    /// 主坦装备优先级
    pub tank_items: Vec<String>,
    /// 过渡棋子（按阶段）
    pub transitions: Vec<TransitionRule>,
    /// 适配海克斯
    pub hextechs_best: Vec<String>,
    /// 兼容海克斯
    pub hextechs_ok: Vec<String>,
    /// 排斥海克斯
    pub hextechs_bad: Vec<String>,
    /// 同行策略 (0-1家/2家/≥3家)
    pub rival_strategies: RivalStrategies,
}

/// 阶段过渡规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRule {
    /// 适用阶段 (e.g. "2-1" ~ "3-2")
    pub stage: String,
    /// 打工棋子
    pub units: Vec<String>,
    /// 打工羁绊
    pub traits: Vec<String>,
}

/// 同行应对策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivalStrategies {
    /// 0-1家同行策略
    pub low: String,
    /// 2家同行策略
    pub medium: String,
    /// ≥3家同行策略
    pub high: String,
}

/// ROI 区域配置 —— 固定分辨率下各识别区域坐标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// 游戏窗口分辨率 (宽, 高)
    pub resolution: (u32, u32),
    /// 金币区域
    pub gold_rect: Rect,
    /// 血量条区域
    pub hp_bar: Rect,
    /// 人口/等级区域
    pub level_rect: Rect,
    /// 经验条区域
    pub exp_bar: Rect,
    /// 回合文字区域
    pub round_rect: Rect,
    /// 装备席10个槽位
    pub equip_slots: Vec<Rect>,
    /// 棋盘格子 (4行 x 7列 = 28格)
    pub board_grid: Vec<Rect>,
    /// 商店5个槽位
    pub shop_slots: Vec<Rect>,
    /// 海克斯区域
    pub hextech_rects: Vec<Rect>,
    /// 对手侧边栏 (7个对手)
    pub opponent_rects: Vec<Rect>,
    /// 连胜/连败指示区域
    pub streak_rect: Rect,
}

/// 矩形区域
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
