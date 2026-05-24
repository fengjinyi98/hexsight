// 海克斯候选排序器
// 核心职责：
// - 将海克斯说明书解析为即时收益、阵容覆盖和锁方向风险
// - 结合轮次、开局强弱和类型权重生成排序分
// - 为刷新决策器提供推荐分与兜底分

use hexsight_core::LineupProfile;

use crate::augment_effect_interpreter::AugmentEffectInterpreter;
use crate::augment_reroll_scorer::{
    AugmentSituationContext, AugmentStage, AugmentTypeWeights, RankedAugmentOption,
};

/// 海克斯候选排序器
pub struct AugmentOptionRanker;

impl AugmentOptionRanker {
    pub fn rank_options(
        candidates: &[(String, String, String, i32)],
        profiles: &[LineupProfile],
        stage: AugmentStage,
        opening_is_strong: bool,
        opening_is_weak: bool,
    ) -> Vec<RankedAugmentOption> {
        Self::rank_options_with_weights(
            candidates,
            profiles,
            stage,
            opening_is_strong,
            opening_is_weak,
            &AugmentTypeWeights::default(),
        )
    }

    pub fn rank_options_with_weights(
        candidates: &[(String, String, String, i32)],
        profiles: &[LineupProfile],
        stage: AugmentStage,
        opening_is_strong: bool,
        opening_is_weak: bool,
        weights: &AugmentTypeWeights,
    ) -> Vec<RankedAugmentOption> {
        let context = AugmentSituationContext {
            current_hp: if opening_is_weak { 40 } else { 80 },
            current_gold: if opening_is_strong { 60 } else { 20 },
            item_gap_level: 0,
            core_hero_count: 0,
            locked_lineup_id: None,
            existing_effect_tags: vec![],
            required_effect_tags: vec![],
            preferred_damage_profile: None,
        };
        Self::rank_options_with_weights_and_context(candidates, profiles, stage, weights, &context)
    }

    pub fn rank_options_with_context(
        candidates: &[(String, String, String, i32)],
        profiles: &[LineupProfile],
        stage: AugmentStage,
        context: &AugmentSituationContext,
    ) -> Vec<RankedAugmentOption> {
        Self::rank_options_with_weights_and_context(
            candidates,
            profiles,
            stage,
            &AugmentTypeWeights::default(),
            context,
        )
    }

    pub fn rank_options_with_weights_and_context(
        candidates: &[(String, String, String, i32)],
        profiles: &[LineupProfile],
        stage: AugmentStage,
        weights: &AugmentTypeWeights,
        context: &AugmentSituationContext,
    ) -> Vec<RankedAugmentOption> {
        let coverage_counts: Vec<(String, i32)> = candidates
            .iter()
            .map(|(augment_id, _, _, _)| {
                (
                    augment_id.clone(),
                    profiles
                        .iter()
                        .filter(|profile| {
                            profile.recommended_hex_ids.contains(augment_id)
                                || profile.replacement_hex_ids.contains(augment_id)
                        })
                        .count() as i32,
                )
            })
            .collect();

        let effects = AugmentEffectInterpreter::interpret_all(candidates, &coverage_counts);
        let mut ranked: Vec<_> = effects
            .into_iter()
            .map(|effect| {
                let supported_lineup_ids: Vec<String> = profiles
                    .iter()
                    .filter(|profile| {
                        profile.recommended_hex_ids.contains(&effect.augment_id)
                            || profile.replacement_hex_ids.contains(&effect.augment_id)
                    })
                    .map(|profile| profile.lineup_id.clone())
                    .collect();

                let coverage = supported_lineup_ids.len() as i32;
                let lineup_coverage =
                    ((coverage as f64 / profiles.len().max(1) as f64) * 100.0).round() as i32;
                let type_bonus: i32 = effect
                    .tags
                    .iter()
                    .map(|tag| weights.weights.get(tag).copied().unwrap_or(0))
                    .sum();
                let stage_bias = Self::stage_bias(stage, &effect.tags);
                let (situation_bias, situation_reasons) =
                    Self::situation_bias(&effect.tags, &effect.augment_id, profiles, context);
                let current_value =
                    (effect.immediate_power + type_bonus + stage_bias + situation_bias)
                        .clamp(0, 100);
                let total_score = ((current_value * 35
                    + lineup_coverage * 35
                    + (100 - effect.lock_risk) * 20
                    + effect.reroll_value * 10)
                    / 100)
                    .clamp(0, 100);
                let fallback_score =
                    ((lineup_coverage * 45 + (100 - effect.lock_risk) * 35 + current_value * 20)
                        / 100)
                        .clamp(0, 100);

                let mut reason = effect.reason;
                if coverage > 0 {
                    reason.push(format!("覆盖 {} 套候选阵容", coverage));
                }
                if effect.lock_risk >= 60 {
                    reason.push("锁方向风险高".into());
                }
                reason.extend(situation_reasons);

                RankedAugmentOption {
                    augment_id: effect.augment_id,
                    augment_name: effect.name,
                    total_score,
                    current_value,
                    lineup_coverage,
                    lock_risk: effect.lock_risk,
                    fallback_score,
                    tags: effect.tags,
                    supported_lineup_ids,
                    reason,
                }
            })
            .collect();

        ranked.sort_by(|a, b| {
            b.total_score
                .cmp(&a.total_score)
                .then_with(|| b.fallback_score.cmp(&a.fallback_score))
                .then_with(|| a.lock_risk.cmp(&b.lock_risk))
        });
        ranked
    }

    fn stage_bias(stage: AugmentStage, tags: &[String]) -> i32 {
        match stage {
            AugmentStage::First => {
                if tags.contains(&"generic_item".into()) || tags.contains(&"generic_econ".into()) {
                    8
                } else if tags.contains(&"hero_commit".into())
                    || tags.contains(&"trait_commit".into())
                {
                    -12
                } else {
                    0
                }
            }
            AugmentStage::Second | AugmentStage::Third => {
                if tags.contains(&"generic_combat".into()) || tags.contains(&"trait_commit".into())
                {
                    6
                } else {
                    0
                }
            }
        }
    }

    fn situation_bias(
        tags: &[String],
        augment_id: &str,
        profiles: &[LineupProfile],
        context: &AugmentSituationContext,
    ) -> (i32, Vec<String>) {
        let mut bias = 0;
        let mut reasons = Vec::new();

        for tag in Self::conflict_tags() {
            let has_tag = tags.iter().any(|candidate_tag| candidate_tag == tag);
            if !has_tag {
                continue;
            }

            let already_covered = context
                .existing_effect_tags
                .iter()
                .any(|existing_tag| existing_tag == tag);
            let currently_required = context
                .required_effect_tags
                .iter()
                .any(|required_tag| required_tag == tag);

            if currently_required {
                bias += 12;
                reasons.push(format!("当前局势需要 {} 覆盖", Self::tag_label(tag)));
            } else if already_covered {
                bias -= 22;
                reasons.push(format!(
                    "已有 {} 覆盖，重复效果收益下降",
                    Self::tag_label(tag)
                ));
            }
        }

        if let Some(preferred) = &context.preferred_damage_profile {
            let preferred_tag = format!("damage_{}", preferred);
            let has_preferred = tags.iter().any(|tag| tag == &preferred_tag);
            let has_mismatch = match preferred.as_str() {
                "ad" => tags.iter().any(|tag| tag == "damage_ap"),
                "ap" => tags.iter().any(|tag| tag == "damage_ad"),
                _ => false,
            };

            if has_preferred {
                bias += 8;
                reasons.push(format!(
                    "贴合当前 {} 输出结构",
                    Self::tag_label(&preferred_tag)
                ));
            } else if has_mismatch {
                bias -= 18;
                reasons.push("当前局势不匹配，输出类型收益下降".into());
            }
        }

        if context.current_hp > 0 && context.current_hp <= 45 {
            if tags.contains(&"generic_combat".into()) {
                bias += 10;
                reasons.push("血量低，优先即时战力".into());
            }
            if tags.contains(&"generic_item".into()) {
                bias += 8;
                reasons.push("血量低，装备补强可止血".into());
            }
        }

        if context.current_gold >= 60
            && (tags.contains(&"generic_econ".into()) || tags.contains(&"late_cap".into()))
        {
            bias += 28;
            reasons.push("经济很好，经济/高上限收益提高".into());
        }

        if context.item_gap_level > 0 && tags.contains(&"generic_item".into()) {
            bias += 8 + context.item_gap_level.clamp(0, 4) * 3;
            reasons.push("装备较差，优先装备补强".into());
        }

        if context.core_hero_count >= 2 && tags.contains(&"reroll_enable".into()) {
            bias += 10;
            reasons.push("核心牌较多，刷新/赌狗开关价值提高".into());
        }

        if let Some(locked_id) = &context.locked_lineup_id {
            if let Some(profile) = profiles
                .iter()
                .find(|profile| &profile.lineup_id == locked_id)
            {
                if profile
                    .recommended_hex_ids
                    .iter()
                    .any(|id| id == augment_id)
                {
                    bias += 38;
                    reasons.push("已锁阵容，推荐海克斯优先".into());
                } else if profile
                    .replacement_hex_ids
                    .iter()
                    .any(|id| id == augment_id)
                {
                    bias += 18;
                    reasons.push("已锁阵容，备选海克斯可接受".into());
                }
            }
        }

        (bias, reasons)
    }

    fn conflict_tags() -> &'static [&'static str] {
        &[
            "anti_heal",
            "burn",
            "armor_shred",
            "mr_shred",
            "mana_engine",
            "crit_enable",
            "sustain",
            "shield_survival",
            "aura_team_buff",
            "unique_trigger",
        ]
    }

    fn tag_label(tag: &str) -> &'static str {
        match tag {
            "anti_heal" => "重伤/减疗",
            "burn" => "灼烧",
            "armor_shred" => "护甲击碎",
            "mr_shred" => "魔抗击碎",
            "mana_engine" => "回蓝/启动",
            "crit_enable" => "暴击体系",
            "sustain" => "续航",
            "shield_survival" => "护盾生存",
            "aura_team_buff" => "团队增益",
            "unique_trigger" => "唯一触发",
            "damage_ad" => "AD",
            "damage_ap" => "AP",
            _ => "机制",
        }
    }
}
