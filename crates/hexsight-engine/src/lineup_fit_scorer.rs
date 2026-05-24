// 阵容适配评分器 + 过渡战力评分器
// 核心职责：
// - LineupFitScorer：综合评分每套阵容的适配度
// - TransitionStrengthScorer：评估当前过渡战力

use std::path::Path;

use hexsight_core::{
    ChampionCapability, EquipmentData, ItemValueProfile, LineupProfile, LineupScore, PlaystyleTag,
    RulePack, TransitionStrength,
};

use crate::champion_item_fit::{
    ChampionItemFitScorer, ItemReplacementGroupLoader, ItemReplacementGroups, ItemSynthesisIndex,
};
use crate::knowledge_builders::KnowledgeBase;

/// 阵容适配评分器
pub struct LineupFitScorer;

/// 阵容装备评分上下文
/// 核心职责：
/// - 为阵容评分提供棋子画像、装备画像、当前装备状态
/// - 接入 P1 替代组配置与装备合成索引
pub struct LineupItemFitContext<'a> {
    pub champions: &'a [ChampionCapability],
    pub items: &'a [ItemValueProfile],
    pub current_completed_item_ids: &'a [String],
    pub current_component_ids: &'a [String],
    pub replacement_groups: Option<ItemReplacementGroups>,
    pub synthesis_index: Option<ItemSynthesisIndex>,
}

impl<'a> LineupItemFitContext<'a> {
    pub fn from_rule_config(
        knowledge_base: &'a KnowledgeBase,
        current_completed_item_ids: &'a [String],
        current_component_ids: &'a [String],
        equipment: &[EquipmentData],
        config_root: &Path,
        version: &str,
    ) -> hexsight_core::HexResult<Self> {
        let replacement_path = config_root
            .join("rules")
            .join(version)
            .join("item_replacement_groups.json");
        let replacement_groups = if replacement_path.exists() {
            Some(ItemReplacementGroupLoader::load(&replacement_path)?)
        } else {
            None
        };

        Ok(Self {
            champions: &knowledge_base.champions,
            items: &knowledge_base.items,
            current_completed_item_ids,
            current_component_ids,
            replacement_groups,
            synthesis_index: Some(ItemSynthesisIndex::from_equipment(equipment)),
        })
    }
}

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
                None,
            )
        }).collect()
    }

    /// 综合评分：带 P1 主 C 装备上下文
    pub fn score_all_with_item_context(
        profiles: &[LineupProfile],
        current_hero_ids: &[String],
        current_augment_ids: &[String],
        current_gold: i32,
        current_hp: i32,
        current_level: i32,
        round_stage: f64,
        rival_counts: &std::collections::HashMap<String, i32>,
        rule_pack: &RulePack,
        item_context: &LineupItemFitContext<'_>,
    ) -> Vec<LineupScore> {
        profiles.iter().map(|profile| {
            Self::score_one(
                profile,
                current_hero_ids,
                item_context.current_completed_item_ids,
                current_augment_ids,
                current_gold,
                current_hp,
                current_level,
                round_stage,
                rival_counts,
                rule_pack,
                Some(item_context),
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
        item_context: Option<&LineupItemFitContext<'_>>,
    ) -> LineupScore {
        let mut reasons = Vec::new();
        let mut risks = Vec::new();

        // 1. 版本基础分
        let base_score = profile.base_tier;
        reasons.push(format!("版本强度: {}", profile.base_tier));

        // 2. 装备匹配分
        let item_fit = if let Some(item_context) = item_context {
            Self::item_fit_with_context(profile, item_context, &mut reasons, &mut risks)
        } else {
            Self::item_fit(profile, current_equipment_ids, &mut reasons, &mut risks)
        };

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
        if profile.core_equipment_ids.is_empty() { return 30; }
        if equipment.is_empty() { return 30; }
        let match_count = profile.core_equipment_ids.iter()
            .filter(|eid| equipment.contains(eid))
            .count();
        let total = profile.core_equipment_ids.len().max(1);
        let ratio = match_count as f64 / total as f64;
        let missing_count = profile.core_equipment_ids.len().saturating_sub(match_count);
        let core_missing_penalty = (missing_count as i32 * 20).min(60);
        if ratio > 0.5 { _reasons.push(format!("核心装匹配 {}/{}", match_count, total)); }
        if missing_count > 0 {
            risks.push(format!("缺 {} 件核心装", missing_count));
        }
        if ratio < 0.2 && !equipment.is_empty() {
            risks.push("装备方向偏离核心装".into());
        }
        ((ratio * 100.0) as i32 - core_missing_penalty).clamp(0, 100)
    }

    fn item_fit_with_context(
        profile: &LineupProfile,
        context: &LineupItemFitContext<'_>,
        reasons: &mut Vec<String>,
        risks: &mut Vec<String>,
    ) -> i32 {
        if profile.core_equipment_ids.is_empty() {
            return 30;
        }
        if context.current_completed_item_ids.is_empty() && context.current_component_ids.is_empty() {
            return 30;
        }

        let Some(carry_id) = profile.carry_hero_ids.first() else {
            return Self::item_fit(profile, context.current_completed_item_ids, reasons, risks);
        };
        let Some(champion) = context.champions.iter().find(|champion| &champion.hero_id == carry_id) else {
            return Self::item_fit(profile, context.current_completed_item_ids, reasons, risks);
        };

        let (_fits, impact) = ChampionItemFitScorer::score_with_item_context(
            champion,
            context.items,
            &profile.core_equipment_ids,
            context.current_completed_item_ids,
            context.current_component_ids,
            context.replacement_groups.as_ref(),
            context.synthesis_index.as_ref(),
        );

        let total = profile.core_equipment_ids.len().max(1) as i32;
        let direct_count = profile.core_equipment_ids.iter()
            .filter(|item_id| context.current_completed_item_ids.contains(item_id))
            .count() as i32;
        let synth_count = profile.core_equipment_ids.iter()
            .filter(|item_id| !context.current_completed_item_ids.contains(item_id))
            .filter(|item_id| context.synthesis_index.as_ref()
                .map(|index| index.can_synthesize(item_id, context.current_component_ids))
                .unwrap_or(false))
            .count() as i32;
        let alternative_count = (impact.alternatives.len() as i32)
            .min(total.saturating_sub(direct_count + synth_count));

        if direct_count > 0 {
            reasons.push(format!("核心装匹配 {}/{}", direct_count, total));
        }
        if synth_count > 0 {
            reasons.push(format!("当前散件可合成核心装 {} 件", synth_count));
        }
        if !impact.alternatives.is_empty() {
            reasons.push(impact.explanation.clone());
        }
        if impact.core_missing_penalty > 0 {
            risks.push(format!("核心装备缺口惩罚 {}", impact.core_missing_penalty));
        }
        if impact.should_pivot {
            risks.push("核心装缺口过大，建议考虑转向".into());
        }

        let score = ((direct_count * 100 + synth_count * 70 + alternative_count * 35) / total)
            .clamp(0, 100);
        if context.current_component_ids.is_empty() {
            score
        } else {
            score.max(30)
        }
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

    fn make_profile_with_core_items(core_equipment_ids: Vec<String>) -> LineupProfile {
        LineupProfile {
            lineup_id: "lineup_item_gap".into(),
            name: "装备缺口测试".into(),
            base_tier: 100,
            playstyle_tags: vec![PlaystyleTag::Standard, PlaystyleTag::ItemStrict],
            final_hero_ids: vec!["hero1".into()],
            carry_hero_ids: vec!["hero1".into()],
            tank_hero_ids: vec![],
            core_equipment_ids,
            tank_equipment_ids: vec![],
            equipment_order_ids: vec![],
            recommended_hex_ids: vec![],
            replacement_hex_ids: vec![],
            early_hero_ids: vec![],
            mid_hero_ids: vec![],
            trait_targets: HashMap::new(),
            strategy_texts: HashMap::new(),
            mode_specific: serde_json::Value::Null,
            carry_costs: vec![4],
            category: None,
        }
    }

    #[test]
    fn missing_core_items_reduce_lineup_score_and_emit_risk() {
        let profile = make_profile_with_core_items(vec![
            "item_a".into(),
            "item_b".into(),
            "item_c".into(),
        ]);
        let rule_pack = RulePack::default();

        let missing = LineupFitScorer::score_all(
            &[profile.clone()],
            &["hero1".into()],
            &["wrong_item".into()],
            &[],
            30, 80, 7, 3.5,
            &HashMap::new(),
            &rule_pack,
        );
        let matched = LineupFitScorer::score_all(
            &[profile],
            &["hero1".into()],
            &["item_a".into(), "item_b".into(), "item_c".into()],
            &[],
            30, 80, 7, 3.5,
            &HashMap::new(),
            &rule_pack,
        );

        assert!(matched[0].total_score > missing[0].total_score);
        assert_eq!(missing[0].item_fit_score, 0);
        assert!(missing[0].risks.iter().any(|risk| risk.contains("缺 3 件核心装")));
    }

    fn make_context_champion() -> ChampionCapability {
        ChampionCapability {
            hero_id: "hero1".into(),
            name: "主C".into(),
            cost: 4,
            traits: vec![],
            role: hexsight_core::ChampionRole::PrimaryCarry,
            damage_profile: "ap".into(),
            damage_pattern: vec!["burst".into()],
            cast_pattern: "mana_cast".into(),
            scaling_stats: vec!["ability_power".into(), "mana".into()],
            position_role: "backline".into(),
            item_strictness: "high".into(),
            power_spikes: vec![],
            risk_profile: vec![],
            item_preferences: Default::default(),
            confidence: 1.0,
            needs_override: false,
        }
    }

    fn make_context_item(id: &str, name: &str) -> ItemValueProfile {
        ItemValueProfile {
            item_id: id.into(),
            name: name.into(),
            item_type: "成型装备".into(),
            stats: vec![hexsight_core::ItemStat { name: "ap".into(), value: 30.0 }],
            effect_tags: vec!["mana_engine".into()],
            best_for_profiles: vec!["ap_carry_mana".into()],
            bad_for_profiles: vec![],
            replacement_group: String::new(),
            conflict_group: String::new(),
            stage_bias: "mid_late".into(),
            tier: None,
            damage_type_fit: vec!["ap".into()],
        }
    }

    #[test]
    fn p1_item_context_distinguishes_alternative_synthesis_and_completed_core() {
        let profile = make_profile_with_core_items(vec!["core".into()]);
        let champions = vec![make_context_champion()];
        let items = vec![
            make_context_item("core", "青龙刀"),
            make_context_item("alt", "蓝霸符"),
        ];
        let groups = ItemReplacementGroups {
            version: "test".into(),
            groups: std::collections::HashMap::from([(
                "ap_mana".into(),
                crate::champion_item_fit::ReplacementGroup {
                    label: "AP回蓝装".into(),
                    item_ids: vec!["core".into(), "alt".into()],
                    item_names: vec![],
                    items: vec![],
                    replaces_for: vec!["ap_carry_mana".into()],
                    notes: String::new(),
                },
            )]),
        };
        let synthesis = ItemSynthesisIndex::from_routes(vec![(
            "core".to_string(),
            ["rod".to_string(), "tear".to_string()],
        )]);
        let wrong_completed = vec!["wrong".to_string()];
        let no_components: Vec<String> = vec![];
        let synth_components = vec!["rod".to_string(), "tear".to_string()];
        let completed_core = vec!["core".to_string()];

        let alternative_context = LineupItemFitContext {
            champions: &champions,
            items: &items,
            current_completed_item_ids: &wrong_completed,
            current_component_ids: &no_components,
            replacement_groups: Some(groups.clone()),
            synthesis_index: Some(synthesis.clone()),
        };
        let synthesis_context = LineupItemFitContext {
            champions: &champions,
            items: &items,
            current_completed_item_ids: &[],
            current_component_ids: &synth_components,
            replacement_groups: Some(groups.clone()),
            synthesis_index: Some(synthesis.clone()),
        };
        let completed_context = LineupItemFitContext {
            champions: &champions,
            items: &items,
            current_completed_item_ids: &completed_core,
            current_component_ids: &no_components,
            replacement_groups: Some(groups),
            synthesis_index: Some(synthesis),
        };

        let score = |context: &LineupItemFitContext<'_>| {
            LineupFitScorer::score_all_with_item_context(
                &[profile.clone()],
                &["hero1".into()],
                &[],
                30, 80, 7, 3.5,
                &HashMap::new(),
                &RulePack::default(),
                context,
            )
            .remove(0)
        };

        let alternative = score(&alternative_context);
        let synthesizable = score(&synthesis_context);
        let completed = score(&completed_context);

        assert!(synthesizable.item_fit_score > alternative.item_fit_score);
        assert!(completed.item_fit_score > synthesizable.item_fit_score);
        assert!(alternative.reasons.iter().any(|reason| reason.contains("蓝霸符")));
        assert!(synthesizable.reasons.iter().any(|reason| reason.contains("当前散件可合成核心装")));
    }

    #[test]
    fn real_data_p1_item_context_scores_lineup_from_rule_config() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().parent().unwrap().join("config");
        let index = GameDataIndex::load(&root, "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&root, "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let profiles = LineupProfileBuilder::build_all(&cards, &index);
        let heroes: Vec<_> = index.heroes_by_id.values().cloned().collect();
        let equipment: Vec<_> = index.equipment_by_id.values().cloned().collect();
        let hexes: Vec<_> = index.hexes_by_id.values().cloned().collect();
        let kb = crate::KnowledgeBaseBuilder::with_defaults()
            .build_all_with_rule_config(&heroes, &equipment, &hexes, &index.traits, &root, "S18.1")
            .unwrap();
        let profile = profiles.iter()
            .find(|profile| {
                !profile.core_equipment_ids.is_empty()
                    && profile.carry_hero_ids.iter().any(|id| kb.champions.iter().any(|champion| &champion.hero_id == id))
                    && profile.core_equipment_ids.iter().any(|id| {
                        index.equipment(id)
                            .map(|equip| !equip.synthesis1.is_empty() && !equip.synthesis2.is_empty())
                            .unwrap_or(false)
                    })
            })
            .unwrap();
        let core = profile.core_equipment_ids.iter()
            .find_map(|id| {
                index.equipment(id)
                    .filter(|equip| !equip.synthesis1.is_empty() && !equip.synthesis2.is_empty())
                    .map(|equip| (id.clone(), equip.clone()))
            })
            .unwrap();
        let current_components = vec![core.1.synthesis1.clone(), core.1.synthesis2.clone()];
        let current_completed = vec!["wrong_item".to_string()];
        let component_context = LineupItemFitContext::from_rule_config(
            &kb,
            &[],
            &current_components,
            &equipment,
            &root,
            "S18.1",
        ).unwrap();
        let wrong_context = LineupItemFitContext::from_rule_config(
            &kb,
            &current_completed,
            &[],
            &equipment,
            &root,
            "S18.1",
        ).unwrap();

        let component_score = LineupFitScorer::score_all_with_item_context(
            &[profile.clone()],
            &profile.final_hero_ids,
            &[],
            30, 80, 7, 3.5,
            &HashMap::new(),
            &RulePack::default(),
            &component_context,
        ).remove(0);
        let wrong_score = LineupFitScorer::score_all_with_item_context(
            &[profile.clone()],
            &profile.final_hero_ids,
            &[],
            30, 80, 7, 3.5,
            &HashMap::new(),
            &RulePack::default(),
            &wrong_context,
        ).remove(0);

        assert!(component_score.item_fit_score > wrong_score.item_fit_score);
        assert!(component_score.reasons.iter().any(|reason| reason.contains("当前散件可合成核心装")));
    }
}
