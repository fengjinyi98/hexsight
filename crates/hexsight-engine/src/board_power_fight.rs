// 棋盘战力评分器 + 战斗结果预测器
// 核心职责：
// - BoardPowerScorer：计算棋盘战力（前排EHP + 后排DPS + 爆发 + 羁绊 + 装备 + 控制）
// - FightOutcomeEstimator：预测胜率、剩余棋子、掉血区间、风险等级

use hexsight_core::{ChampionCombatProfile, FightOutcome, RulePack};
use crate::DamageProfile;

/// 棋盘战力评分器
pub struct BoardPowerScorer;

/// 战力分解
#[derive(Debug, Clone)]
pub struct BoardPower {
    pub total_power: f64,
    pub frontline_ehp: f64,
    pub backline_dps: f64,
    pub burst_power: f64,
    pub trait_power: f64,
    pub item_power: f64,
    pub control_power: f64,
    pub alive_count: i32,
}

impl BoardPowerScorer {
    /// 计算棋盘总战力
    pub fn compute(
        profiles: &[ChampionCombatProfile],
        active_trait_count: i32,
        completed_item_count: i32,
        has_combat_augment: bool,
        rule_pack: &RulePack,
    ) -> BoardPower {
        let w = &rule_pack.weights.board_power;
        let alive = profiles.len() as i32;

        // 前排：row 靠前（此处按 role_tags 中是否有 主坦/副坦 判断）
        let frontliners: Vec<&ChampionCombatProfile> = profiles.iter()
            .filter(|p| p.role_tags.iter().any(|r| r == "主坦" || r == "副坦"))
            .collect();
        let backliners: Vec<&ChampionCombatProfile> = profiles.iter()
            .filter(|p| !frontliners.contains(&p))
            .collect();

        let fallback_frontline = frontliners.is_empty();
        let effective_frontline: Vec<&ChampionCombatProfile> = if fallback_frontline {
            profiles.iter().take(2).collect()
        } else {
            frontliners
        };

        // 前排有效生命值
        let frontline_ehp: f64 = effective_frontline.iter()
            .map(|p| p.effective_hp as f64)
            .sum::<f64>()
            / 1000.0;  // 归一化

        // 后排 DPS（普攻 + 技能估算）
        let backline_dps: f64 = backliners.iter()
            .map(|p| {
                let ad_contrib = p.base_ad as f64 * p.base_as;
                let skill_burst = if p.damage_type == "魔法" { ad_contrib * 0.8 } else { ad_contrib * 0.5 };
                (ad_contrib + skill_burst) / 100.0
            })
            .sum::<f64>()
            .max(0.5);

        // 爆发力：快启动 + 高伤害 + 处决标签
        let burst_power: f64 = profiles.iter()
            .map(|p| {
                let mut burst = if p.cast_tempo == "快启动" { 15.0 } else { 5.0 };
                if p.special_tags.contains(&"处决".to_string()) { burst += 10.0; }
                if p.special_tags.contains(&"技能暴击".to_string()) { burst += 5.0; }
                burst
            })
            .sum::<f64>()
            / 100.0;

        // 羁绊战力：每个激活羁绊贡献
        let trait_power = (active_trait_count as f64 * 8.0) / 100.0;

        // 装备战力：每件成装贡献
        let item_power = (completed_item_count as f64 * 10.0) / 100.0;

        // 控制战力
        let control_power: f64 = profiles.iter()
            .filter(|p| p.role_tags.contains(&"控制".to_string()))
            .count() as f64 * 5.0 / 100.0;
        let control_bonus = if has_combat_augment { 5.0 } else { 0.0 };

        let total = frontline_ehp * w.frontline_ehp
            + backline_dps * w.backline_dps
            + burst_power * w.burst_power
            + trait_power * w.trait_power
            + item_power * w.item_power
            + (control_power + control_bonus / 100.0) * w.control_power;

        BoardPower {
            total_power: total,
            frontline_ehp,
            backline_dps,
            burst_power,
            trait_power,
            item_power,
            control_power: control_power + control_bonus / 100.0,
            alive_count: alive,
        }
    }
}

// ============================================================

/// 战斗结果预测器
pub struct FightOutcomeEstimator;

impl FightOutcomeEstimator {
    /// 预测战斗结果
    pub fn predict(
        our_power: &BoardPower,
        enemy_power: &BoardPower,
        our_hp: i32,
        stage: i32,
        damage_profile: &DamageProfile,
    ) -> FightOutcome {
        let power_diff = our_power.total_power - enemy_power.total_power;

        // 胜率：sigmoid 函数映射战力差
        let win_prob = 1.0 / (1.0 + (-power_diff * 2.5).exp());
        let win_prob = win_prob.clamp(0.05, 0.95);

        // 预计剩余棋子：战力差映射
        let survivor_diff = (power_diff * 3.0).round() as i32;
        let _our_survivors = if survivor_diff > 0 {
            (survivor_diff as f64).min(our_power.alive_count as f64)
        } else {
            (-survivor_diff as f64).min(enemy_power.alive_count as f64)
        };

        let expected_enemy_survivors = if power_diff > 0.0 {
            (enemy_power.alive_count as f64 - survivor_diff as f64 * 0.8).max(0.0)
        } else {
            (enemy_power.alive_count as f64 - survivor_diff as f64).max(0.5)
        };

        // 掉血计算
        let expected_damage = if win_prob < 0.5 {
            damage_profile.calculate_damage(stage, expected_enemy_survivors.ceil() as i32) as f64
        } else {
            ((1.0 - win_prob) * damage_profile.calculate_damage(stage, 4) as f64).max(0.0)
        };

        // 掉血区间：赢时包含 0，输时给从小入到大入的范围
        let min_dmg = if win_prob > 0.5 { 0 } else { damage_profile.calculate_damage(stage, 0) };
        let max_dmg = damage_profile.calculate_damage(stage, enemy_power.alive_count);
        let damage_range = [min_dmg, max_dmg];

        // 风险等级
        let risk_level = if expected_damage >= 8.0 {
            "high"
        } else if expected_damage >= 5.0 {
            "medium"
        } else {
            "low"
        };

        // 置信度（战力差越大越确定）
        let confidence = (0.5 + power_diff.abs().min(1.0) * 0.35).clamp(0.45, 0.85);

        let mut reason = Vec::new();
        if win_prob > 0.7 { reason.push("战力占优".into()); }
        else if win_prob < 0.3 { reason.push("战力劣势".into()); }
        else { reason.push("战力接近".into()); }

        if expected_damage >= 8.0 { reason.push("可能被大入".into()); }
        if our_hp < 35 && expected_damage >= 4.0 { reason.push("血量危险需止血".into()); }

        let recommended_action = if risk_level == "high" && our_hp < 35 {
            "立即补战力或止血".to_string()
        } else if risk_level == "high" {
            "考虑补前排或合战力装".to_string()
        } else if risk_level == "medium" {
            "可接受，继续观察".to_string()
        } else {
            "战力充足".to_string()
        };

        FightOutcome {
            win_probability: (win_prob * 100.0).round() / 100.0,
            expected_damage_taken: (expected_damage * 10.0).round() / 10.0,
            damage_range,
            expected_enemy_survivors: (expected_enemy_survivors * 10.0).round() / 10.0,
            risk_level: risk_level.to_string(),
            confidence: (confidence * 100.0).round() / 100.0,
            reason,
            recommended_action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DamageProfile as DPExt;

    fn make_profile(hp: i32, armor: i32, ad: i32, is_tank: bool, is_control: bool) -> ChampionCombatProfile {
        let mut roles = vec!["副C".to_string()];
        if is_tank { roles.push("主坦".to_string()); }
        if is_control { roles.push("控制".to_string()); }
        ChampionCombatProfile {
            hero_id: "test".into(), name: "test".into(), cost: 3,
            role_tags: roles,
            damage_type: "物理".into(), attack_pattern: vec!["单体".into()],
            targeting: "当前目标".into(), cast_tempo: "中启动".into(),
            special_tags: vec![],
            base_hp: hp, base_armor: armor, base_mr: 30,
            base_ad: ad, base_as: 0.7,
            effective_hp: (hp as f64 * (1.0 + armor as f64 / 100.0)) as i32,
        }
    }

    #[test]
    fn board_power_with_tank_and_carry() {
        let profiles = vec![
            make_profile(900, 60, 50, true, false),   // 主坦
            make_profile(600, 30, 85, false, false),  // 主C
            make_profile(550, 25, 70, false, true),   // 控制
        ];
        let pack = RulePack::default();
        let power = BoardPowerScorer::compute(&profiles, 3, 2, false, &pack);
        assert!(power.total_power > 0.0);
        assert!(power.frontline_ehp > 0.0);
        assert!(power.backline_dps > 0.0);
        assert!(power.control_power > 0.0);
    }

    #[test]
    fn fight_prediction_winning() {
        let pack = RulePack::default();
        let dp = DPExt::from_rule_pack(&pack);
        let our = BoardPower {
            total_power: 2.5, frontline_ehp: 1.0, backline_dps: 0.8,
            burst_power: 0.2, trait_power: 0.3, item_power: 0.2,
            control_power: 0.1, alive_count: 5,
        };
        let enemy = BoardPower {
            total_power: 1.0, frontline_ehp: 0.5, backline_dps: 0.3,
            burst_power: 0.1, trait_power: 0.1, item_power: 0.1,
            control_power: 0.0, alive_count: 4,
        };
        let outcome = FightOutcomeEstimator::predict(&our, &enemy, 80, 4, &dp);
        assert!(outcome.win_probability > 0.5);
        assert_eq!(outcome.risk_level, "low");
    }

    #[test]
    fn fight_prediction_losing() {
        let pack = RulePack::default();
        let dp = DPExt::from_rule_pack(&pack);
        let our = BoardPower {
            total_power: 0.5, frontline_ehp: 0.2, backline_dps: 0.2,
            burst_power: 0.0, trait_power: 0.1, item_power: 0.0,
            control_power: 0.0, alive_count: 3,
        };
        let enemy = BoardPower {
            total_power: 2.0, frontline_ehp: 1.0, backline_dps: 0.6,
            burst_power: 0.2, trait_power: 0.2, item_power: 0.2,
            control_power: 0.1, alive_count: 5,
        };
        let outcome = FightOutcomeEstimator::predict(&our, &enemy, 30, 5, &dp);
        assert!(outcome.win_probability < 0.5);
        assert!(outcome.risk_level == "medium" || outcome.risk_level == "high");
    }

    #[test]
    fn damage_range_is_valid() {
        let pack = RulePack::default();
        let dp = DPExt::from_rule_pack(&pack);
        let our = BoardPower {
            total_power: 1.0, frontline_ehp: 0.5, backline_dps: 0.3,
            burst_power: 0.1, trait_power: 0.1, item_power: 0.1,
            control_power: 0.0, alive_count: 4,
        };
        let enemy = BoardPower {
            total_power: 1.0, frontline_ehp: 0.5, backline_dps: 0.3,
            burst_power: 0.1, trait_power: 0.1, item_power: 0.1,
            control_power: 0.0, alive_count: 4,
        };
        let outcome = FightOutcomeEstimator::predict(&our, &enemy, 80, 4, &dp);
        assert!(outcome.damage_range[0] <= outcome.damage_range[1]);
        assert!(outcome.confidence > 0.0);
    }
}
