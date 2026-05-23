// 阵容库加载与匹配
// 核心职责：
// - 从 config/lineups/ 加载 JSON 阵容配置
// - 按装备/海克斯/来牌匹配最优阵容
// - 提供阵容查询接口

use hexsight_core::{HexResult, LineupConfig, RecognizedFrame};

/// 阵容数据库
pub struct LineupDB {
    /// 已加载的所有阵容
    lineups: Vec<LineupConfig>,
}

impl LineupDB {
    /// 创建空阵容库
    pub fn new() -> Self {
        Self { lineups: vec![] }
    }

    /// 从目录加载所有阵容 JSON 文件
    pub fn load_from_dir(&mut self, _dir: &str) -> HexResult<usize> {
        // TODO: 遍历目录 → 读取 .json → 反序列化 LineupConfig
        Ok(self.lineups.len())
    }

    /// 加载单个阵容配置
    pub fn load_config(&mut self, _json: &str) -> HexResult<()> {
        // TODO: serde_json 反序列化 → 加入 lineups
        Ok(())
    }

    /// 根据当前装备和来牌，返回 Top2 推荐阵容
    pub fn recommend(&self, _frame: &RecognizedFrame, _top_n: usize) -> Vec<&LineupConfig> {
        // TODO: 按 装备适配 > 初始来牌 > 版本强度 排序
        self.lineups.iter().take(_top_n).collect()
    }

    /// 按名称查找阵容
    pub fn find_by_name(&self, _name: &str) -> Option<&LineupConfig> {
        self.lineups.iter().find(|l| l.name == _name)
    }

    /// 获取阵容总数
    pub fn count(&self) -> usize {
        self.lineups.len()
    }
}

impl Default for LineupDB {
    fn default() -> Self {
        Self::new()
    }
}
