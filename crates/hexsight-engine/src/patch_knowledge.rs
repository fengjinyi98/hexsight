// 版本公告知识加载器
// 核心职责：
// - 从规则目录加载结构化版本公告与人工覆写条目
// - 合并 patch_knowledge.json 与 patch_knowledge_overrides.json 中的评分修正
// - 为版本修正器提供稳定的 PatchEntry 列表

use std::fs;
use std::path::Path;

use hexsight_core::{HexError, HexResult, PatchEntry};
use serde::{Deserialize, Serialize};

/// 版本公告知识集合
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatchKnowledge {
    pub version: String,
    #[serde(default)]
    pub entries: Vec<PatchEntry>,
}

#[derive(Debug, Deserialize)]
struct PatchKnowledgeFile {
    #[serde(default)]
    version: String,
    #[serde(default)]
    entries: Vec<PatchEntry>,
}

/// 版本公告知识加载器
pub struct PatchKnowledgeLoader;

impl PatchKnowledgeLoader {
    /// 加载指定版本的公告知识和人工覆写
    pub fn load(config_root: &Path, version: &str) -> HexResult<PatchKnowledge> {
        let rules_dir = config_root.join("rules").join(version);
        let mut knowledge = PatchKnowledge {
            version: version.to_string(),
            entries: Vec::new(),
        };

        Self::append_entries(&mut knowledge, &rules_dir.join("patch_knowledge.json"))?;
        Self::append_entries(
            &mut knowledge,
            &rules_dir.join("patch_knowledge_overrides.json"),
        )?;

        Ok(knowledge)
    }

    fn append_entries(knowledge: &mut PatchKnowledge, path: &Path) -> HexResult<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path).map_err(|e| {
            HexError::Config(format!("读取版本公告知识失败 {}: {}", path.display(), e))
        })?;

        let parsed: PatchKnowledgeFile = serde_json::from_str(&content).map_err(|e| {
            HexError::Config(format!("解析版本公告知识失败 {}: {}", path.display(), e))
        })?;

        if !parsed.version.is_empty() && knowledge.version.is_empty() {
            knowledge.version = parsed.version;
        }
        knowledge.entries.extend(parsed.entries);
        Ok(())
    }
}
