// 队伍效果覆盖率检查
// 核心职责：
// - 汇总队伍中所有效果来源：装备、海克斯、羁绊、英雄技能
// - 判断每个冲突组是否已被覆盖
// - 输出队伍覆盖率报告，指导是否还需要特定效果
// - 支持对手环境修正（对手回复多 → 重伤价值提高等）

use std::collections::{HashMap, HashSet};

use hexsight_core::{
    AugmentEffectProfile, ChampionCapability, CoverageType, EffectSource, ItemConflictGroup,
    ItemConflictGroups, ItemValueProfile, TraitEffectProfile,
};

/// 队伍效果覆盖率检查器
pub struct TeamEffectCoverage {
    conflict_groups: HashMap<String, ItemConflictGroup>,
}

impl TeamEffectCoverage {
    pub fn new(groups_config: &ItemConflictGroups) -> Self {
        Self {
            conflict_groups: groups_config.groups.clone(),
        }
    }

    /// 收集队伍中所有效果来源
    pub fn collect_all_sources(
        &self,
        equipped_item_ids: &[String],
        item_profiles: &[&ItemValueProfile],
        active_augments: &[&AugmentEffectProfile],
        active_traits: &[&TraitEffectProfile],
        champions: &[&ChampionCapability],
    ) -> Vec<EffectSource> {
        let mut sources: Vec<EffectSource> = Vec::new();

        // 1. 装备效果
        for item_id in equipped_item_ids {
            for profile in item_profiles {
                if &profile.item_id == item_id {
                    for tag in &profile.effect_tags {
                        let Some(group_id) = self.map_tag_to_group(tag) else {
                            continue;
                        };
                        sources.push(EffectSource {
                            source_type: "item".into(),
                            source_id: item_id.clone(),
                            source_name: profile.name.clone(),
                            group_id,
                            coverage: self.infer_coverage_from_item(profile),
                            weight: 100,
                        });
                    }
                    break;
                }
            }
        }

        // 2. 海克斯效果
        for augment in active_augments {
            for tag in &augment.tags {
                let Some(group_id) = self.map_tag_to_group(tag) else {
                    continue;
                };
                sources.push(EffectSource {
                    source_type: "augment".into(),
                    source_id: augment.augment_id.clone(),
                    source_name: augment.name.clone(),
                    group_id,
                    coverage: CoverageType::Unknown,
                    weight: 90,
                });
            }
        }

        // 3. 羁绊效果
        for trait_data in active_traits {
            for tag in &trait_data.effect_tags {
                let Some(group_id) = self.map_tag_to_group(tag) else {
                    continue;
                };
                sources.push(EffectSource {
                    source_type: "trait".into(),
                    source_id: trait_data.trait_id.clone(),
                    source_name: trait_data.name.clone(),
                    group_id,
                    coverage: CoverageType::Unknown,
                    weight: 80,
                });
            }
        }

        // 4. 英雄技能效果
        for champ in champions {
            // 从技能描述推断效果（使用 damage_profile 和 casting_pattern 等）
            for risk_tag in &champ.risk_profile {
                if let Some(group_id) = self.map_tag_to_group(risk_tag) {
                    sources.push(EffectSource {
                        source_type: "champion_skill".into(),
                        source_id: champ.hero_id.clone(),
                        source_name: champ.name.clone(),
                        group_id,
                        coverage: CoverageType::Unknown,
                        weight: 70,
                    });
                }
            }
            // 检查 cast_pattern 是否暗示某些效果
            if champ.cast_pattern == "mana_cast" {
                sources.push(EffectSource {
                    source_type: "champion_skill".into(),
                    source_id: champ.hero_id.clone(),
                    source_name: champ.name.clone(),
                    group_id: "mana_engine".into(),
                    coverage: CoverageType::Unknown,
                    weight: 30,
                });
            }
        }

        sources
    }

    /// 生成队伍覆盖率报告
    pub fn build_coverage_report(
        &self,
        sources: &[EffectSource],
        equipped_item_ids: &[String],
    ) -> TeamCoverageReport {
        // group_id → 覆盖次数
        let mut group_counts: HashMap<String, usize> = HashMap::new();
        // group_id → 覆盖来源
        let mut group_sources: HashMap<String, Vec<&EffectSource>> = HashMap::new();
        // group_id → 总权重
        let mut group_weight: HashMap<String, i32> = HashMap::new();

        for source in sources {
            *group_counts.entry(source.group_id.clone()).or_default() += 1;
            group_sources
                .entry(source.group_id.clone())
                .or_default()
                .push(source);
            *group_weight.entry(source.group_id.clone()).or_default() += source.weight;
        }

        // 按覆盖权重排序的已覆盖组
        let mut covered: Vec<CoveredGroup> = group_counts
            .iter()
            .map(|(group_id, count)| {
                let label = self
                    .conflict_groups
                    .get(group_id)
                    .map(|g| g.label.clone())
                    .unwrap_or_else(|| group_id.clone());
                let total_weight = group_weight.get(group_id).copied().unwrap_or(0);
                let coverage_quality = if total_weight >= 180 {
                    "充分覆盖"
                } else if total_weight >= 100 {
                    "基本覆盖"
                } else {
                    "部分覆盖"
                };

                CoveredGroup {
                    group_id: group_id.clone(),
                    label,
                    source_count: *count,
                    total_weight,
                    coverage_quality: coverage_quality.into(),
                    primary_source: group_sources
                        .get(group_id)
                        .and_then(|s| s.first())
                        .map(|s| format!("{}:{}", s.source_type, s.source_name))
                        .unwrap_or_default(),
                }
            })
            .collect();

        covered.sort_by(|a, b| b.total_weight.cmp(&a.total_weight));

        // 未覆盖的冲突组
        let covered_ids: HashSet<&str> = group_counts.keys().map(|s| s.as_str()).collect();
        let uncovered: Vec<UncoveredGroup> = self
            .conflict_groups
            .keys()
            .filter(|id| !covered_ids.contains(id.as_str()))
            .map(|id| {
                let label = self
                    .conflict_groups
                    .get(id)
                    .map(|g| g.label.clone())
                    .unwrap_or_else(|| id.clone());
                UncoveredGroup {
                    group_id: id.clone(),
                    label,
                    recommendation: self.recommend_for_coverage(id),
                }
            })
            .collect();

        // 检查装备中是否有冲突
        let conflict_summary = if equipped_item_ids.len() > 1 {
            let mut seen: HashMap<String, Vec<String>> = HashMap::new();
            for source in sources {
                if source.source_type == "item" {
                    seen.entry(source.group_id.clone())
                        .or_default()
                        .push(source.source_name.clone());
                }
            }
            seen.into_iter()
                .filter(|(_, items)| items.len() > 1)
                .map(|(gid, items)| format!("[{}]: {}", gid, items.join("+"),))
                .collect::<Vec<_>>()
                .join("; ")
        } else {
            String::new()
        };

        let summary = if covered.is_empty() {
            "队伍无明确效果覆盖".to_string()
        } else {
            let covered_labels: Vec<String> = covered
                .iter()
                .map(|c| format!("{}({})", c.label, c.coverage_quality))
                .collect();
            let uncovered_labels: Vec<String> = uncovered.iter().map(|u| u.label.clone()).collect();
            let mut parts = vec![format!("已覆盖: {}", covered_labels.join(", "))];
            if !uncovered_labels.is_empty() {
                parts.push(format!("未覆盖: {}", uncovered_labels.join(", ")));
            }
            if !conflict_summary.is_empty() {
                parts.push(format!("装备冲突: {}", conflict_summary));
            }
            parts.join(" | ")
        };

        TeamCoverageReport {
            covered,
            uncovered,
            conflict_summary,
            summary,
            total_sources: sources.len(),
        }
    }

    /// 推荐未覆盖效果组的获取方式
    fn recommend_for_coverage(&self, group_id: &str) -> String {
        match group_id {
            "anti_heal" => "推荐合成日炎、红霸符或鬼书".into(),
            "burn" => "推荐合成日炎或红霸符".into(),
            "armor_shred" => "AD阵容推荐合成轻语或薄暮".into(),
            "mr_shred" => "AP阵容推荐合成离子火花或虚空之杖".into(),
            "mana_engine" => "法系主C推荐合成青龙刀或蓝霸符".into(),
            "crit_enable" => "暴击流主C推荐合成无尽或法爆".into(),
            "sustain" => "前排/主C推荐合成饮血或科技枪".into(),
            "shield_survival" => "防爆发推荐合成夜刃或血手".into(),
            "aura_team_buff" => "辅助棋子推荐携带圣杯、旗子等团队装".into(),
            "unique_trigger" => "每场战斗仅触发1次的装备避免重复带在同一棋子".into(),
            _ => format!("未收录推荐，请参考 {} 组", group_id),
        }
    }

    /// 将效果标签映射到冲突组 ID
    fn map_tag_to_group(&self, tag: &str) -> Option<String> {
        match tag {
            "anti_heal" | "重伤" | "减疗" | "治疗降低" => Some("anti_heal".into()),
            "burn" | "灼烧" | "燃烧" => Some("burn".into()),
            "armor_shred" | "护甲击碎" | "降低护甲" | "破甲" => {
                Some("armor_shred".into())
            }
            "mr_shred" | "魔抗击碎" | "降低魔抗" => Some("mr_shred".into()),
            "mana_engine" | "回蓝" | "法力值回复" | "启动" => Some("mana_engine".into()),
            "crit_enable" | "暴击" | "技能暴击" => Some("crit_enable".into()),
            "sustain" | "吸血" | "全能吸血" | "回复" => Some("sustain".into()),
            "shield_survival" | "护盾" | "保命" | "夜刃" => Some("shield_survival".into()),
            "aura_team_buff" | "光环" | "团队" => Some("aura_team_buff".into()),
            "unique_trigger" => Some("unique_trigger".into()),
            _ => None,
        }
    }

    /// 从装备 profile 推断覆盖方式
    fn infer_coverage_from_item(&self, profile: &ItemValueProfile) -> CoverageType {
        let all_tags: HashSet<&str> = profile.effect_tags.iter().map(|s| s.as_str()).collect();

        if all_tags.contains("anti_heal") {
            // 日炎是光环灼烧+重伤
            if profile.name.contains("日炎") {
                return CoverageType::AoeAura;
            }
            // 红霸符是普攻/技能附加
            if profile.name.contains("红霸符") {
                return CoverageType::SingleTarget;
            }
            // 鬼书
            return CoverageType::AoeSpell;
        }

        if all_tags.contains("armor_shred") {
            if profile.name.contains("薄暮") {
                return CoverageType::AoeAura;
            }
            return CoverageType::SingleTargetSpell;
        }

        if all_tags.contains("mr_shred") {
            if profile.name.contains("离子") {
                return CoverageType::AoeAura;
            }
            return CoverageType::SingleTargetSpell;
        }

        if all_tags.contains("mana_engine") {
            if profile.name.contains("青龙刀") || profile.name.contains("朔极") {
                return CoverageType::AttackTrigger;
            }
            if profile.name.contains("蓝霸符") {
                return CoverageType::FlatRegen;
            }
            if profile.name.contains("大天使") {
                return CoverageType::TimeBased;
            }
        }

        CoverageType::Unknown
    }
}

/// 已覆盖的效果组
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoveredGroup {
    pub group_id: String,
    pub label: String,
    pub source_count: usize,
    pub total_weight: i32,
    pub coverage_quality: String,
    pub primary_source: String,
}

/// 未覆盖的效果组
#[derive(Debug, Clone, serde::Serialize)]
pub struct UncoveredGroup {
    pub group_id: String,
    pub label: String,
    pub recommendation: String,
}

/// 队伍覆盖率报告
#[derive(Debug, Clone, serde::Serialize)]
pub struct TeamCoverageReport {
    pub covered: Vec<CoveredGroup>,
    pub uncovered: Vec<UncoveredGroup>,
    pub conflict_summary: String,
    pub summary: String,
    pub total_sources: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conflict_groups() -> ItemConflictGroups {
        let mut groups = HashMap::new();
        groups.insert(
            "anti_heal".into(),
            ItemConflictGroup {
                label: "重伤".into(),
                conflicts: vec![
                    hexsight_core::ConflictEntry {
                        item_id: "2029".into(),
                        item_name: "日炎斗篷".into(),
                        coverage: CoverageType::AoeAura,
                    },
                    hexsight_core::ConflictEntry {
                        item_id: "2009".into(),
                        item_name: "红霸符".into(),
                        coverage: CoverageType::SingleTarget,
                    },
                ],
                stacking_policy: hexsight_core::ConflictStackingPolicy {
                    first_source: 100,
                    second_source: 30,
                    third_plus_source: 10,
                },
                notes: String::new(),
            },
        );
        groups.insert(
            "armor_shred".into(),
            ItemConflictGroup {
                label: "破甲".into(),
                conflicts: vec![hexsight_core::ConflictEntry {
                    item_id: "2037".into(),
                    item_name: "轻语".into(),
                    coverage: CoverageType::SingleTargetSpell,
                }],
                stacking_policy: hexsight_core::ConflictStackingPolicy {
                    first_source: 100,
                    second_source: 25,
                    third_plus_source: 5,
                },
                notes: String::new(),
            },
        );
        groups.insert(
            "mana_engine".into(),
            ItemConflictGroup {
                label: "回蓝".into(),
                conflicts: vec![],
                stacking_policy: hexsight_core::ConflictStackingPolicy {
                    first_source: 100,
                    second_source: 60,
                    third_plus_source: 30,
                },
                notes: String::new(),
            },
        );

        ItemConflictGroups {
            version: "test".into(),
            groups,
        }
    }

    fn make_item_profile(id: &str, name: &str, effect_tags: Vec<&str>) -> ItemValueProfile {
        ItemValueProfile {
            item_id: id.into(),
            name: name.into(),
            item_type: "成型装备".into(),
            stats: vec![],
            effect_tags: effect_tags.into_iter().map(|s| s.to_string()).collect(),
            best_for_profiles: vec![],
            bad_for_profiles: vec![],
            replacement_group: String::new(),
            conflict_group: String::new(),
            stage_bias: String::new(),
            tier: None,
            damage_type_fit: vec![],
        }
    }

    #[test]
    fn collects_effects_from_items() {
        let groups = make_conflict_groups();
        let coverage = TeamEffectCoverage::new(&groups);

        let item = make_item_profile("2029", "日炎斗篷", vec!["anti_heal", "burn"]);
        let sources = coverage.collect_all_sources(&["2029".into()], &[&item], &[], &[], &[]);

        assert!(sources.iter().any(|s| s.group_id == "anti_heal"));
        assert!(sources.iter().any(|s| s.group_id == "burn"));
        assert_eq!(
            sources.iter().filter(|s| s.source_type == "item").count(),
            2
        );
    }

    #[test]
    fn detects_uncovered_groups() {
        let groups = make_conflict_groups();
        let coverage = TeamEffectCoverage::new(&groups);

        let item = make_item_profile("2029", "日炎斗篷", vec!["anti_heal"]);
        let sources = coverage.collect_all_sources(&["2029".into()], &[&item], &[], &[], &[]);

        let report = coverage.build_coverage_report(&sources, &["2029".into()]);
        assert!(report.uncovered.iter().any(|u| u.group_id == "armor_shred"));
        assert!(report.uncovered.iter().any(|u| u.group_id == "mana_engine"));
    }

    #[test]
    fn coverage_quality_from_weight() {
        let groups = make_conflict_groups();
        let coverage = TeamEffectCoverage::new(&groups);

        let item = make_item_profile("2029", "日炎斗篷", vec!["anti_heal"]);
        let sources = coverage.collect_all_sources(&["2029".into()], &[&item], &[], &[], &[]);

        let report = coverage.build_coverage_report(&sources, &["2029".into()]);
        let anti_heal = report
            .covered
            .iter()
            .find(|c| c.group_id == "anti_heal")
            .unwrap();
        assert_eq!(anti_heal.source_count, 1);
        assert!(anti_heal.total_weight >= 100);
    }

    #[test]
    fn empty_sources_produces_valid_report() {
        let groups = make_conflict_groups();
        let coverage = TeamEffectCoverage::new(&groups);

        let sources = coverage.collect_all_sources(&[], &[], &[], &[], &[]);
        let report = coverage.build_coverage_report(&sources, &[]);

        assert!(report.covered.is_empty());
        assert!(!report.uncovered.is_empty()); // 所有组都未覆盖
        assert_eq!(report.total_sources, 0);
    }

    #[test]
    fn map_tag_to_group_coverage() {
        let groups = make_conflict_groups();
        let coverage = TeamEffectCoverage::new(&groups);

        assert_eq!(
            coverage.map_tag_to_group("anti_heal"),
            Some("anti_heal".into())
        );
        assert_eq!(coverage.map_tag_to_group("重伤"), Some("anti_heal".into()));
        assert_eq!(
            coverage.map_tag_to_group("破甲"),
            Some("armor_shred".into())
        );
        assert_eq!(coverage.map_tag_to_group("unknown_tag"), None);
    }
}
