// LLM 上下文生成器
// 核心职责：
// - 将 RulesContextOutput 序列化为 LLM 可消费的精简上下文 JSON
// - 移除 UI 渲染细节，只保留语义信息
// - 输出格式对齐目标文档第七章 JSON Schema

use hexsight_core::RulesContextOutput;

/// LLM 上下文构建器
pub struct LlmContextBuilder;

impl LlmContextBuilder {
    /// 从规则上下文构建 LLM 输入 JSON
    /// 输出精简的阵容摘要，适合作为 LLM prompt 的一部分
    pub fn build(context: &RulesContextOutput) -> String {
        let hero_summaries: Vec<String> = context
            .final_heroes
            .iter()
            .map(|h| {
                let carry_tag = if h.is_carry { "[C]" } else { "" };
                let equip_str = if h.equipment_names.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", h.equipment_names.join(", "))
                };
                format!(
                    "{} {}{} cost={} pos={}{}",
                    h.id, h.name, carry_tag, h.cost, h.position, equip_str
                )
            })
            .collect();

        let trait_summaries: Vec<String> = context
            .traits
            .iter()
            .map(|t| format!("{}({}) x{}", t.name, t.trait_type, t.count))
            .collect();

        let hex_summaries: Vec<String> = context
            .augments
            .recommended
            .iter()
            .chain(context.augments.replacement.iter())
            .map(|h| format!("{} (Lv.{})", h.name, h.level))
            .collect();

        let equip_order: Vec<String> = context
            .equipment
            .order
            .iter()
            .map(|e| e.name.clone())
            .collect();

        let strategy_parts: Vec<String> = context
            .strategy_texts
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect();

        let mut lines = Vec::new();
        lines.push(format!(
            "模式: {} (S{})",
            context.mode.name, context.mode.season
        ));
        lines.push(format!(
            "阵容: {} by {}",
            context.lineup.name, context.lineup.author
        ));
        lines.push(format!(
            "评级: {} 标签: {:?}",
            context.lineup.quality, context.lineup.tags
        ));
        lines.push(format!("英雄: {}", hero_summaries.join(" | ")));
        lines.push(format!("羁绊: {}", trait_summaries.join(", ")));

        if !hex_summaries.is_empty() {
            lines.push(format!("符文: {}", hex_summaries.join(", ")));
        }
        if !equip_order.is_empty() {
            lines.push(format!("装备优先级: {}", equip_order.join(" → ")));
        }
        if !strategy_parts.is_empty() {
            lines.push(format!("策略: {}", strategy_parts.join("; ")));
        }

        lines.join("\n")
    }

    /// 构建紧凑版 JSON（适合作为 API 参数）
    pub fn build_compact_json(context: &RulesContextOutput) -> serde_json::Value {
        serde_json::json!({
            "mode": context.mode.name,
            "season": context.mode.season,
            "lineup": context.lineup.name,
            "author": context.lineup.author,
            "quality": context.lineup.quality,
            "heroes": context.final_heroes.iter().map(|h| {
                serde_json::json!({
                    "name": h.name,
                    "cost": h.cost,
                    "carry": h.is_carry,
                    "position": h.position,
                    "equipment": h.equipment_names,
                })
            }).collect::<Vec<_>>(),
            "traits": context.traits.iter().map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "count": t.count,
                    "type": t.trait_type,
                })
            }).collect::<Vec<_>>(),
            "hexes": context.augments.recommended.iter().map(|h| h.name.clone()).collect::<Vec<_>>(),
            "strategy_hints": context.strategy_texts.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameDataIndex, LineupAdapter, LineupLoader, RulesContextBuilder};
    use std::path::PathBuf;

    fn test_config_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("config")
    }

    #[test]
    fn build_llm_context_content() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let ctx = RulesContextBuilder::build(&cards[0], "17", Some(&index));

        let text = LlmContextBuilder::build(&ctx);
        assert!(text.contains("模式:"));
        assert!(text.contains("阵容:"));
        assert!(text.contains("英雄:"));
        assert!(text.contains("羁绊:"));
    }

    #[test]
    fn build_compact_json_valid() {
        let index = GameDataIndex::load(&test_config_root(), "17").unwrap();
        let raw = LineupLoader::load_cached_lineups(&test_config_root(), "17", "S18").unwrap();
        let cards = LineupAdapter::cards_from_raw_list(&raw, "17");
        let ctx = RulesContextBuilder::build(&cards[0], "17", Some(&index));

        let json = LlmContextBuilder::build_compact_json(&ctx);
        assert!(json["lineup"].is_string());
        assert!(json["heroes"].is_array());
        assert!(json["traits"].is_array());
    }
}
