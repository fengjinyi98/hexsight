// 版本评分修正器
// 核心职责：
// - 将版本公告 buff/nerf 转成阵容评分修正
// - 按棋子、装备、羁绊、海克斯和系统改动映射到对应评分项
// - 输出玩家可理解的版本影响解释与风险提示

use hexsight_core::{LineupProfile, LineupScore, PatchEntry};

use crate::patch_knowledge::PatchKnowledge;

/// 版本评分修正器
pub struct PatchModifier;

impl PatchModifier {
    /// 应用版本修正到单个阵容评分
    pub fn apply_to_score(
        score: &LineupScore,
        profile: &LineupProfile,
        knowledge: &PatchKnowledge,
    ) -> LineupScore {
        let mut adjusted = score.clone();
        let mut total_delta = 0;

        for entry in knowledge
            .entries
            .iter()
            .filter(|entry| Self::affects_lineup(entry, profile))
        {
            let delta = entry.impact_score;
            total_delta += delta;
            Self::apply_component_delta(&mut adjusted, entry, delta);
            Self::append_explanation(&mut adjusted, entry, delta);
        }

        adjusted.total_score = (adjusted.total_score + total_delta).clamp(0, 100);
        adjusted
    }

    /// 批量应用版本修正
    pub fn apply_to_scores(
        scores: &[LineupScore],
        profiles: &[LineupProfile],
        knowledge: &PatchKnowledge,
    ) -> Vec<LineupScore> {
        scores
            .iter()
            .map(|score| {
                profiles
                    .iter()
                    .find(|profile| profile.lineup_id == score.lineup_id)
                    .map(|profile| Self::apply_to_score(score, profile, knowledge))
                    .unwrap_or_else(|| score.clone())
            })
            .collect()
    }

    fn affects_lineup(entry: &PatchEntry, profile: &LineupProfile) -> bool {
        if !entry.affected_lineups.is_empty() {
            return entry
                .affected_lineups
                .iter()
                .any(|lineup_id| lineup_id == &profile.lineup_id);
        }

        match entry.target_type.as_str() {
            "champion" => Self::contains_id(
                &entry.target_id,
                [
                    &profile.final_hero_ids,
                    &profile.carry_hero_ids,
                    &profile.tank_hero_ids,
                    &profile.early_hero_ids,
                    &profile.mid_hero_ids,
                ],
            ),
            "item" => Self::contains_id(
                &entry.target_id,
                [
                    &profile.core_equipment_ids,
                    &profile.tank_equipment_ids,
                    &profile.equipment_order_ids,
                ],
            ),
            "trait" => profile.trait_targets.contains_key(&entry.target_id),
            "augment" => Self::contains_id(
                &entry.target_id,
                [&profile.recommended_hex_ids, &profile.replacement_hex_ids],
            ),
            "lineup" => entry.target_id == profile.lineup_id,
            "system" => true,
            _ => false,
        }
    }

    fn contains_id<'a, const N: usize>(target_id: &str, groups: [&'a Vec<String>; N]) -> bool {
        groups
            .iter()
            .any(|group| group.iter().any(|id| id == target_id))
    }

    fn apply_component_delta(score: &mut LineupScore, entry: &PatchEntry, delta: i32) {
        match entry.target_type.as_str() {
            "champion" | "lineup" | "system" => {
                score.base_score = (score.base_score + delta).clamp(0, 100);
            }
            "item" => {
                score.item_fit_score = (score.item_fit_score + delta).clamp(0, 100);
            }
            "trait" => {
                score.trait_fit_score = (score.trait_fit_score + delta).clamp(0, 100);
            }
            "augment" => {
                score.augment_fit_score = (score.augment_fit_score + delta).clamp(0, 100);
            }
            _ => {}
        }
    }

    fn append_explanation(score: &mut LineupScore, entry: &PatchEntry, delta: i32) {
        let signed = if delta >= 0 {
            format!("+{}", delta)
        } else {
            delta.to_string()
        };
        let reason = if entry.reason.is_empty() {
            "版本公告修正".to_string()
        } else {
            entry.reason.clone()
        };
        let line = format!(
            "{} {} {}：{}",
            entry.target_name, entry.change_type, signed, reason
        );

        if delta >= 0 {
            score.reasons.push(line);
        } else {
            score.risks.push(line);
        }
    }
}
