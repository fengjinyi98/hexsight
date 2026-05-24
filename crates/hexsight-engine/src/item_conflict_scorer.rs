// 装备冲突评分器
// 核心职责：
// - 加载冲突组配置（item_conflict_groups.json）
// - 检测已有装备中的同类效果重复（重伤、灼烧、减抗、回蓝、续航等）
// - 按边际收益降权，输出冲突警告和降权分
// - 支持环境修正（对手回复多、前排厚、爆发高等场景）

use std::collections::{HashMap, HashSet};
use std::path::Path;

use hexsight_core::{
    ConflictEnvironment, ConflictWarning, CoverageType, EffectSource, EnvironmentModifier,
    ItemConflictGroup, ItemConflictGroups, ItemValueProfile, StackingPolicies, TeamConflictReport,
};

/// 冲突组配置加载器
pub struct ConflictGroupLoader;

impl ConflictGroupLoader {
    /// 从 JSON 文件加载冲突组配置
    pub fn load(path: &Path) -> Result<ItemConflictGroups, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取冲突组配置失败 {}: {}", path.display(), e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析冲突组配置失败: {}", e))
    }
}

/// 堆叠策略配置加载器
pub struct StackingPolicyLoader;

impl StackingPolicyLoader {
    /// 从 JSON 文件加载堆叠策略配置
    pub fn load(path: &Path) -> Result<StackingPolicies, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("读取堆叠策略配置失败 {}: {}", path.display(), e))?;
        serde_json::from_str(&content).map_err(|e| format!("解析堆叠策略配置失败: {}", e))
    }
}

/// 装备冲突评分器
pub struct ItemConflictScorer {
    groups: HashMap<String, ItemConflictGroup>,
    /// item_id → 所属冲突组列表 的索引
    item_to_groups: HashMap<String, Vec<String>>,
    configured_value_per_source: Option<Vec<i32>>,
    environment_modifiers: HashMap<String, EnvironmentModifier>,
}

impl ItemConflictScorer {
    /// 从冲突组配置构建评分器
    pub fn new(groups_config: &ItemConflictGroups) -> Self {
        Self::build(groups_config, None, HashMap::new())
    }

    /// 从冲突组配置和堆叠策略配置构建评分器
    pub fn new_with_stacking_policies(
        groups_config: &ItemConflictGroups,
        stacking_policies: &StackingPolicies,
    ) -> Self {
        let configured_value_per_source = stacking_policies
            .policies
            .get(&stacking_policies.default_policy)
            .map(|policy| policy.value_per_source.clone())
            .unwrap_or_else(|| vec![100, 40, 10]);
        Self::build(
            groups_config,
            Some(configured_value_per_source),
            stacking_policies.environment_modifiers.clone(),
        )
    }

    fn build(
        groups_config: &ItemConflictGroups,
        configured_value_per_source: Option<Vec<i32>>,
        environment_modifiers: HashMap<String, EnvironmentModifier>,
    ) -> Self {
        let mut item_to_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (group_id, group) in &groups_config.groups {
            for entry in &group.conflicts {
                item_to_groups
                    .entry(entry.item_id.clone())
                    .or_default()
                    .push(group_id.clone());
            }
        }

        Self {
            groups: groups_config.groups.clone(),
            item_to_groups,
            configured_value_per_source,
            environment_modifiers,
        }
    }

    /// 批量检测装备列表中的冲突
    /// 返回每件装备的冲突警告列表
    pub fn detect_conflicts(&self, item_ids: &[String]) -> Vec<ConflictWarning> {
        self.detect_conflicts_with_environment(item_ids, &ConflictEnvironment::default())
    }

    /// 在指定环境下批量检测装备列表中的冲突
    pub fn detect_conflicts_with_environment(
        &self,
        item_ids: &[String],
        environment: &ConflictEnvironment,
    ) -> Vec<ConflictWarning> {
        let sources = self.sources_from_item_ids(item_ids);
        self.warnings_from_sources(&sources, environment)
    }

    fn sources_from_item_ids(&self, item_ids: &[String]) -> Vec<EffectSource> {
        let mut sources = Vec::new();
        for item_id in item_ids {
            if let Some(group_ids) = self.item_to_groups.get(item_id) {
                for group_id in group_ids {
                    let Some(group) = self.groups.get(group_id) else {
                        continue;
                    };
                    let Some(entry) = group.conflicts.iter().find(|e| &e.item_id == item_id) else {
                        continue;
                    };
                    sources.push(EffectSource {
                        source_type: "item".into(),
                        source_id: item_id.clone(),
                        source_name: entry.item_name.clone(),
                        group_id: group_id.clone(),
                        coverage: entry.coverage.clone(),
                        weight: 100,
                    });
                }
            }
        }
        sources
    }

    fn warnings_from_sources(
        &self,
        sources: &[EffectSource],
        environment: &ConflictEnvironment,
    ) -> Vec<ConflictWarning> {
        let mut warnings: Vec<ConflictWarning> = Vec::new();
        // group_id → 已出现的来源数
        let mut group_counts: HashMap<String, usize> = HashMap::new();
        // group_id → 已有覆盖方式集合
        let mut group_coverages: HashMap<String, HashSet<CoverageType>> = HashMap::new();
        // group_id → 来源摘要
        let mut group_source_labels: HashMap<String, Vec<String>> = HashMap::new();

        for source in sources {
            *group_counts.entry(source.group_id.clone()).or_default() += 1;
            group_coverages
                .entry(source.group_id.clone())
                .or_default()
                .insert(source.coverage.clone());
            group_source_labels
                .entry(source.group_id.clone())
                .or_default()
                .push(format!("{}:{}", source.source_type, source.source_name));
        }

        for (group_id, count) in group_counts {
            let Some(group) = self.groups.get(&group_id) else {
                continue;
            };
            if count <= 1 {
                continue;
            }

            let retained_value = self.retained_value(group, count);
            let mut penalty_score = -(100 - retained_value);

            let coverages = group_coverages.get(&group_id);
            let has_diverse_coverage = coverages.map(|c| c.len() > 1).unwrap_or(false);
            if has_diverse_coverage {
                penalty_score = (penalty_score as f64 * 0.6) as i32;
            }

            penalty_score =
                self.apply_environment_adjustment(&group_id, penalty_score, environment);

            let remaining_value = if has_diverse_coverage {
                format!("覆盖方式不同({}种)，可互补", coverages.unwrap().len())
            } else {
                "重复效果边际收益下降".to_string()
            };
            let source_summary = group_source_labels
                .get(&group_id)
                .map(|labels| labels.join("、"))
                .unwrap_or_default();

            let explanation = format!(
                "冲突组[{}]: 已有 {} 个{}来源，重复收益下降{}%，来源: {}{}",
                group.label,
                count - 1,
                group.label,
                -penalty_score,
                source_summary,
                if has_diverse_coverage {
                    "，覆盖方式不同"
                } else {
                    ""
                }
            );

            warnings.push(ConflictWarning {
                group_id: group_id.clone(),
                group_label: group.label.clone(),
                existing_sources: count - 1,
                penalty_score,
                remaining_value,
                explanation,
            });
        }

        warnings.sort_by(|a, b| {
            a.group_id
                .cmp(&b.group_id)
                .then(a.penalty_score.cmp(&b.penalty_score))
        });

        warnings
    }

    fn retained_value(&self, group: &ItemConflictGroup, count: usize) -> i32 {
        if let Some(value_per_source) = &self.configured_value_per_source {
            return value_per_source
                .get(count.saturating_sub(1))
                .copied()
                .or_else(|| value_per_source.last().copied())
                .unwrap_or(100);
        }

        let policy = &group.stacking_policy;
        match count {
            0 | 1 => policy.first_source,
            2 => policy.second_source,
            _ => policy.third_plus_source,
        }
    }

    fn apply_environment_adjustment(
        &self,
        group_id: &str,
        penalty_score: i32,
        environment: &ConflictEnvironment,
    ) -> i32 {
        if !self.environment_modifiers.is_empty() {
            let relief: i32 = self
                .active_environment_modifier_keys(environment)
                .into_iter()
                .filter_map(|key| self.environment_modifiers.get(key))
                .filter(|modifier| modifier.groups.iter().any(|group| group == group_id))
                .map(|modifier| modifier.penalty_relief)
                .sum();
            return (penalty_score + relief).min(0);
        }

        let relief = match group_id {
            "anti_heal" if environment.opponent_heal_heavy => 10,
            "armor_shred" if environment.own_ad_heavy && environment.opponent_frontline_thick => 25,
            "armor_shred" if environment.own_ad_heavy => 25,
            "armor_shred" if environment.opponent_frontline_thick => 10,
            "mr_shred" if environment.own_ap_heavy => 25,
            "shield_survival" | "unique_trigger" if environment.opponent_burst_heavy => 15,
            _ => 0,
        };
        (penalty_score + relief).min(0)
    }

    fn active_environment_modifier_keys(
        &self,
        environment: &ConflictEnvironment,
    ) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if environment.opponent_heal_heavy {
            keys.push("opponent_heal_heavy");
        }
        if environment.opponent_frontline_thick {
            keys.push("opponent_frontline_thick");
        }
        if environment.opponent_burst_heavy {
            keys.push("opponent_burst_heavy");
        }
        if environment.own_ap_heavy {
            keys.push("own_ap_heavy");
        }
        if environment.own_ad_heavy {
            keys.push("own_ad_heavy");
        }
        keys
    }

    /// 基于装备、海克斯、羁绊、棋子技能等统一来源检测冲突
    pub fn detect_source_conflicts(
        &self,
        sources: &[EffectSource],
        environment: &ConflictEnvironment,
    ) -> TeamConflictReport {
        let warnings = self.warnings_from_sources(sources, environment);
        let total_penalty: i32 = warnings.iter().map(|w| w.penalty_score).sum();

        let mut item_conflicts: HashMap<String, Vec<ConflictWarning>> = HashMap::new();
        for warning in &warnings {
            for source in sources.iter().filter(|source| {
                source.source_type == "item" && source.group_id == warning.group_id
            }) {
                item_conflicts
                    .entry(source.source_id.clone())
                    .or_default()
                    .push(warning.clone());
            }
        }

        let covered: HashSet<String> = sources
            .iter()
            .map(|source| source.group_id.clone())
            .collect();
        let all_groups: HashSet<String> = self.groups.keys().cloned().collect();
        let mut uncovered: Vec<String> = all_groups.difference(&covered).cloned().collect();
        let mut covered_vec: Vec<String> = covered.into_iter().collect();
        covered_vec.sort();
        uncovered.sort();

        let source_summary = sources
            .iter()
            .map(|source| format!("{}:{}", source.source_type, source.source_name))
            .collect::<Vec<_>>()
            .join("、");
        let warning_summary = warnings
            .iter()
            .map(|w| format!("{} (降权{})", w.group_label, w.penalty_score))
            .collect::<Vec<_>>()
            .join("; ");
        let coverage_explanation = if warning_summary.is_empty() {
            format!("来源: {} | 无效果冲突", source_summary)
        } else {
            format!("来源: {} | 冲突警告: {}", source_summary, warning_summary)
        };

        TeamConflictReport {
            item_conflicts,
            covered_groups: covered_vec,
            uncovered_groups: uncovered,
            total_penalty,
            coverage_explanation,
        }
    }

    /// 检测队伍级别的冲突报告
    pub fn detect_team_conflicts(
        &self,
        item_ids: &[String],
        _item_profiles: &[&ItemValueProfile],
    ) -> TeamConflictReport {
        let mut sources = self.sources_from_item_ids(item_ids);
        self.add_profile_sources(&mut sources, item_ids, _item_profiles);
        self.detect_source_conflicts(&sources, &ConflictEnvironment::default())
    }

    fn add_profile_sources(
        &self,
        sources: &mut Vec<EffectSource>,
        item_ids: &[String],
        item_profiles: &[&ItemValueProfile],
    ) {
        let known: HashSet<(String, String)> = sources
            .iter()
            .map(|source| (source.source_id.clone(), source.group_id.clone()))
            .collect();
        for item_id in item_ids {
            let Some(profile) = item_profiles
                .iter()
                .find(|profile| profile.item_id == *item_id)
            else {
                continue;
            };
            if profile.conflict_group.is_empty() {
                continue;
            }
            if known.contains(&(item_id.clone(), profile.conflict_group.clone())) {
                continue;
            }
            sources.push(EffectSource {
                source_type: "item".into(),
                source_id: item_id.clone(),
                source_name: profile.name.clone(),
                group_id: profile.conflict_group.clone(),
                coverage: CoverageType::Unknown,
                weight: 100,
            });
        }
    }

    /// 获取某件装备的冲突组
    pub fn get_conflict_groups(&self, item_id: &str) -> Vec<&ItemConflictGroup> {
        self.item_to_groups
            .get(item_id)
            .map(|ids| ids.iter().filter_map(|id| self.groups.get(id)).collect())
            .unwrap_or_default()
    }

    /// 判断两个装备是否属于同一冲突组
    pub fn are_conflicting(&self, item_a: &str, item_b: &str) -> Option<String> {
        let groups_a = self.item_to_groups.get(item_a)?;
        let groups_b = self.item_to_groups.get(item_b)?;
        for ga in groups_a {
            if groups_b.contains(ga) {
                return Some(ga.clone());
            }
        }
        None
    }

    /// 计算叠加权重（按堆叠策略）
    pub fn stacking_weight(&self, group_id: &str, source_index: usize) -> i32 {
        let Some(group) = self.groups.get(group_id) else {
            return 100;
        };
        let policy = &group.stacking_policy;
        match source_index {
            0 => policy.first_source,
            1 => policy.second_source,
            _ => policy.third_plus_source,
        }
    }

    /// 计算默认堆叠策略权重
    pub fn default_stacking_weight(&self, source_index: usize) -> i32 {
        self.configured_value_per_source
            .as_ref()
            .and_then(|values| {
                values
                    .get(source_index)
                    .copied()
                    .or_else(|| values.last().copied())
            })
            .unwrap_or(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hexsight_core::{
        ConflictEntry, ConflictEnvironment, EffectSource, EnvironmentModifier, StackingPolicies,
        StackingPolicyDef,
    };

    fn make_conflict_groups() -> ItemConflictGroups {
        let mut groups = HashMap::new();
        groups.insert(
            "anti_heal".into(),
            ItemConflictGroup {
                label: "重伤".into(),
                conflicts: vec![
                    ConflictEntry {
                        item_id: "2009".into(),
                        item_name: "红霸符".into(),
                        coverage: CoverageType::SingleTarget,
                    },
                    ConflictEntry {
                        item_id: "2029".into(),
                        item_name: "日炎斗篷".into(),
                        coverage: CoverageType::AoeAura,
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
                conflicts: vec![
                    ConflictEntry {
                        item_id: "2037".into(),
                        item_name: "轻语".into(),
                        coverage: CoverageType::SingleTargetSpell,
                    },
                    ConflictEntry {
                        item_id: "2032".into(),
                        item_name: "薄暮".into(),
                        coverage: CoverageType::AoeAura,
                    },
                ],
                stacking_policy: hexsight_core::ConflictStackingPolicy {
                    first_source: 100,
                    second_source: 25,
                    third_plus_source: 5,
                },
                notes: String::new(),
            },
        );

        ItemConflictGroups {
            version: "test".into(),
            groups,
        }
    }

    #[test]
    fn no_conflict_with_single_item() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let warnings = scorer.detect_conflicts(&["2009".into()]);
        assert!(warnings.is_empty(), "单件装备不应有冲突");
    }

    #[test]
    fn detects_anti_heal_conflict() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let warnings = scorer.detect_conflicts(&["2009".into(), "2029".into()]);
        assert!(!warnings.is_empty(), "日炎+红霸符应有重伤冲突");
        let anti_heal_warn = warnings.iter().find(|w| w.group_id == "anti_heal").unwrap();
        assert_eq!(anti_heal_warn.penalty_score, -42); // -70 * 0.6 (diverse coverage)
    }

    #[test]
    fn no_conflict_across_different_groups() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let warnings = scorer.detect_conflicts(&["2009".into(), "2037".into()]);
        // 重伤和破甲不同冲突组
        let anti_heal = warnings.iter().any(|w| w.group_id == "anti_heal");
        let armor = warnings.iter().any(|w| w.group_id == "armor_shred");
        assert!(!anti_heal, "红霸符单独不应有重伤冲突");
        assert!(!armor, "轻语单独不应有破甲冲突");
    }

    #[test]
    fn diverse_coverage_reduces_penalty() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        // 轻语(单体) + 薄暮(光环) = 不同覆盖方式
        let warnings = scorer.detect_conflicts(&["2037".into(), "2032".into()]);
        let armor_warn = warnings
            .iter()
            .find(|w| w.group_id == "armor_shred")
            .unwrap();
        // 重叠破甲: second_source=25, penalty=-(100-25)=-75, adjusted *0.6 = -45
        assert_eq!(armor_warn.penalty_score, -45);
    }

    #[test]
    fn are_conflicting_detection() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        assert_eq!(
            scorer.are_conflicting("2009", "2029"),
            Some("anti_heal".into())
        );
        assert_eq!(scorer.are_conflicting("2009", "2037"), None);
    }

    #[test]
    fn team_conflict_report_generates_coverage() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let report = scorer.detect_team_conflicts(&["2009".into(), "2037".into()], &[]);
        assert!(report.covered_groups.contains(&"anti_heal".into()));
        assert!(report.covered_groups.contains(&"armor_shred".into()));
        assert_eq!(report.total_penalty, 0); // 无重复
    }

    #[test]
    fn stacking_weights() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        assert_eq!(scorer.stacking_weight("anti_heal", 0), 100);
        assert_eq!(scorer.stacking_weight("anti_heal", 1), 30);
        assert_eq!(scorer.stacking_weight("anti_heal", 2), 10);
    }

    #[test]
    fn stacking_policy_config_loads_snake_case_and_drives_default_weight() {
        let rules_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../config/rules/S18.1");
        let policies = StackingPolicyLoader::load(&rules_dir.join("stacking_policy.json"))
            .expect("S18.1 堆叠策略配置应可加载");
        let groups = ConflictGroupLoader::load(&rules_dir.join("item_conflict_groups.json"))
            .expect("S18.1 冲突组配置应可加载");

        let scorer = ItemConflictScorer::new_with_stacking_policies(&groups, &policies);

        assert_eq!(policies.default_policy, "diminishing_moderate");
        assert_eq!(scorer.default_stacking_weight(1), 40);
        assert!(policies.environment_modifiers.contains_key("own_ap_heavy"));
    }

    #[test]
    fn stacking_policy_values_drive_warning_penalty() {
        let groups = make_conflict_groups();
        let policies = StackingPolicies {
            version: "test".into(),
            policies: HashMap::from([(
                "full_stack".into(),
                StackingPolicyDef {
                    description: "测试完全叠加".into(),
                    value_per_source: vec![100, 100, 100],
                },
            )]),
            default_policy: "full_stack".into(),
            environment_modifiers: HashMap::new(),
        };
        let scorer = ItemConflictScorer::new_with_stacking_policies(&groups, &policies);

        let warnings = scorer.detect_conflicts(&["2009".into(), "2029".into()]);
        let anti_heal = warnings.iter().find(|w| w.group_id == "anti_heal").unwrap();

        assert_eq!(anti_heal.penalty_score, 0);
    }

    #[test]
    fn environment_modifier_config_values_drive_penalty_relief() {
        let groups = make_conflict_groups();
        let policies = StackingPolicies {
            version: "test".into(),
            policies: HashMap::from([(
                "group_equivalent".into(),
                StackingPolicyDef {
                    description: "测试等同冲突组策略".into(),
                    value_per_source: vec![100, 30, 10],
                },
            )]),
            default_policy: "group_equivalent".into(),
            environment_modifiers: HashMap::from([(
                "opponent_heal_heavy".into(),
                EnvironmentModifier {
                    description: "测试回复环境".into(),
                    effect: String::new(),
                    groups: vec!["anti_heal".into()],
                    penalty_relief: 30,
                },
            )]),
        };
        let scorer = ItemConflictScorer::new_with_stacking_policies(&groups, &policies);

        let adjusted = scorer.detect_conflicts_with_environment(
            &["2009".into(), "2029".into()],
            &ConflictEnvironment {
                opponent_heal_heavy: true,
                ..Default::default()
            },
        );
        let anti_heal = adjusted.iter().find(|w| w.group_id == "anti_heal").unwrap();

        assert_eq!(anti_heal.penalty_score, -12);
    }

    #[test]
    fn environment_reduces_penalty_for_relevant_matchups() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let normal = scorer.detect_conflicts(&["2009".into(), "2029".into()]);
        let heal_heavy = scorer.detect_conflicts_with_environment(
            &["2009".into(), "2029".into()],
            &ConflictEnvironment {
                opponent_heal_heavy: true,
                ..Default::default()
            },
        );

        let normal_penalty = normal
            .iter()
            .find(|w| w.group_id == "anti_heal")
            .unwrap()
            .penalty_score;
        let adjusted_penalty = heal_heavy
            .iter()
            .find(|w| w.group_id == "anti_heal")
            .unwrap()
            .penalty_score;
        assert!(adjusted_penalty > normal_penalty);
    }

    #[test]
    fn effect_sources_from_multiple_types_join_conflict_scoring() {
        let scorer = ItemConflictScorer::new(&make_conflict_groups());
        let sources = vec![
            EffectSource {
                source_type: "item".into(),
                source_id: "2029".into(),
                source_name: "日炎斗篷".into(),
                group_id: "anti_heal".into(),
                coverage: CoverageType::AoeAura,
                weight: 100,
            },
            EffectSource {
                source_type: "trait".into(),
                source_id: "void".into(),
                source_name: "虚空".into(),
                group_id: "anti_heal".into(),
                coverage: CoverageType::Unknown,
                weight: 80,
            },
        ];

        let report = scorer.detect_source_conflicts(&sources, &ConflictEnvironment::default());

        assert!(report.total_penalty < 0);
        assert!(report.covered_groups.contains(&"anti_heal".into()));
        assert!(report.item_conflicts.contains_key("2029"));
        assert!(report.coverage_explanation.contains("trait:虚空"));
    }

    #[test]
    fn real_s18_config_detects_item_conflicts_from_live_equipment_ids() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let groups = ConflictGroupLoader::load(
            &repo_root.join("config/rules/S18.1/item_conflict_groups.json"),
        )
        .expect("S18.1 冲突组配置应可加载");
        let equip_json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(repo_root.join("config/game_data/mode17/equip.json")).unwrap(),
        )
        .unwrap();
        let equips = equip_json
            .get("data")
            .and_then(|data| data.as_object())
            .expect("equip.json 应包含 data 装备 ID map");
        for (id, name) in [
            ("2009", "红霸符"),
            ("2029", "日炎斗篷"),
            ("2019", "离子火花"),
            ("2011", "虚空之杖"),
            ("2037", "最后的轻语"),
            ("2032", "薄暮法袍"),
        ] {
            let item = equips
                .get(id)
                .unwrap_or_else(|| panic!("真实装备数据缺少 {}", id));
            assert_eq!(
                item.get("name").and_then(|value| value.as_str()),
                Some(name)
            );
        }

        let scorer = ItemConflictScorer::new(&groups);
        let burn_heal = scorer.detect_conflicts(&["2009".into(), "2029".into()]);
        let mr = scorer.detect_conflicts(&["2019".into(), "2011".into()]);
        let armor = scorer.detect_conflicts(&["2037".into(), "2032".into()]);

        assert!(burn_heal.iter().any(|w| w.group_id == "anti_heal"));
        assert!(burn_heal.iter().any(|w| w.group_id == "burn"));
        assert!(mr.iter().any(|w| w.group_id == "mr_shred"));
        assert!(armor.iter().any(|w| w.group_id == "armor_shred"));
    }
}
