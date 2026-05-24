// 海克斯效果解释器
// 核心职责：
// - 实时解析候选海克斯的数值/机制影响（不依赖历史统计）
// - 通过关键词 + 通用机制词典输出标签、即时战力、阵容覆盖度、锁方向风险
// - 支持版本覆写处理文本难解析或机制特殊的海克斯

use serde::{Deserialize, Serialize};

/// 海克斯效果解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentEffect {
    /// 海克斯 ID
    #[serde(rename = "augmentId")]
    pub augment_id: String,
    /// 海克斯名称
    pub name: String,
    /// 效果标签
    pub tags: Vec<String>,
    /// 即时战力 (0-100)
    #[serde(rename = "immediatePower")]
    pub immediate_power: i32,
    /// 阵容覆盖度（支持几套候选阵容）
    #[serde(rename = "lineupCoverage")]
    pub lineup_coverage: i32,
    /// 锁方向风险 (0-100)
    #[serde(rename = "lockRisk")]
    pub lock_risk: i32,
    /// 赌狗开关价值
    #[serde(rename = "rerollValue")]
    pub reroll_value: i32,
    /// 解析原因
    pub reason: Vec<String>,
}

/// 海克斯效果解释器
pub struct AugmentEffectInterpreter;

/// 海克斯机制关键词表
struct TagRules {
    keywords: &'static [&'static str],
    tag: &'static str,
    power: i32,
    lock: i32,
}

impl AugmentEffectInterpreter {
    /// 解析海克斯效果
    pub fn interpret(
        augment_id: &str,
        augment_name: &str,
        augment_desc: &str,
        augment_level: i32,
        lineup_recommended_count: i32, // 多少套阵容推荐此海克斯
    ) -> AugmentEffect {
        let desc_lower = augment_desc.to_lowercase();
        let name_lower = augment_name.to_lowercase();
        let combined = format!("{} {}", name_lower, desc_lower);

        let mut tags = Vec::new();
        let mut reasons = Vec::new();
        let mut immediate_power = 30; // 基础分
        let mut lock_risk = 20;
        let mut reroll_value = 0;

        // 关键词规则表
        let rules: &[TagRules] = &[
            // 通用战力
            TagRules {
                keywords: &["攻击力", "法术强度", "法强", "ad", "ap", "伤害"],
                tag: "generic_combat",
                power: 25,
                lock: 0,
            },
            TagRules {
                keywords: &["攻速", "攻击速度", "暴击"],
                tag: "generic_combat",
                power: 20,
                lock: 0,
            },
            TagRules {
                keywords: &["生命值", "护甲", "魔抗", "双抗", "护盾", "治疗"],
                tag: "generic_combat",
                power: 15,
                lock: 0,
            },
            TagRules {
                keywords: &["攻击力", "攻速", "攻击速度", "物理伤害", "ad"],
                tag: "damage_ad",
                power: 0,
                lock: 0,
            },
            TagRules {
                keywords: &["法术强度", "法强", "魔法伤害", "ap"],
                tag: "damage_ap",
                power: 0,
                lock: 0,
            },
            TagRules {
                keywords: &["重伤", "减疗", "治疗降低"],
                tag: "anti_heal",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["灼烧", "燃烧"],
                tag: "burn",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["护甲击碎", "破甲", "穿甲"],
                tag: "armor_shred",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["魔抗击碎", "破防", "减魔抗"],
                tag: "mr_shred",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["回蓝", "启动", "法力值"],
                tag: "mana_engine",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["技能暴击", "暴击体系"],
                tag: "crit_enable",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["吸血", "续航", "全能吸血"],
                tag: "sustain",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["护盾"],
                tag: "shield_survival",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["光环", "团队增益", "全队"],
                tag: "aura_team_buff",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["唯一", "不可叠加"],
                tag: "unique_trigger",
                power: 0,
                lock: 0,
            },
            // 通用经济
            TagRules {
                keywords: &["金币", "经济", "利息", "免费"],
                tag: "generic_econ",
                power: 5,
                lock: 0,
            },
            TagRules {
                keywords: &["刷新", "商店", "d牌", "免费刷新"],
                tag: "generic_econ",
                power: 10,
                lock: 0,
            },
            TagRules {
                keywords: &["经验", "等级", "人口", "升星"],
                tag: "generic_econ",
                power: 10,
                lock: 0,
            },
            // 装备
            TagRules {
                keywords: &["装备", "散件", "锻造", "重铸", "拆卸", "组件"],
                tag: "generic_item",
                power: 15,
                lock: 0,
            },
            TagRules {
                keywords: &["成装", "神器", "奥恩", "光明"],
                tag: "generic_item",
                power: 25,
                lock: 10,
            },
            // 羁绊强化
            TagRules {
                keywords: &["羁绊", "纹章", "转职", "之心", "之魂", "之冕"],
                tag: "trait_commit",
                power: 20,
                lock: 30,
            },
            // 英雄专属
            TagRules {
                keywords: &["专属", "英雄强化"],
                tag: "hero_commit",
                power: 30,
                lock: 60,
            },
            // 赌狗开关
            TagRules {
                keywords: &["刷新", "复制", "免费"],
                tag: "reroll_enable",
                power: 5,
                lock: 20,
            },
            TagRules {
                keywords: &["低费", "1费", "2费", "三星"],
                tag: "reroll_enable",
                power: 0,
                lock: 30,
            },
            // 节奏/后期
            TagRules {
                keywords: &["连胜", "节奏", "前期", "开局"],
                tag: "tempo",
                power: 20,
                lock: 10,
            },
            TagRules {
                keywords: &["后期", "上限", "高费", "彩色"],
                tag: "late_cap",
                power: 5,
                lock: 10,
            },
            // 模式特殊
            TagRules {
                keywords: &["星神", "神明", "天选", "任务", "传奇"],
                tag: "mode_special",
                power: 10,
                lock: 40,
            },
            // 高风险
            TagRules {
                keywords: &["高风险", "赌", "概率", "随机"],
                tag: "high_risk",
                power: 10,
                lock: 30,
            },
        ];

        for rule in rules {
            for kw in rule.keywords {
                if combined.contains(kw) {
                    if !tags.contains(&rule.tag.to_string()) {
                        tags.push(rule.tag.to_string());
                    }
                    immediate_power += rule.power;
                    lock_risk += rule.lock;
                    if rule.tag == "reroll_enable" {
                        reroll_value += 15;
                    }
                    break;
                }
            }
        }

        // 等级修正
        match augment_level {
            1 => {
                immediate_power -= 5;
                lock_risk -= 10;
            }
            3 => {
                immediate_power += 10;
                lock_risk += 10;
            }
            _ => {}
        }

        // 阵容覆盖度修正
        let lineup_coverage = (lineup_recommended_count as i32).min(5);
        if lineup_coverage >= 3 {
            lock_risk = (lock_risk as f64 * 0.6) as i32;
        }
        if lineup_coverage == 0 {
            lock_risk += 15;
        }

        // 生成原因
        for tag in &tags {
            match tag.as_str() {
                "generic_combat" => reasons.push("提供通用即时战力".into()),
                "generic_econ" => reasons.push("提供经济/运营收益".into()),
                "generic_item" => reasons.push("提供装备/散件弹性".into()),
                "trait_commit" => reasons.push("绑定羁绊方向".into()),
                "hero_commit" => reasons.push("绑定特定英雄".into()),
                "reroll_enable" => reasons.push("低费/赌狗路线加成".into()),
                "tempo" => reasons.push("前中期节奏优势".into()),
                "late_cap" => reasons.push("后期上限提升".into()),
                "high_risk" => reasons.push("高风险高收益".into()),
                "anti_heal" => reasons.push("提供重伤/减疗覆盖".into()),
                "burn" => reasons.push("提供灼烧覆盖".into()),
                "armor_shred" => reasons.push("提供护甲击碎".into()),
                "mr_shred" => reasons.push("提供魔抗击碎".into()),
                "mana_engine" => reasons.push("提供回蓝/启动收益".into()),
                "crit_enable" => reasons.push("提供暴击体系收益".into()),
                "sustain" => reasons.push("提供续航收益".into()),
                "shield_survival" => reasons.push("提供护盾生存收益".into()),
                "aura_team_buff" => reasons.push("提供团队增益覆盖".into()),
                "unique_trigger" => reasons.push("包含唯一触发效果".into()),
                "damage_ad" => reasons.push("偏 AD 输出收益".into()),
                "damage_ap" => reasons.push("偏 AP 输出收益".into()),
                _ => reasons.push(format!("标签: {}", tag)),
            }
        }

        immediate_power = immediate_power.clamp(0, 100);
        lock_risk = lock_risk.clamp(0, 100);
        reroll_value = reroll_value.clamp(0, 100);

        AugmentEffect {
            augment_id: augment_id.to_string(),
            name: augment_name.to_string(),
            tags,
            immediate_power,
            lineup_coverage,
            lock_risk,
            reroll_value,
            reason: reasons,
        }
    }

    /// 批量解析多个候选海克斯
    pub fn interpret_all(
        candidates: &[(String, String, String, i32)], // (id, name, desc, level)
        lineup_recommended_counts: &[(String, i32)],  // (augment_id, count)
    ) -> Vec<AugmentEffect> {
        candidates
            .iter()
            .map(|(id, name, desc, level)| {
                let coverage = lineup_recommended_counts
                    .iter()
                    .find(|(aid, _)| aid == id)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                Self::interpret(id, name, desc, *level, coverage)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_generic_combat() {
        let effect = AugmentEffectInterpreter::interpret(
            "hex_001",
            "战斗强化",
            "获得30攻击力和20法术强度",
            2,
            3,
        );
        assert!(effect.tags.contains(&"generic_combat".to_string()));
        assert!(effect.immediate_power > 30);
        assert!(effect.lock_risk < 40);
    }

    #[test]
    fn interpret_trait_commit() {
        let effect = AugmentEffectInterpreter::interpret(
            "hex_trait",
            "羁绊之魂",
            "获得一个随机纹章和羁绊强化",
            3,
            1,
        );
        assert!(effect.tags.contains(&"trait_commit".to_string()));
        assert!(effect.lock_risk > 30);
    }

    #[test]
    fn interpret_reroll_enabler() {
        let effect = AugmentEffectInterpreter::interpret(
            "hex_reroll",
            "金色刷新",
            "每次D牌获得一次免费刷新，1费和2费英雄概率提升",
            2,
            2,
        );
        assert!(effect.tags.contains(&"reroll_enable".to_string()));
        assert!(effect.reroll_value > 0);
    }

    #[test]
    fn interpret_econ_augment() {
        let effect = AugmentEffectInterpreter::interpret(
            "hex_econ",
            "理财达人",
            "立即获得15金币，每回合额外获得2金币利息",
            2,
            0,
        );
        assert!(effect.tags.contains(&"generic_econ".to_string()));
        assert!(effect.immediate_power < 40); // 经济海克斯即时战力低
    }

    #[test]
    fn batch_interpret() {
        let candidates = vec![
            ("a1".into(), "战力强化".into(), "获得30攻击力".into(), 2),
            ("a2".into(), "经济强化".into(), "获得20金币".into(), 1),
        ];
        let coverage = vec![("a1".into(), 3), ("a2".into(), 1)];
        let effects = AugmentEffectInterpreter::interpret_all(&candidates, &coverage);
        assert_eq!(effects.len(), 2);
        assert!(effects[0].lineup_coverage == 3);
    }
}
