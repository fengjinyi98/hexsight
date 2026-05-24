// 棋子战斗画像构建器
// 核心职责：
// - 从 HeroData 的 skillDesc/skillValueDesc 和属性生成 ChampionCombatProfile
// - 通过关键词匹配输出角色标签、伤害类型、攻击模式、索敌、施法节奏
// - 计算有效生命值供棋盘战力评分使用

use hexsight_core::{ChampionCombatProfile, HeroData};

/// 棋子战斗画像构建器
pub struct ChampionCombatProfileBuilder;

impl ChampionCombatProfileBuilder {
    /// 从英雄数据生成战斗画像
    pub fn build(hero: &HeroData) -> ChampionCombatProfile {
        let desc = format!("{} {}", hero.skillName, hero.skillDesc);
        let desc_lower = desc.to_lowercase();

        // 角色标签
        let role_tags = Self::classify_roles(&desc_lower, hero);

        // 伤害类型
        let damage_type = Self::classify_damage_type(&desc_lower, hero);

        // 攻击模式
        let attack_pattern = Self::classify_attack_pattern(&desc_lower);

        // 索敌方式
        let targeting = Self::classify_targeting(&desc_lower);

        // 施法节奏
        let cast_tempo = Self::classify_cast_tempo(hero);

        // 特殊标签
        let special_tags = Self::classify_special_tags(&desc_lower);

        // 基础属性
        let base_hp = hero.initHP.parse::<i32>().unwrap_or(500);
        let base_armor = hero.armor.parse::<i32>().unwrap_or(30);
        let base_mr = hero.magicResist.parse::<i32>().unwrap_or(30);
        let base_ad = hero.initAttackDamage.parse::<i32>().unwrap_or(50);
        let base_as = hero.attackSpeed.parse::<f64>().unwrap_or(0.7);

        // 有效生命值 = HP * (1 + armor/100)（简化公式）
        let effective_hp = (base_hp as f64 * (1.0 + base_armor as f64 / 100.0)) as i32;

        ChampionCombatProfile {
            hero_id: hero.id.clone(),
            name: hero.name.clone(),
            cost: hero.cost,
            role_tags,
            damage_type,
            attack_pattern,
            targeting,
            cast_tempo,
            special_tags,
            base_hp,
            base_armor,
            base_mr,
            base_ad,
            base_as,
            effective_hp,
        }
    }

    /// 批量构建
    pub fn build_all(heroes: &[&HeroData]) -> Vec<ChampionCombatProfile> {
        heroes.iter().map(|h| Self::build(h)).collect()
    }

    fn classify_roles(desc: &str, hero: &HeroData) -> Vec<String> {
        let mut roles = Vec::new();
        let hp = hero.initHP.parse::<i32>().unwrap_or(500);
        let armor = hero.armor.parse::<i32>().unwrap_or(30);
        let mr = hero.magicResist.parse::<i32>().unwrap_or(30);
        let ad = hero.initAttackDamage.parse::<i32>().unwrap_or(50);

        // 坦克判断：高血量 + 高双抗
        if hp > 800 || armor > 50 || mr > 50 {
            roles.push("主坦".to_string());
        } else if hp > 600 || armor > 40 {
            roles.push("副坦".to_string());
        }

        // C 位判断：高攻击 + 技能关键词
        if ad > 70 || desc.contains("伤害") || desc.contains("攻击") {
            if roles.contains(&"主坦".to_string()) {
                roles.push("副C".to_string());
            } else {
                roles.push("主C".to_string());
            }
        } else if ad > 50 {
            roles.push("副C".to_string());
        }

        // 控制判断
        if desc.contains("眩晕") || desc.contains("击飞") || desc.contains("冰冻")
            || desc.contains("恐惧") || desc.contains("嘲讽") {
            roles.push("控制".to_string());
        }

        // 护盾/治疗判断
        if desc.contains("护盾") || desc.contains("治疗") || desc.contains("回复") {
            roles.push("功能".to_string());
        }

        if roles.is_empty() {
            roles.push("功能".to_string());
        }
        roles
    }

    fn classify_damage_type(desc: &str, hero: &HeroData) -> String {
        let has_ad = desc.contains("攻击力") || desc.contains("物理");
        let has_ap = desc.contains("法术强度") || desc.contains("法强") || desc.contains("魔法");
        let has_true = desc.contains("真实伤害") || desc.contains("最大生命值");

        if has_true { return "真实".into(); }
        if has_ad && has_ap { return "混合".into(); }
        if has_ap { return "魔法".into(); }
        if has_ad { return "物理".into(); }
        // 默认根据技能描述推断
        if hero.skillBriefValue.contains("%") || hero.skillDesc.contains("法术") {
            return "魔法".into();
        }
        "物理".into()
    }

    fn classify_attack_pattern(desc: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        if desc.contains("范围") || desc.contains("周围") || desc.contains("所有敌人") || desc.contains("全场") {
            patterns.push("范围".into());
        }
        if desc.contains("弹射") {
            patterns.push("弹射".into());
        }
        if desc.contains("穿透") {
            patterns.push("穿透".into());
        }
        if desc.contains("召唤") || desc.contains("分身") {
            patterns.push("召唤物".into());
        }
        if patterns.is_empty() {
            patterns.push("单体".into());
        }
        patterns
    }

    fn classify_targeting(desc: &str) -> String {
        if desc.contains("距离最远") { return "最远".into(); }
        if desc.contains("血量最低") { return "最低血量".into(); }
        if desc.contains("敌人最多") || desc.contains("最密集") { return "最密集".into(); }
        if desc.contains("后排") { return "后排".into(); }
        "当前目标".into()
    }

    fn classify_cast_tempo(hero: &HeroData) -> String {
        let init_mp = hero.initMP.parse::<i32>().unwrap_or(0);
        let max_mp = hero.maxMP.parse::<i32>().unwrap_or(100);
        let ratio = init_mp as f64 / max_mp.max(1) as f64;

        if ratio > 0.5 { return "快启动".into(); }
        if ratio > 0.25 { return "中启动".into(); }
        if max_mp <= 50 { return "快启动".into(); }
        if max_mp >= 120 { return "慢启动".into(); }
        "中启动".into()
    }

    fn classify_special_tags(desc: &str) -> Vec<String> {
        let mut tags = Vec::new();
        if desc.contains("处决") || desc.contains("最大生命值") { tags.push("处决".into()); }
        if desc.contains("护甲击碎") || desc.contains("降低护甲") || desc.contains("破甲") { tags.push("破甲".into()); }
        if desc.contains("魔抗击碎") || desc.contains("降低魔抗") { tags.push("魔抗击碎".into()); }
        if desc.contains("重伤") || desc.contains("减疗") || desc.contains("治疗降低") { tags.push("减疗".into()); }
        if desc.contains("护盾") { tags.push("护盾".into()); }
        if desc.contains("治疗") || desc.contains("回复") || desc.contains("吸血") { tags.push("治疗".into()); }
        if desc.contains("暴击") || desc.contains("会心") { tags.push("技能暴击".into()); }
        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hero(name: &str, skill: &str, hp: i32, ad: i32, armor: i32) -> HeroData {
        HeroData {
            id: "test".into(), name: name.into(), price: "3".into(),
            picture: "".into(),
            skillName: "测试技能".into(), skillDesc: skill.into(),
            skillIcon: "".into(), skillBriefValue: "100%".into(), skillValueDesc: "".into(),
            species: "".into(), hero_class: "".into(),
            initHP: hp.to_string(), initAttackDamage: ad.to_string(),
            attackSpeed: "0.7".into(), armor: armor.to_string(),
            magicResist: "30".into(), attackRange: "1".into(),
            initMP: "0".into(), maxMP: "80".into(), criticalStrikeChance: "25".into(),
            cost: 3, base_key: "".into(), star_level: 1,
        }
    }

    #[test]
    fn tank_profile() {
        let hero = make_hero("主坦", "获得护盾持续3秒", 900, 50, 60);
        let profile = ChampionCombatProfileBuilder::build(&hero);
        assert!(profile.role_tags.contains(&"主坦".to_string()));
        assert!(profile.special_tags.contains(&"护盾".to_string()));
    }

    #[test]
    fn carry_profile() {
        let hero = make_hero("主C", "对目标造成大量物理伤害", 600, 85, 30);
        let profile = ChampionCombatProfileBuilder::build(&hero);
        assert!(profile.role_tags.contains(&"主C".to_string()));
        assert_eq!(profile.damage_type, "物理");
    }

    #[test]
    fn control_mage() {
        let hero = make_hero("控制法师", "眩晕周围敌人并造成魔法伤害", 550, 40, 20);
        let profile = ChampionCombatProfileBuilder::build(&hero);
        assert!(profile.role_tags.contains(&"控制".to_string()));
        assert!(profile.attack_pattern.contains(&"范围".to_string()));
        assert_eq!(profile.damage_type, "魔法");
    }
}
