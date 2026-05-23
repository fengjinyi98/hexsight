// 快照与恢复
// 核心职责：
// - 每回合结束自动生成 JSON 快照
// - 闪退重启后恢复上一帧对局记忆
// - 快照文件仅保留最新1份，自动覆盖

use hexsight_core::{GameState, HexResult};

/// 保存快照到本地文件（JSON 格式）
/// 仅保留最新1份，自动覆盖
pub fn save_snapshot(_state: &GameState, _path: &str) -> HexResult<()> {
    // TODO: serde_json 序列化 → 写入文件
    Ok(())
}

/// 从本地快照恢复对局状态
/// 返回 None 表示无可用快照
pub fn load_snapshot(_path: &str) -> HexResult<Option<GameState>> {
    // TODO: 读取 JSON 文件 → 反序列化
    Ok(None)
}

/// 删除快照文件
pub fn delete_snapshot(_path: &str) -> HexResult<()> {
    // TODO: 删除快照文件
    Ok(())
}
