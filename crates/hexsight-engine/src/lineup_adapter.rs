// 官方阵容数据适配器
// 核心职责：
// - 将 lineup_detail_total.json 的原始条目解析为稳定的 LineupCardData 领域模型
// - 集中处理 detail 二次 JSON 解析与字段兼容优先级
// - 隔离追版本时的官方字段变化

use serde_json::Value;

use hexsight_core::{
    ChosenBackupData, GodRewardData, HeroReplacementData, LineupCardData, LineupDetailData,
    LineupPieceData, LineupRawItem, TraitContactData, UnlockTaskData,
};

/// 阵容适配器
pub struct LineupAdapter;

impl LineupAdapter {
    /// 从原始条目列表解析为阵容卡片列表
    pub fn cards_from_raw_list(raw_list: &[LineupRawItem], _mode: &str) -> Vec<LineupCardData> {
        let mut cards: Vec<LineupCardData> =
            raw_list.iter().filter_map(Self::card_from_raw).collect();
        // 按 category 再按 name 排序
        cards.sort_by(|a, b| {
            a.category
                .as_deref()
                .unwrap_or("")
                .cmp(b.category.as_deref().unwrap_or(""))
                .then_with(|| a.name.cmp(&b.name))
        });
        cards
    }

    /// 从单个原始条目解析为 LineupCardData
    pub fn card_from_raw(raw: &LineupRawItem) -> Option<LineupCardData> {
        let detail_payload = Self::parse_detail_payload(&raw.detail);
        let parsed_detail = Self::detail_from_payload(&detail_payload);

        // 阵容名：优先 detail.line_name > raw 其他字段
        let resolved_name = detail_payload
            .get("line_name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                raw.extra
                    .get("name")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or("未知阵容");

        // ID 优先级: raw.id > raw.extra.queue_id > resolvedName
        let rid = raw.id.clone();
        let qid = raw
            .extra
            .get("queue_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        let id = [rid, qid, resolved_name.to_string()]
            .into_iter()
            .find(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown".to_string());

        // 作者优先级：lineupauthor_data.name > detail.author_littlelegend.desc > raw.author
        let author_data = &raw.lineupauthor_data;
        let little_legend = detail_payload.get("author_littlelegend");
        let author = author_data
            .get("name")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                little_legend.and_then(|ll| {
                    ll.get("desc")
                        .and_then(|v| v.as_str())
                        .or_else(|| ll.get("item_name").and_then(|v| v.as_str()))
                })
            })
            .or(Some(raw.author.as_str()))
            .unwrap_or("未知作者");

        // 头像优先级
        let author_avatar = author_data
            .get("imgUrl")
            .and_then(|v| v.as_str())
            .or_else(|| {
                little_legend.and_then(|ll| {
                    ll.get("imagePath")
                        .and_then(|v| v.as_str())
                        .or_else(|| ll.get("icon").and_then(|v| v.as_str()))
                })
            })
            .unwrap_or("")
            .to_string();

        // 羁绊标签：优先 contact[].name > 从阵容名提取【】
        let contacts = detail_payload
            .get("contact")
            .and_then(|v| v.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);
        let contact_traits: Vec<String> = contacts
            .iter()
            .filter_map(|c| c.get("name").and_then(|n| n.as_str()))
            .map(|s| s.to_string())
            .collect();
        let traits = if contact_traits.is_empty() {
            extract_bracket_traits(resolved_name)
        } else {
            contact_traits
        };

        let line_tag = detail_payload
            .get("line_tag")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i32>().ok());
        let category = line_tag
            .and_then(|t| match t {
                1 => Some("新手推荐"),
                2 => Some("高手进阶"),
                3 => Some("趣味娱乐"),
                _ => None,
            })
            .map(|s| s.to_string());

        let tags = Self::parse_tags(&detail_payload, category.as_deref());
        let top4_rate = Self::parse_top4_rate(&detail_payload);

        Some(LineupCardData {
            id,
            name: resolved_name.to_string(),
            author: author.to_string(),
            author_avatar,
            quality: if raw.quality.is_empty() {
                "A".to_string()
            } else {
                raw.quality.clone()
            },
            traits,
            category,
            tags,
            top4_rate,
            detail: parsed_detail,
        })
    }

    /// 从 detail payload 解析 LineupDetailData
    pub fn detail_from_payload(payload: &Value) -> LineupDetailData {
        let hex_buff = payload.get("hexbuff");

        LineupDetailData {
            final_heroes: parse_pieces(payload.get("hero_location")),
            early_heroes: parse_pieces(payload.get("y21_early_heros")),
            mid_heroes: parse_pieces(payload.get("y21_metaphase_heros")),
            recommended_hex_ids: split_ids(
                hex_buff
                    .and_then(|h| h.get("recomm"))
                    .and_then(|v| v.as_str()),
            ),
            replacement_hex_ids: split_ids(
                hex_buff
                    .and_then(|h| h.get("replace"))
                    .and_then(|v| v.as_str()),
            ),
            equipment_order_ids: split_ids(payload.get("equipment_order").and_then(|v| v.as_str())),
            level_3_hero_ids: split_ids(payload.get("level_3_heros").and_then(|v| v.as_str())),
            hero_replacements: parse_hero_replacements(payload.get("hero_replace")),
            unlock_tasks: parse_unlock_tasks(payload.get("task_list")),
            god_rewards: parse_god_rewards(payload.get("god_list")),
            official_traits: parse_trait_contacts(payload.get("contact")),
            early_traits: parse_trait_contacts(payload.get("y21_early_heros_contact")),
            mid_traits: parse_trait_contacts(payload.get("y21_metaphase_heros_contact")),
            chosen_contact: parse_single_trait(payload.get("chosen_contact")),
            messenger_contact: parse_single_trait(payload.get("messengerContact")),
            chosen_backups: parse_chosen_backups(payload.get("chosen_backup")),
            line_feature: value_str(payload.get("line_feature")),
            early_info: value_str(payload.get("early_info")),
            d_time: value_str(payload.get("d_time")),
            location_info: value_str(payload.get("location_info")),
            location_info2: value_str(payload.get("location_info_2")),
            enemy_info: value_str(payload.get("enemy_info")),
            hex_info: value_str(payload.get("hex_info")),
            equipment_info: value_str(payload.get("equipment_info")),
            god_reward_info: value_str(payload.get("godreward_info")),
            task_info: value_str(payload.get("task_info")),
            chosen_info: value_str(payload.get("chosen_info")),
            early_round: value_str(payload.get("early_round")),
            mid_round: value_str(payload.get("metaphase_round")),
            staff_info: value_str(payload.get("staff_info")),
            goop_info: value_str(payload.get("goop_info")),
            trait_party_info: value_str(payload.get("traitparty_info")),
            legend_galaxy_info: value_str(payload.get("legendgalaxyinfo")),
        }
    }

    // ---- 内部辅助 ----

    /// 解析 detail 字段（尝试 dict，再尝试二次 JSON 字符串解析）
    fn parse_detail_payload(detail_str: &str) -> Value {
        if detail_str.is_empty() {
            return Value::Object(serde_json::Map::new());
        }
        // 直接作为 JSON 解析（如果 detail 本身就是对象）
        if let Ok(v) = serde_json::from_str::<Value>(detail_str) {
            if v.is_object() && !v.as_object().unwrap().is_empty() {
                return v;
            }
        }
        Value::Object(serde_json::Map::new())
    }

    fn parse_tags(detail_payload: &Value, fallback_category: Option<&str>) -> Vec<String> {
        let smar_tag = detail_payload.get("smar_lineup_tag");
        let all_tag = smar_tag.and_then(|s| s.get("all"));
        if let Some(tags) = all_tag
            .and_then(|a| a.get("tag"))
            .and_then(|t| t.as_array())
        {
            return tags
                .iter()
                .filter_map(|t| t.as_str())
                .map(|s| s.to_string())
                .collect();
        }
        if let Some(tag) = all_tag.and_then(|a| a.get("tag")).and_then(|t| t.as_str()) {
            return vec![tag.to_string()];
        }
        if let Some(cat) = fallback_category {
            return vec![cat.to_string()];
        }
        vec![]
    }

    fn parse_top4_rate(detail_payload: &Value) -> f64 {
        detail_payload
            .get("smar_lineup_tag")
            .and_then(|s| s.get("all"))
            .and_then(|a| a.get("rate"))
            .and_then(|r| r.get("top4_rate"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0)
            * 100.0
    }
}

// ============================================================
// 子解析函数
// ============================================================

/// 解析棋子列表
fn parse_pieces(value: Option<&Value>) -> Vec<LineupPieceData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_piece).collect())
        .unwrap_or_default()
}

/// 解析单个棋子
fn parse_piece(val: &Value) -> Option<LineupPieceData> {
    let hero_id = val.get("hero_id").and_then(|v| v.as_str()).unwrap_or("");
    if hero_id.is_empty() {
        return None;
    }

    let location = val
        .get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("0,0");
    let parts: Vec<i32> = location
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if parts.len() != 2 {
        return None;
    }

    Some(LineupPieceData {
        id_in_lineup: val
            .get("idInLineup")
            .and_then(parse_flexible_int)
            .unwrap_or(0),
        chess_type: val
            .get("chess_type")
            .and_then(|v| v.as_str())
            .unwrap_or("hero")
            .to_string(),
        hero_id: hero_id.to_string(),
        equipment_ids: split_ids(val.get("equipment_id").and_then(|v| v.as_str())),
        is_carry_hero: val
            .get("is_carry_hero")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        row: parts[0],
        col: parts[1],
        location_key: format!("{},{}", parts[0], parts[1]),
    })
}

/// 解析羁绊计数列表
fn parse_trait_contacts(value: Option<&Value>) -> Vec<TraitContactData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_trait_contact).collect())
        .unwrap_or_default()
}

/// 解析单个羁绊计数
fn parse_trait_contact(val: &Value) -> Option<TraitContactData> {
    let id = parse_flexible_id_str(&val["id"]).unwrap_or_default();
    let contact_type = val
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if id.is_empty() && contact_type.is_empty() {
        return None;
    }

    Some(TraitContactData {
        id,
        contact_type,
        count: val.get("num").and_then(parse_flexible_int).unwrap_or(0),
        color: val.get("color").and_then(parse_flexible_int).unwrap_or(0),
        level: val.get("level").and_then(parse_flexible_int).unwrap_or(0),
    })
}

/// 解析单个可选羁绊
fn parse_single_trait(value: Option<&Value>) -> Option<TraitContactData> {
    value.and_then(|v| v.as_object()).and_then(|obj| {
        if obj.is_empty() {
            return None;
        }
        parse_trait_contact(value.unwrap())
    })
}

/// 解析英雄替换列表
fn parse_hero_replacements(value: Option<&Value>) -> Vec<HeroReplacementData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let hero_id = v
                        .get("hero_id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    if hero_id.is_empty() {
                        return None;
                    }
                    Some(HeroReplacementData {
                        hero_id,
                        replacement_hero_ids: split_ids(
                            v.get("replace_heros").and_then(|s| s.as_str()),
                        ),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 解析解锁任务列表
fn parse_unlock_tasks(value: Option<&Value>) -> Vec<UnlockTaskData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let task_id = v
                        .get("task_id")
                        .and_then(parse_flexible_id_str)
                        .unwrap_or_default();
                    if task_id.is_empty() {
                        return None;
                    }
                    Some(UnlockTaskData {
                        // hero_id = task_id 去掉末尾 2 位任务序号（对齐 Swift dropLast(2)）
                        hero_id: if task_id.len() >= 2 {
                            task_id[..task_id.len() - 2].to_string()
                        } else {
                            task_id.clone()
                        },
                        chess_id: v
                            .get("chess_id")
                            .and_then(parse_flexible_id_str)
                            .unwrap_or_default(),
                        task_id,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 解析星神奖励列表
fn parse_god_rewards(value: Option<&Value>) -> Vec<GodRewardData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let god_id = v
                        .get("god_id")
                        .and_then(parse_flexible_id_str)
                        .unwrap_or_default();
                    if god_id.is_empty() {
                        return None;
                    }
                    Some(GodRewardData {
                        stage: v.get("stage_num").and_then(parse_flexible_int).unwrap_or(0),
                        god_id,
                        wish_ids: parse_wish_ids(v.get("wishes")),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// 解析 wish ID 列表（兼容字符串和数组格式）
fn parse_wish_ids(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(parse_flexible_id_str)
            .filter(|s| !s.is_empty())
            .collect(),
        Some(Value::String(s)) => split_ids(Some(s)),
        _ => split_ids(value.and_then(|v| v.as_str())),
    }
}

/// 解析天选备选列表
fn parse_chosen_backups(value: Option<&Value>) -> Vec<ChosenBackupData> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    let hero_id = v
                        .get("hero_$key_id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let trait_id = v
                        .get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    if hero_id.is_empty() && trait_id.is_empty() {
                        return None;
                    }
                    Some(ChosenBackupData {
                        hero_id,
                        trait_id,
                        backup_type: v
                            .get("type")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================
// 通用辅助函数
// ============================================================

/// 从 JSON Value 提取字符串（兼容 String 和 Number）
fn parse_flexible_id_str(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// 从 JSON Value 提取 i32（兼容 String 和 Number）
fn parse_flexible_int(val: &Value) -> Option<i32> {
    match val {
        Value::Number(n) => n.as_i64().map(|i| i as i32),
        Value::String(s) => s.trim().parse::<i32>().ok(),
        _ => None,
    }
}

/// 拆分逗号分隔的 ID 列表
fn split_ids(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "0")
        .collect()
}

/// Value → String
fn value_str(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// 从阵容名提取【】中的羁绊标签
fn extract_bracket_traits(name: &str) -> Vec<String> {
    if let (Some(start), Some(end)) = (name.find('【'), name.find('】')) {
        if start < end {
            return name[start + 3..end] // skip 3 bytes for '【'
                .split([' ', '/', '、'])
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    vec![]
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LineupLoader;
    use std::path::PathBuf;

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    fn load_mode_items(mode: &str) -> Vec<LineupRawItem> {
        LineupLoader::load_cached_lineups(&test_config_root(), mode, "S18").unwrap()
    }

    #[test]
    fn parse_cards_mode17() {
        let raw = load_mode_items("17");
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        assert!(!cards.is_empty());
        for card in &cards {
            assert!(!card.id.is_empty());
            assert!(!card.name.is_empty());
        }
    }

    #[test]
    fn parse_shenyu_longwang() {
        let raw = load_mode_items("17");
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let card = cards.iter().find(|c| c.name.contains("神谕龙王"));
        assert!(card.is_some(), "未找到神谕龙王阵容");
        let card = card.unwrap();
        assert_eq!(card.quality, "S");
        assert!(!card.detail.final_heroes.is_empty(), "无最终英雄");
        assert!(!card.detail.equipment_order_ids.is_empty(), "无装备顺序");
    }

    #[test]
    fn parse_all_modes() {
        for mode in &["17", "16", "4"] {
            let raw = load_mode_items(mode);
            let cards = LineupAdapter::cards_from_raw_list(&raw, mode);
            assert!(!cards.is_empty(), "mode {} 阵容为空", mode);
        }
    }

    #[test]
    fn detail_fields_present() {
        let raw = load_mode_items("17");
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let sample = &cards[0].detail;
        // 必填字段非空
        assert!(!sample.final_heroes.is_empty());
    }
}
