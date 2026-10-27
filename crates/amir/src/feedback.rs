/// Validation Feedback — 将验证错误转换为自然语言
///
/// 将 Omcom Validator 产生的结构化错误翻译为用户友好的消息。

/// 翻译单个验证错误为自然语言
pub fn translate_error(error: &str, _context: &str) -> String {
    let lower = error.to_lowercase();

    if lower.contains("not found") {
        extract_subject(error, "not found")
            .map(|subject| format!("找不到 {}", subject))
            .unwrap_or_else(|| format!("无法找到指定的对象或资源: {}", error))
    } else if lower.contains("empty") && lower.contains("name") {
        "对象名称不能为空，请指定一个名称。".to_string()
    } else if lower.contains("invalid") && lower.contains("type") {
        "不支持的对象类型，请检查类型名称是否正确。支持的包括：墙、梁、柱、板、门、窗等。".to_string()
    } else if lower.contains("unknown action") {
        "不支持的操作类型。".to_string()
    } else if lower.contains("parent") && lower.contains("not found") {
        "找不到指定的父对象，请检查父对象是否存在。".to_string()
    } else if lower.contains("child") && lower.contains("not found") {
        "找不到指定的子对象，请检查子对象是否存在。".to_string()
    } else if lower.contains("material") && lower.contains("not found") {
        "找不到指定的材料，请先创建该材料。".to_string()
    } else if lower.contains("lifecycle record") && lower.contains("not found") {
        "找不到指定的生命周期记录。".to_string()
    } else if lower.contains("asset") && lower.contains("not found") {
        "找不到指定的资产对象。".to_string()
    } else if lower.contains("spatial conflict") || lower.contains("intersect") {
        "新对象与现有对象存在空间冲突，但工程中重叠是常见的（如管道穿墙），已记录为警告。".to_string()
    } else if lower.contains("permission") || lower.contains("denied") || lower.contains("forbidden") {
        "没有权限执行此操作。".to_string()
    } else if lower.contains("transaction") && lower.contains("already") {
        "事务已在进行中，无法开始新事务。".to_string()
    } else if lower.contains("transaction") && lower.contains("no active") {
        "没有活跃的事务可提交或回滚。".to_string()
    } else if lower.contains("connection") && lower.contains("not found") {
        "找不到指定的连接关系。".to_string()
    } else if lower.contains("instruction id") {
        "指令 ID 不能为空。".to_string()
    } else if lower.contains("duplicate") || lower.contains("already exists") {
        "对象已存在，请使用不同的名称或 ID。".to_string()
    } else if lower.contains("validation") {
        "验证失败。".to_string() + &error
    } else {
        format!("操作失败: {}", error)
    }
}

/// 提取错误消息中的主体（如 "target object 'abc-123' not found" → "abc-123"）
fn extract_subject<'a>(error: &'a str, keyword: &str) -> Option<String> {
    let lower = error.to_lowercase();
    let kw_lower = keyword.to_lowercase();
    if let Some(pos) = lower.find(&kw_lower) {
        let before = &error[..pos];
        // Find the first quoted substring before the keyword
        if let Some(open) = before.find('\'') {
            if let Some(close) = before[open + 1..].find('\'') {
                return Some(before[open + 1..open + 1 + close].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_not_found() {
        let result = translate_error("target object 'abc-123' not found", "");
        assert!(result.contains("abc-123"));
        assert!(result.contains("找不到"));
    }

    #[test]
    fn test_translate_empty_name() {
        let result = translate_error("object name is empty", "");
        assert!(!result.is_empty());
        assert!(result.contains("名称不能为空"));
    }

    #[test]
    fn test_translate_invalid_type() {
        let result = translate_error("invalid object type: 999", "");
        assert!(result.contains("不支持的对象类型"));
    }

    #[test]
    fn test_translate_spatial_conflict() {
        let result = translate_error("[WARN] spatial conflict with existing object", "");
        assert!(result.contains("空间冲突"));
    }

    #[test]
    fn test_translate_generic() {
        let result = translate_error("some unknown error happened", "");
        assert!(result.contains("操作失败"));
    }
}
