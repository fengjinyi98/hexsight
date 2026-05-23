// 开局路线分类器
// 核心职责：
// - 判断连胜/连败/混合开局路线
// - 评估前期战力、二星数量、前排质量、装备状态
// - 输出推荐开局策略

use hexsight_core::{OpeningRoute, OpeningRouteResult};

/// 开局路线分类器
pub struct OpeningRouteClassifier;

impl OpeningRouteClassifier {
    /// 根据早期对局状态分类开局路线
    pub fn classify(
        _hero_count: i32,
        two_star_count: i32,
        has_frontline_tank: bool,
        can_build_combat_item: bool,
        current_hp: i32,
        _current_gold: i32,
        current_streak: i32,
        _round: &str,
    ) -> OpeningRouteResult {
        // 二星数量是最直接的战力指标
        let mut reasons = Vec::new();
        let mut recommended_actions = Vec::new();

        // 前排质量（简化：有坦克型前排 + 二星）
        let frontline_quality = if has_frontline_tank && two_star_count >= 1 {
            70
        } else if has_frontline_tank || two_star_count >= 1 {
            45
        } else {
            20
        };

        // 判断开局路线
        let (route, confidence) = if two_star_count >= 3 && can_build_combat_item {
            reasons.push(format!("已有 {} 个二星，装备可合成战力装", two_star_count));
            recommended_actions.push("合通用战力装保连胜".into());
            recommended_actions.push("提前升人口扩大优势".into());
            (OpeningRoute::WinStreak, 0.78)
        } else if two_star_count >= 2 && has_frontline_tank {
            reasons.push(format!("{} 二星 + 前排，有一定战力", two_star_count));
            recommended_actions.push("评估是否能匹配前几家强度".into());
            recommended_actions.push("能赢就保连胜，不能就保经济".into());
            (OpeningRoute::Mixed, 0.65)
        } else if two_star_count <= 1 && !can_build_combat_item && current_hp >= 85 {
            reasons.push("二星少，装备暂时无法合成战力装".into());
            reasons.push("血量安全，适合精致连败".into());
            recommended_actions.push("控强度保连败".into());
            recommended_actions.push("保利息优先".into());
            recommended_actions.push("选秀抢关键装备".into());
            (OpeningRoute::LossStreak, 0.72)
        } else if current_hp < 70 {
            reasons.push("血量偏低，不适合连败".into());
            recommended_actions.push("立即补战力，避免继续掉血".into());
            recommended_actions.push("合当前能用的装备".into());
            (OpeningRoute::Mixed, 0.60)
        } else {
            reasons.push("开局状态中等，方向未定".into());
            recommended_actions.push("保血量 + 保利息".into());
            recommended_actions.push("等待装备/海克斯确认方向".into());
            (OpeningRoute::Mixed, 0.55)
        };

        // 连胜/连败修正
        if current_streak >= 2 && route == OpeningRoute::WinStreak {
            reasons.push(format!("已有 {} 连胜，继续保连胜", current_streak));
        }
        if current_streak <= -2 && route == OpeningRoute::LossStreak {
            reasons.push(format!("已有 {} 连败，继续控败", current_streak.abs()));
        }

        OpeningRouteResult {
            route,
            confidence,
            reasons,
            two_star_count,
            frontline_quality,
            can_build_combat_item,
            recommended_actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_strong_opening() {
        let result = OpeningRouteClassifier::classify(
            5, 3, true, true, 100, 15, 0, "2-1",
        );
        assert_eq!(result.route, OpeningRoute::WinStreak);
        assert!(result.confidence > 0.7);
    }

    #[test]
    fn classify_weak_opening() {
        let result = OpeningRouteClassifier::classify(
            3, 0, false, false, 100, 10, 0, "2-1",
        );
        assert_eq!(result.route, OpeningRoute::LossStreak);
        assert!(result.confidence > 0.6);
    }

    #[test]
    fn classify_low_hp() {
        let result = OpeningRouteClassifier::classify(
            4, 1, true, false, 60, 20, 0, "3-1",
        );
        // 血量低时不推荐连败
        assert_ne!(result.route, OpeningRoute::LossStreak);
    }
}
