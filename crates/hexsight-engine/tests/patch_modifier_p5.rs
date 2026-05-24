use hexsight_core::{LineupProfile, LineupScore, PlaystyleTag};
use hexsight_engine::{PatchKnowledgeLoader, PatchModifier};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn temp_config_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("hexsight_p5_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("rules/S18.1")).expect("应创建临时规则目录");
    root
}

fn write_rules_file(root: &Path, file_name: &str, content: &str) {
    fs::write(root.join("rules/S18.1").join(file_name), content).expect("应写入临时规则文件");
}

fn profile(id: &str) -> LineupProfile {
    LineupProfile {
        lineup_id: id.into(),
        name: format!("阵容{}", id),
        base_tier: 80,
        playstyle_tags: vec![PlaystyleTag::Standard],
        final_hero_ids: vec!["hero_carry".into(), "hero_tank".into()],
        carry_hero_ids: vec!["hero_carry".into()],
        tank_hero_ids: vec!["hero_tank".into()],
        core_equipment_ids: vec!["item_core".into()],
        tank_equipment_ids: vec!["item_tank".into()],
        equipment_order_ids: vec![],
        recommended_hex_ids: vec!["augment_core".into()],
        replacement_hex_ids: vec![],
        early_hero_ids: vec![],
        mid_hero_ids: vec![],
        trait_targets: HashMap::from([("trait_core".into(), 4)]),
        strategy_texts: HashMap::new(),
        mode_specific: serde_json::Value::Null,
        carry_costs: vec![4],
        category: None,
    }
}

fn score(id: &str) -> LineupScore {
    LineupScore {
        lineup_id: id.into(),
        name: format!("阵容{}", id),
        total_score: 70,
        base_score: 80,
        item_fit_score: 50,
        champion_hit_score: 60,
        augment_fit_score: 50,
        trait_fit_score: 50,
        stage_fit_score: 70,
        economy_fit_score: 70,
        health_safety_score: 70,
        playstyle_switch_score: 50,
        rival_penalty: 0,
        difficulty_penalty: 0,
        reasons: vec![],
        risks: vec![],
        requires_augment: false,
    }
}

#[test]
fn loads_patch_knowledge_and_patch_knowledge_overrides_entries() {
    let root = temp_config_root("load");
    write_rules_file(
        &root,
        "patch_knowledge.json",
        r#"{
          "version": "S18.1",
          "entries": [
            {
              "patchVersion": "S18.1",
              "targetType": "champion",
              "targetId": "hero_carry",
              "targetName": "主C",
              "changeType": "buff",
              "impactScore": 12,
              "affectedLineups": ["line_a"],
              "reason": "技能伤害提高"
            }
          ]
        }"#,
    );
    write_rules_file(
        &root,
        "patch_knowledge_overrides.json",
        r#"{
          "version": "S18.1",
          "description": "P5 人工公告知识覆写",
          "entries": [
            {
              "patchVersion": "S18.1-hotfix",
              "targetType": "item",
              "targetId": "item_core",
              "targetName": "核心装",
              "changeType": "nerf",
              "impactScore": -8,
              "affectedLineups": ["line_a"],
              "reason": "装备属性降低"
            }
          ]
        }"#,
    );
    write_rules_file(
        &root,
        "patch_overrides.json",
        r#"{
          "version": "S18.1",
          "description": "旧规则包覆写，P5 不应从这里读取 PatchEntry",
          "entries": [
            {
              "patchVersion": "S18.1-wrong",
              "targetType": "augment",
              "targetId": "augment_core",
              "targetName": "旧文件误填",
              "changeType": "buff",
              "impactScore": 99,
              "affectedLineups": ["line_a"],
              "reason": "该条目应被忽略"
            }
          ],
          "lineup_tier_overrides": {},
          "item_synergy_overrides": {},
          "damage_overrides": {},
          "threshold_overrides": {},
          "weight_overrides": {}
        }"#,
    );

    let knowledge = PatchKnowledgeLoader::load(&root, "S18.1").expect("应加载 P5 版本知识");

    assert_eq!(knowledge.version, "S18.1");
    assert_eq!(knowledge.entries.len(), 2);
    assert!(knowledge
        .entries
        .iter()
        .any(|entry| entry.target_type == "champion"));
    assert!(knowledge
        .entries
        .iter()
        .any(|entry| entry.target_type == "item"));
    assert!(!knowledge
        .entries
        .iter()
        .any(|entry| entry.target_name == "旧文件误填"));
}

#[test]
fn patch_modifier_changes_lineup_score_and_explains_sources() {
    let root = temp_config_root("modify");
    write_rules_file(
        &root,
        "patch_knowledge.json",
        r#"{
          "version": "S18.1",
          "entries": [
            {
              "patchVersion": "S18.1",
              "targetType": "champion",
              "targetId": "hero_carry",
              "targetName": "主C",
              "changeType": "buff",
              "impactScore": 10,
              "affectedLineups": ["line_a"],
              "reason": "技能伤害提高"
            },
            {
              "patchVersion": "S18.1",
              "targetType": "item",
              "targetId": "item_core",
              "targetName": "核心装",
              "changeType": "nerf",
              "impactScore": -6,
              "affectedLineups": ["line_a"],
              "reason": "核心装备削弱"
            },
            {
              "patchVersion": "S18.1",
              "targetType": "trait",
              "targetId": "trait_core",
              "targetName": "核心羁绊",
              "changeType": "buff",
              "impactScore": 4,
              "reason": "羁绊数值提高"
            },
            {
              "patchVersion": "S18.1",
              "targetType": "augment",
              "targetId": "augment_core",
              "targetName": "核心海克斯",
              "changeType": "nerf",
              "impactScore": -3,
              "reason": "海克斯收益降低"
            }
          ]
        }"#,
    );
    let knowledge = PatchKnowledgeLoader::load(&root, "S18.1").expect("应加载 P5 版本知识");

    let adjusted = PatchModifier::apply_to_score(&score("line_a"), &profile("line_a"), &knowledge);

    assert_eq!(adjusted.total_score, 75);
    assert_eq!(adjusted.base_score, 90);
    assert_eq!(adjusted.item_fit_score, 44);
    assert_eq!(adjusted.augment_fit_score, 47);
    assert_eq!(adjusted.trait_fit_score, 54);
    assert!(adjusted
        .reasons
        .iter()
        .any(|line| line.contains("主C") && line.contains("+10")));
    assert!(adjusted
        .risks
        .iter()
        .any(|line| line.contains("核心装") && line.contains("-6")));
}

#[test]
fn unrelated_patch_entries_do_not_change_lineup() {
    let root = temp_config_root("unrelated");
    write_rules_file(
        &root,
        "patch_knowledge.json",
        r#"{
          "version": "S18.1",
          "entries": [
            {
              "patchVersion": "S18.1",
              "targetType": "champion",
              "targetId": "other_hero",
              "targetName": "其他英雄",
              "changeType": "buff",
              "impactScore": 20,
              "reason": "不相关加强"
            }
          ]
        }"#,
    );
    let knowledge = PatchKnowledgeLoader::load(&root, "S18.1").expect("应加载 P5 版本知识");

    let adjusted = PatchModifier::apply_to_score(&score("line_a"), &profile("line_a"), &knowledge);

    assert_eq!(adjusted.total_score, 70);
    assert!(adjusted.reasons.is_empty());
    assert!(adjusted.risks.is_empty());
}

#[test]
fn carry_nerf_reduces_lineup_total_and_base_score() {
    let root = temp_config_root("carry_nerf");
    write_rules_file(
        &root,
        "patch_knowledge.json",
        r#"{
          "version": "S18.1",
          "entries": [
            {
              "patchVersion": "S18.1",
              "targetType": "champion",
              "targetId": "hero_carry",
              "targetName": "主C",
              "changeType": "nerf",
              "impactScore": -12,
              "reason": "技能伤害降低"
            }
          ]
        }"#,
    );
    let knowledge = PatchKnowledgeLoader::load(&root, "S18.1").expect("应加载 P5 版本知识");

    let adjusted = PatchModifier::apply_to_score(&score("line_a"), &profile("line_a"), &knowledge);

    assert_eq!(adjusted.base_score, 68);
    assert_eq!(adjusted.total_score, 58);
    assert!(adjusted
        .risks
        .iter()
        .any(|line| line.contains("主C") && line.contains("-12")));
}

#[test]
fn core_item_buff_increases_lineup_item_fit_and_total_score() {
    let root = temp_config_root("item_buff");
    write_rules_file(
        &root,
        "patch_knowledge.json",
        r#"{
          "version": "S18.1",
          "entries": [
            {
              "patchVersion": "S18.1",
              "targetType": "item",
              "targetId": "item_core",
              "targetName": "核心装",
              "changeType": "buff",
              "impactScore": 9,
              "reason": "装备属性提高"
            }
          ]
        }"#,
    );
    let knowledge = PatchKnowledgeLoader::load(&root, "S18.1").expect("应加载 P5 版本知识");

    let adjusted = PatchModifier::apply_to_score(&score("line_a"), &profile("line_a"), &knowledge);

    assert_eq!(adjusted.item_fit_score, 59);
    assert_eq!(adjusted.total_score, 79);
    assert!(adjusted
        .reasons
        .iter()
        .any(|line| line.contains("核心装") && line.contains("+9")));
}
