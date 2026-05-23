// 阵容适配评分器 + 过渡战力评分器
// 核心职责：
// - LineupFitScorer：综合评分每套阵容的适配度
// - TransitionStrengthScorer：评估当前过渡战力

use hexsight_core::{LineupProfile, LineupScore, PlaystyleTag, RulePack, TransitionStrength};

/// 阵容适配评分器
pub struct LineupFitScorer;

impl LineupFitScorer {
    /// 综合评分：根据当前局势给每套阵容打分（使用规则包权重）
    pub fn score_all(
        profiles: &[LineupProfile],
        current_hero_ids: &[String],
        current_equipment_ids: &[String],
        current_augment_ids: &[String],
        current_gold: i32,
        current_hp: i32,
        current_level: i32,
        round_stage: f64,
        rival_counts: &std::collections::HashMap<String, i32>,
        rule_pack: &RulePack,
    ) -> Vec<LineupScore> {
        profiles.iter().map(|profile| {
            Self::score_one(
                profile,
                current_hero_ids,
                current_equipment_ids,
                current_augment_ids,
                current_gold,
                current_hp,
                current_level,
                round_stage,
                rival_counts,
                rule_pack,
            )
        }).collect()
    }

    fn score_one(
        profile: &LineupProfile,
        current_hero_ids: &[String],
        current_equipment_ids: &[String],
        current_augment_ids: &[String],
        current_gold: i32,
        current_hp: i32,
        current_level: i32,
        round_stage: f64,
        rival_counts: &std::collections::HashMap<String, i32>,
        rule_pack: &RulePack,
    ) -> LineupScore {
        let mut reasons = Vec::new();
        let mut risks = Vec::new();

        // 1. 版本基础分
        let base_score = profile.base_tier;
        reasons.push(format!("版本强度: {}", profile.base_tier));

        // 2. 装备匹配分
        let item_fit = Self::item_fit(profile, current_equipment_ids, &mut reasons, &mut risks);

        // 3. 棋子命中分
        let champion_hit = Self::champion_hit(profile, current_hero_ids, &mut reasons);

        // 4. 海克斯适配分
        let augment_fit = Self::augment_fit(profile, current_augment_ids, &mut reasons);

        // 5. 羁绊成型分
        let trait_fit = Self::trait_fit(profile, current_hero_ids, &mut reasons);

        // 6. 阶段适配分
        let stage_fit = Self::stage_fit(profile, round_stage, current_level, &mut reasons);

        // 7. 经济适配分
        let economy_fit = Self::economy_fit(profile, current_gold, current_level, &mut reasons);

        // 8. 血量安全分
        let health_safety = Self::health_safety(profile, current_hp, &mut reasons, &mut risks);

        // 9. 玩法开关分
        let playstyle_switch = Self::playstyle_switch(profile, current_augment_ids, &mut reasons);

        // 10. 同行风险扣分
        let rival_penalty = rival_counts.get(&profile.lineup_id)
            .copied().unwrap_or(0) * 10;
        if rival_penalty > 0 {
            risks.push(format!("同行 {} 家", rival_penalty / 10));
        }

        // 11. 成型难度扣分
        let difficulty_penalty = Self::difficulty_penalty(profile, round_stage);

        // 加权总分（使用规则包权重）
        let w = &rule_pack.weights.lineup_fit;
        let total = (base_score as f64 * w.base_score
            + item_fit as f64 * w.item_fit
            + champion_hit as f64 * w.champion_hit
            + augment_fit as f64 * w.augment_fit
            + trait_fit as f64 * w.trait_fit
            + stage_fit as f64 * w.stage_fit
            + economy_fit as f64 * w.economy_fit
            + health_safety as f64 * w.health_safety
            + playstyle_switch as f64 * w.playstyle_switch
            - rival_penalty as f64 * 0.03
            - difficulty_penalty as f64 * 0.02) as i32;

        let total = total.clamp(0, 100);

        LineupScore {
            lineup_id: profile.lineup_id.clone(),
            name: profile.name.clone(),
            total_score: total,
            base_score,
            item_fit_score: item_fit,
            champion_hit_score: champion_hit,
            augment_fit_score: augment_fit,
            trait_fit_score: trait_fit,
            stage_fit_score: stage_fit,
            economy_fit_score: economy_fit,
            health_safety_score: health_safety,
            playstyle_switch_score: playstyle_switch,
            rival_penalty,
            difficulty_penalty,
            reasons,
            risks,
            requires_augment: profile.playstyle_tags.contains(&PlaystyleTag::RequiresAugment),
        }
    }

    fn item_fit(profile: &LineupProfile, equipment: &[String], _reasons: &mut Vec<String>, risks: &mut Vec<String>) -> i32 {
        if equipment.is_empty() { return 30; }
        let match_count = profile.core_equipment_ids.iter()
            .filter(|eid| equipment.contains(eid))
            .count();
        let total = profile.core_equipment_ids.len().max(1);
        let ratio = match_count as f64 / total as f64;
        if ratio > 0.5 { _reasons.push(format!("核心装匹配 {}/{}", match_count, total)); }
        if ratio < 0.2 && !equipment.is_empty() {
            risks.push("装备方向偏离核心装".into());
        }
        (ratio * 100.0) as i32
    }

    fn champion_hit(profile: &LineupProfile, heroes: &[String], _reasons: &mut Vec<String>) -> i32 {
        let final_hits = profile.final_hero_ids.iter()
            .filter(|id| heroes.contains(id))
            .count();
        let early_hits = profile.early_hero_ids.iter()
            .filter(|id| heroes.contains(id))
            .count();
        let total_hits = final_hits + early_hits;
        if total_hits > 0 { _reasons.push(format!("英雄命中 {} 个", total_hits)); }
        (total_hits as f64 / 3.0 * 100.0).min(100.0) as i32
    }

    fn augment_fit(profile: &LineupProfile, augments: &[String], reasons: &mut Vec<String>) -> i32 {
        if augments.is_empty() { return 50; }
        let rec_hits = profile.recommended_hex_ids.iter()
            .filter(|id| augments.contains(id))
            .count();
        if rec_hits > 0 { reasons.push(format!("海克斯命中推荐 {}", rec_hits)); return 90; }
        let rep_hits = profile.replacement_hex_ids.iter()
            .filter(|id| augments.contains(id))
            .count();
        if rep_hits > 0 { reasons.push(format!("海克斯命中备选 {}", rep_hits)); return 70; }
        40
    }

    fn trait_fit(profile: &LineupProfile, heroes: &[String], _reasons: &mut Vec<String>) -> i32 {
        if heroes.is_empty() || profile.trait_targets.is_empty() { return 30; }
        // 简化：至少需要一个英雄命中才加分
        let has_any_hero = profile.final_hero_ids.iter().any(|id| heroes.contains(id));
        if has_any_hero { 60 } else { 30 }
    }

    fn stage_fit(profile: &LineupProfile, stage: f64, level: i32, _reasons: &mut Vec<String>) -> i32 {
        let is_reroll1 = profile.playstyle_tags.contains(&PlaystyleTag::Reroll1Cost);
        let is_fast8 = profile.playstyle_tags.contains(&PlaystyleTag::Fast8);

        if is_reroll1 && stage <= 3.5 { return 85; }
        if is_reroll1 && stage > 4.0 { return 20; }
        if is_fast8 && stage >= 3.5 && level >= 7 { return 80; }
        if is_fast8 && stage < 3.0 { return 60; }
        70
    }

    fn economy_fit(profile: &LineupProfile, gold: i32, _level: i32, _reasons: &mut Vec<String>) -> i32 {
        let is_fast9 = profile.playstyle_tags.contains(&PlaystyleTag::Fast9);
        if is_fast9 && gold >= 40 { return 85; }
        if is_fast9 && gold < 20 { return 30; }
        if gold >= 30 { 75 }
        else if gold >= 10 { 50 }
        else { 30 }
    }

    fn health_safety(_profile: &LineupProfile, hp: i32, reasons: &mut Vec<String>, risks: &mut Vec<String>) -> i32 {
        if hp < 30 { risks.push("血量危险".into()); return 20; }
        if hp < 50 { risks.push("血量偏低".into()); return 50; }
        if hp >= 80 { reasons.push("血量安全".into()); return 90; }
        70
    }

    fn playstyle_switch(profile: &LineupProfile, augments: &[String], reasons: &mut Vec<String>) -> i32 {
        let has_switch = profile.playstyle_tags.contains(&PlaystyleTag::AugmentEnabled);
        if !has_switch { return 50; }
        let has_required_aug = profile.recommended_hex_ids.iter()
            .any(|id| augments.contains(id));
        if has_required_aug { reasons.push("关键海克斯已激活".into()); 95 }
        else { 30 }
    }

    fn difficulty_penalty(profile: &LineupProfile, stage: f64) -> i32 {
        let mut penalty = 0;
        if profile.playstyle_tags.contains(&PlaystyleTag::Reroll3Cost) { penalty += 5; }
        if profile.playstyle_tags.contains(&PlaystyleTag::Fast9) { penalty += 10; }
        if profile.playstyle_tags.contains(&PlaystyleTag::ItemStrict) { penalty += 5; }
        if profile.carry_costs.iter().any(|&c| c >= 5) && stage > 3.0 { penalty += 8; }
        penalty
    }
}

// ============================================================

/// 过渡战力评分器
pub struct TransitionStrengthScorer;

impl TransitionStrengthScorer {
    pub fn assess(
        _current_hero_count: i32,
        two_star_count: i32,
        has_frontline: bool,
        has_backline: bool,
        active_trait_count: i32,
        completed_item_count: i32,
        has_combat_augment: bool,
        rule_pack: &RulePack,
    ) -> TransitionStrength {
        let two_star_score = (two_star_count as f64 / 3.0 * 100.0).min(100.0) as i32;
        let frontline_score = if has_frontline { 60 } else { 20 };
        let backline_score = if has_backline { 60 } else { 20 };
        let trait_score = (active_trait_count as f64 / 4.0 * 100.0).min(100.0) as i32;
        let item_score = (completed_item_count as f64 / 3.0 * 100.0).min(100.0) as i32;
        let augment_score = if has_combat_augment { 70 } else { 40 };

        let total = (two_star_score as f64 * rule_pack.weights.transition.two_star
            + frontline_score as f64 * rule_pack.weights.transition.frontline
            + backline_score as f64 * rule_pack.weights.transition.backline
            + trait_score as f64 * rule_pack.weights.transition.transition_trait
            + item_score as f64 * rule_pack.weights.transition.transition_item
            + augment_score as f64 * rule_pack.weights.transition.transition_augment) as i32;

        let mut weaknesses = Vec::new();
        if two_star_count < 2 { weaknesses.push("二星英雄不足".into()); }
        if !has_frontline { weaknesses.push("缺少前排坦克".into()); }
        if completed_item_count < 1 { weaknesses.push("无成装".into()); }

        TransitionStrength {
            total_score: total,
            two_star_score,
            frontline_score,
            backline_score,
            trait_score,
            item_score,
            augment_score,
            lobby_rank_estimate: None,
            is_strong_enough: total >= 60,
            weaknesses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use crate::{GameDataIndex, LineupAdapter, LineupLoader, LineupProfileBuilder};

    fn load_profiles(mode: &str) -> Vec<LineupProfile> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().join("config");
        let index = GameDataIndex::load(&root, mode).unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, mode, "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, mode);
        LineupProfileBuilder::build_all(&cards, &index)
    }

    #[test]
    fn score_all_lineups() {
        let profiles = load_profiles("17");
        let rule_pack = RulePack::default();
        let results = LineupFitScorer::score_all(
            &profiles,
            &[],
            &[],
            &[],
            20, 100, 6, 3.5,
            &HashMap::new(),
            &rule_pack,
        );
        assert!(!results.is_empty());
        // 所有分数在 0-100 范围
        assert!(results.iter().all(|r| r.total_score <= 100));
        // 有分数区分度
        let max = results.iter().map(|r| r.total_score).max().unwrap();
        let min = results.iter().map(|r| r.total_score).min().unwrap();
        assert!(max > min, "所有阵容分数相同，无区分度");
    }

    #[test]
    fn transition_strength_weak() {
        let rule_pack = RulePack::default();
        let strength = TransitionStrengthScorer::assess(3, 0, false, true, 1, 0, false, &rule_pack);
        assert!(!strength.is_strong_enough);
        assert!(!strength.weaknesses.is_empty());
    }

    #[test]
    fn transition_strength_strong() {
        let rule_pack = RulePack::default();
        let strength = TransitionStrengthScorer::assess(5, 3, true, true, 3, 2, true, &rule_pack);
        assert!(strength.is_strong_enough);
        assert!(strength.total_score >= 60);
    }
}
