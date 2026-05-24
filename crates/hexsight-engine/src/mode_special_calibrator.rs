// 模式特殊规划器 + 预测校准器
// 核心职责：
// - ModeSpecialPlanner：星神奖励/任务解锁/天选机制的模式特殊规则
// - DamagePredictionCalibrator：记录预测vs实际结果，输出误差分桶

use hexsight_core::LineupProfile;

/// 模式特殊规则输出
#[derive(Debug, Clone)]
pub struct ModeSpecialPlan {
    pub mode: String,
    /// 星神奖励推荐
    pub god_reward_picks: Vec<GodRewardPick>,
    /// 任务优先级
    pub task_priorities: Vec<TaskPriority>,
    /// 天选建议
    pub chosen_advice: Option<ChosenAdvice>,
    /// 特殊规则文本
    pub extra_rules: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GodRewardPick {
    pub stage: i32,
    pub recommended_god_id: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct TaskPriority {
    pub task_id: String,
    pub hero_name: String,
    pub priority: i32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ChosenAdvice {
    pub target_trait: String,
    pub backup_heroes: Vec<String>,
    pub lock_early: bool,
}

/// 模式特殊规划器
pub struct ModeSpecialPlanner;

impl ModeSpecialPlanner {
    /// 根据模式和阵容档案输出特殊规则
    pub fn plan(mode: &str, profiles: &[LineupProfile]) -> ModeSpecialPlan {
        let mut god_reward_picks = Vec::new();
        let mut task_priorities = Vec::new();
        let mut chosen_advice = None;
        let mut extra_rules = Vec::new();

        match mode {
            "17" => {
                // 星神模式：分析 god_rewards
                for profile in profiles.iter().take(3) {
                    if let Some(god_list) = profile
                        .mode_specific
                        .get("god_rewards")
                        .and_then(|v| v.as_array())
                    {
                        for god in god_list {
                            if let (Some(stage), Some(god_id)) = (
                                god.get("stage").and_then(|v| v.as_i64()),
                                god.get("god_id").and_then(|v| v.as_str()),
                            ) {
                                let wish_count = god
                                    .get("wish_ids")
                                    .and_then(|v| v.as_array())
                                    .map(|a| a.len())
                                    .unwrap_or(0);
                                god_reward_picks.push(GodRewardPick {
                                    stage: stage as i32,
                                    recommended_god_id: god_id.to_string(),
                                    reason: format!(
                                        "{} 阶段 {} 神明奖励（{} 个选项），适配阵容 {}",
                                        stage, god_id, wish_count, profile.name
                                    ),
                                });
                            }
                        }
                    }
                }
                extra_rules
                    .push("星神模式：优先选择经济/战力神祇，2阶段偏经济，3阶段后偏战力".into());
            }
            "16" => {
                // 任务模式：分析 unlock_tasks
                for profile in profiles.iter().take(3) {
                    if let Some(tasks) = profile
                        .mode_specific
                        .get("unlock_tasks")
                        .and_then(|v| v.as_array())
                    {
                        for task in tasks {
                            let task_id =
                                task.get("task_id").and_then(|v| v.as_str()).unwrap_or("");
                            let hero_id =
                                task.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
                            task_priorities.push(TaskPriority {
                                task_id: task_id.to_string(),
                                hero_name: hero_id.to_string(),
                                priority: 5,
                                reason: format!("阵容 {} 的任务解锁", profile.name),
                            });
                        }
                    }
                }
                task_priorities.sort_by(|a, b| b.priority.cmp(&a.priority));
                extra_rules
                    .push("任务模式：优先完成主C/主坦的解锁任务，3-2前至少完成一个关键任务".into());
            }
            "4" => {
                // 天选模式
                for profile in profiles.iter().take(1) {
                    if let Some(chosen) = profile.mode_specific.get("chosen_contact") {
                        let trait_id = chosen.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let chosen_type = chosen.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        chosen_advice = Some(ChosenAdvice {
                            target_trait: format!("{} ({})", trait_id, chosen_type),
                            backup_heroes: vec![],
                            lock_early: profile.base_tier >= 80,
                        });
                    }
                }
                extra_rules.push(
                    "天选模式：优先拿阵容核心羁绊天选，若4-1前未拿到核心天选考虑转阵容".into(),
                );
            }
            _ => {}
        }

        ModeSpecialPlan {
            mode: mode.to_string(),
            god_reward_picks,
            task_priorities,
            chosen_advice,
            extra_rules,
        }
    }
}

// ============================================================

/// 预测校准条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalibrationEntry {
    pub timestamp: String,
    pub stage: i32,
    pub predicted_win_prob: f64,
    pub predicted_damage: f64,
    pub actual_damage: i32,
    pub actual_win: bool,
    pub actual_enemy_survivors: i32,
    pub error_damage: i32,
    pub error_survivors: i32,
    pub tags: Vec<String>,
}

/// 误差分桶
#[derive(Debug, Clone)]
pub struct ErrorBuckets {
    pub damage_mae: f64,
    pub survivor_mae: f64,
    pub win_direction_accuracy: f64,
    pub total_samples: usize,
}

/// 预测校准器
pub struct DamagePredictionCalibrator;

impl DamagePredictionCalibrator {
    /// 记录一次预测结果
    pub fn record(
        stage: i32,
        predicted_win_prob: f64,
        predicted_damage: f64,
        predicted_enemy_survivors: f64,
        actual_damage: i32,
        actual_win: bool,
        actual_enemy_survivors: i32,
    ) -> CalibrationEntry {
        let mut tags = Vec::new();
        let error = predicted_damage as i32 - actual_damage;
        if error.abs() > 4 {
            tags.push("大偏差".into());
        }
        if error > 0 {
            tags.push("高估伤害".into());
        } else if error < 0 {
            tags.push("低估伤害".into());
        }

        CalibrationEntry {
            timestamp: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            ),
            stage,
            predicted_win_prob,
            predicted_damage,
            actual_damage,
            actual_win,
            actual_enemy_survivors,
            error_damage: error,
            error_survivors: predicted_enemy_survivors.ceil() as i32 - actual_enemy_survivors,
            tags,
        }
    }

    /// 从历史记录计算误差分桶
    pub fn compute_buckets(entries: &[CalibrationEntry]) -> ErrorBuckets {
        if entries.is_empty() {
            return ErrorBuckets {
                damage_mae: 0.0,
                survivor_mae: 0.0,
                win_direction_accuracy: 0.0,
                total_samples: 0,
            };
        }
        let n = entries.len();
        let damage_mae = entries
            .iter()
            .map(|e| e.error_damage.abs() as f64)
            .sum::<f64>()
            / n as f64;
        let survivor_mae = entries
            .iter()
            .map(|e| e.error_survivors.abs() as f64)
            .sum::<f64>()
            / n as f64;
        let correct_direction = entries
            .iter()
            .filter(|e| (e.predicted_win_prob >= 0.5) == e.actual_win)
            .count();
        let win_direction_accuracy = correct_direction as f64 / n as f64;

        ErrorBuckets {
            damage_mae,
            survivor_mae,
            win_direction_accuracy,
            total_samples: n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode17_god_rewards_planned() {
        let profile = LineupProfile {
            lineup_id: "test".into(),
            name: "测试".into(),
            base_tier: 80,
            playstyle_tags: vec![],
            final_hero_ids: vec![],
            carry_hero_ids: vec![],
            tank_hero_ids: vec![],
            core_equipment_ids: vec![],
            tank_equipment_ids: vec![],
            equipment_order_ids: vec![],
            recommended_hex_ids: vec![],
            replacement_hex_ids: vec![],
            early_hero_ids: vec![],
            mid_hero_ids: vec![],
            trait_targets: Default::default(),
            strategy_texts: Default::default(),
            mode_specific: serde_json::json!({
                "god_rewards": [{"stage": 2, "god_id": "2", "wish_ids": ["a", "b"]}]
            }),
            carry_costs: vec![4],
            category: None,
        };
        let plan = ModeSpecialPlanner::plan("17", &[profile]);
        assert!(!plan.god_reward_picks.is_empty());
        assert_eq!(plan.god_reward_picks[0].stage, 2);
    }

    #[test]
    fn calibration_records_error() {
        let entry = DamagePredictionCalibrator::record(4, 0.65, 3.5, 1.5, 6, false, 3);
        assert_eq!(entry.error_damage, -3); // 预测 3.5 伤害，实际 6
        assert!(entry.tags.contains(&"低估伤害".to_string()));
    }

    #[test]
    fn calibration_buckets_computed() {
        let entries = vec![
            DamagePredictionCalibrator::record(3, 0.70, 1.0, 1.0, 1, true, 1),
            DamagePredictionCalibrator::record(4, 0.40, 5.0, 3.0, 7, false, 3),
            DamagePredictionCalibrator::record(4, 0.60, 2.0, 1.5, 2, true, 1),
        ];
        let buckets = DamagePredictionCalibrator::compute_buckets(&entries);
        assert_eq!(buckets.total_samples, 3);
        assert!(buckets.win_direction_accuracy >= 0.0);
    }
}
