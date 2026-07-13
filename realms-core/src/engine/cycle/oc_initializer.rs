//! OC initializer (M5.1) — OC 关系模板 + 批量初始化 npc_user_relationships
//!
//! 产品创新点: 创建 OC 时为每个内置 NPC 选关系模板, 不等 LLM 自创.
//!
//! 7 个预置模板 (3.md §1.7):
//! 萍水相逢 / 一见钟情 / 青梅竹马 / 宿敌 / 师徒 / 命中注定 / 无关系

use std::collections::HashMap;

use rusqlite::params;

use crate::db::pool::DbPool;
use crate::error::{RealmsError, Result};

/// OC 关系模板定义
#[derive(Debug, Clone)]
pub struct OcRelationshipTemplate {
    /// 模板 id (如 tpl_childhood_friend)
    pub template_id: &'static str,
    /// 中文名
    pub name: &'static str,
    /// 初始好感度 (-100..100, None = 不设置)
    pub affinity: Option<i32>,
    /// 初始信任 (0..100, None = 不设置)
    pub trust: Option<i32>,
    /// 初始亲密 (0..100, None = 不设置)
    pub intimacy: Option<i32>,
    /// 关系类型
    pub current_type: &'static str,
}

/// 7 个预置模板 (来自 3.md §1.7)
pub const OC_TEMPLATES: &[OcRelationshipTemplate] = &[
    OcRelationshipTemplate {
        template_id: "tpl_stranger",
        name: "萍水相逢",
        affinity: Some(0),
        trust: Some(0),
        intimacy: Some(0),
        current_type: "stranger",
    },
    OcRelationshipTemplate {
        template_id: "tpl_love_at_first_sight",
        name: "一见钟情",
        affinity: Some(70),
        trust: Some(30),
        intimacy: Some(50),
        current_type: "lover",
    },
    OcRelationshipTemplate {
        template_id: "tpl_childhood_friend",
        name: "青梅竹马",
        affinity: Some(60),
        trust: Some(80),
        intimacy: Some(70),
        current_type: "close_friend",
    },
    OcRelationshipTemplate {
        template_id: "tpl_sworn_enemy",
        name: "宿敌",
        affinity: Some(-80),
        trust: Some(10),
        intimacy: Some(0),
        current_type: "enemy",
    },
    OcRelationshipTemplate {
        template_id: "tpl_master_student",
        name: "师徒",
        affinity: Some(40),
        trust: Some(90),
        intimacy: Some(50),
        current_type: "master",
    },
    OcRelationshipTemplate {
        template_id: "tpl_destined",
        name: "命中注定",
        affinity: Some(50),
        trust: Some(50),
        intimacy: Some(50),
        current_type: "destined",
    },
    OcRelationshipTemplate {
        template_id: "tpl_no_relation",
        name: "无关系",
        affinity: None,
        trust: None,
        intimacy: None,
        current_type: "none",
    },
];

/// 用 OC 关系模板批量初始化 npc_user_relationships.
///
/// `selections`: npc_id → template_id 映射.
/// 对每个 NPC, 按模板写入 affinity/trust/intimacy/current_type + initial_template.
/// `tpl_no_relation` 模板的 NPC 不写入数值 (保留 NULL/默认值).
pub fn initialize_cycle_relationships(
    pool: &DbPool,
    cycle_id: &str,
    selections: &HashMap<String, String>,
) -> Result<()> {
    let conn = pool
        .get()
        .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;

    for (npc_id, tpl_id) in selections {
        let tpl = OC_TEMPLATES
            .iter()
            .find(|t| t.template_id == *tpl_id)
            .ok_or_else(|| RealmsError::not_found("template", tpl_id))?;

        if tpl_id == "tpl_no_relation" {
            // 无关系: INSERT 空行
            conn.execute(
                "INSERT OR IGNORE INTO npc_user_relationships (cycle_id, npc_id, current_type, initial_template)
                 VALUES (?1, ?2, 'none', 'tpl_no_relation')",
                params![cycle_id, npc_id],
            )
            .map_err(RealmsError::Database)?;
        } else {
            conn.execute(
                "INSERT OR REPLACE INTO npc_user_relationships
                 (cycle_id, npc_id, affinity, trust, intimacy, current_type, initial_template)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    cycle_id,
                    npc_id,
                    tpl.affinity,
                    tpl.trust,
                    tpl.intimacy,
                    tpl.current_type,
                    tpl.template_id,
                ],
            )
            .map_err(RealmsError::Database)?;
        }
    }

    Ok(())
}

/// 按 template_id 查找模板.
pub fn find_template(template_id: &str) -> Option<&'static OcRelationshipTemplate> {
    OC_TEMPLATES.iter().find(|t| t.template_id == template_id)
}

/// 列出所有可用模板 (供 WebUI 渲染).
pub fn list_templates() -> &'static [OcRelationshipTemplate] {
    OC_TEMPLATES
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    fn setup() -> DbPool {
        init_memory_pool().expect("pool")
    }

    #[test]
    fn all_seven_templates_exist() {
        let ids: Vec<&str> = OC_TEMPLATES.iter().map(|t| t.template_id).collect();
        let expected = [
            "tpl_stranger",
            "tpl_love_at_first_sight",
            "tpl_childhood_friend",
            "tpl_sworn_enemy",
            "tpl_master_student",
            "tpl_destined",
            "tpl_no_relation",
        ];
        for e in expected {
            assert!(ids.contains(&e), "缺少模板 {e}");
        }
    }

    #[test]
    fn initialize_relationships_populates_db() {
        let pool = setup();
        let mut selections = HashMap::new();
        selections.insert("lin_yueru".to_string(), "tpl_childhood_friend".to_string());
        selections.insert("li_xiaoyao".to_string(), "tpl_stranger".to_string());

        initialize_cycle_relationships(&pool, "c1", &selections).expect("init");

        let conn = pool.get().unwrap();
        let (aff, current_type, tpl): (i64, String, String) = conn
            .query_row(
                "SELECT affinity, current_type, initial_template
                 FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='lin_yueru'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(aff, 60);
        assert_eq!(current_type, "close_friend");
        assert_eq!(tpl, "tpl_childhood_friend");
    }

    #[test]
    fn no_relation_template_inserts_empty() {
        let pool = setup();
        let mut selections = HashMap::new();
        selections.insert("npc_x".to_string(), "tpl_no_relation".to_string());

        initialize_cycle_relationships(&pool, "c1", &selections).expect("init");

        let conn = pool.get().unwrap();
        let (aff, tpl): (Option<i64>, String) = conn
            .query_row(
                "SELECT affinity, initial_template
                 FROM npc_user_relationships WHERE cycle_id='c1' AND npc_id='npc_x'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(aff, Some(0), "无关系模板 affinity 为默认值 0");
        assert_eq!(tpl, "tpl_no_relation");
    }

    #[test]
    fn find_template_roundtrip() {
        for t in OC_TEMPLATES {
            let found = find_template(t.template_id).unwrap();
            assert_eq!(found.name, t.name);
        }
        assert!(find_template("nonexistent").is_none());
    }
}
