// P7 知识回归样例
// 核心职责：
// - 从版本化 regression_cases 目录读取规则争议和即时动作样例
// - 为知识解析、装备冲突、海克斯刷新、版本修正和即时动作输出提供回放入口
// - 保持版本更新后的回归测试入口稳定

use std::fs;
use std::path::Path;

use hexsight_core::{HexError, HexResult};

/// P7 回归样例
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeRegressionCase {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub fixture: Option<String>,
    #[serde(default)]
    pub expected_actions: Vec<String>,
    #[serde(default)]
    pub expected_explanations: Vec<String>,
}

/// P7 回归样例文件
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeRegressionFile {
    #[serde(default)]
    cases: Vec<KnowledgeRegressionCase>,
}

/// P7 回归样例加载器
pub struct KnowledgeRegression;

impl KnowledgeRegression {
    pub fn load_cases(
        config_root: &Path,
        version: &str,
    ) -> HexResult<Vec<KnowledgeRegressionCase>> {
        let dir = config_root
            .join("rules")
            .join(version)
            .join("regression_cases");
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut cases = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|error| {
            HexError::Config(format!(
                "读取 P7 回归样例目录失败 {}: {}",
                dir.display(),
                error
            ))
        })? {
            let entry = entry.map_err(|error| {
                HexError::Config(format!(
                    "读取 P7 回归样例目录项失败 {}: {}",
                    dir.display(),
                    error
                ))
            })?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|error| {
                HexError::Config(format!(
                    "读取 P7 回归样例失败 {}: {}",
                    path.display(),
                    error
                ))
            })?;
            let parsed: KnowledgeRegressionFile =
                serde_json::from_str(&content).map_err(|error| {
                    HexError::Config(format!(
                        "解析 P7 回归样例失败 {}: {}",
                        path.display(),
                        error
                    ))
                })?;
            cases.extend(parsed.cases);
        }

        cases.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(cases)
    }
}
