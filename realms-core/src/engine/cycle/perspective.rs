//! Perspective manager (M5.2-M5.5) — 用户视角 CRUD + source=npc/oc 区分
//!
//! source='npc': 内置 NPC 视角, persona_data 锁定 (不允许编辑).
//! source='oc':  自定义角色, persona_data 自由编辑.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{RealmsError, Result};

/// 用户视角 (跨 world 复用)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    /// 视角 id (如 pov_npc_li_xiaoyao 或 pov_oc_shixiong)
    pub id: String,
    /// 来源: 'npc' (内置不可变) | 'oc' (自定义)
    pub source: String,
    /// 角色名
    pub name: String,
    /// 角色人设全文
    pub persona_data: String,
    /// 关联的 world id (source='npc' 时指向 npc_base 所在世界, oc 可为 None)
    pub world_id: Option<String>,
}

impl Perspective {
    /// 是否为内置 NPC 视角 (锁定不可改).
    pub fn is_npc(&self) -> bool {
        self.source == "npc"
    }

    /// 是否为自定义 OC 视角.
    pub fn is_oc(&self) -> bool {
        self.source == "oc"
    }

    /// 文件路径 (perspectives/{id}.json)
    fn file_path(dir: &Path, id: &str) -> std::path::PathBuf {
        dir.join(format!("{id}.json"))
    }
}

/// 加载视角 (从 perspectives/ 目录).
pub fn load_perspective(dir: &Path, id: &str) -> Result<Perspective> {
    let path = Perspective::file_path(dir, id);
    let json = std::fs::read_to_string(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => RealmsError::not_found("perspective", id),
        _ => RealmsError::Io(e),
    })?;
    serde_json::from_str(&json)
        .map_err(|e| RealmsError::invalid(format!("invalid perspective JSON: {e}")))
}

/// 保存视角 (创建或更新).
///
/// - source='npc' 时拒绝修改 persona_data (NPC 人设锁死).
/// - source='oc' 允许自由修改.
pub fn save_perspective(dir: &Path, p: &Perspective) -> Result<()> {
    // 验证 source
    if p.source != "npc" && p.source != "oc" {
        return Err(RealmsError::invalid(format!(
            "invalid source: {}, must be 'npc' or 'oc'",
            p.source
        )));
    }

    // source='npc': 检查是否有已存在的版本, 如果有则拒绝覆盖 persona_data
    if p.is_npc() {
        if let Ok(existing) = load_perspective(dir, &p.id) {
            if existing.persona_data != p.persona_data {
                return Err(RealmsError::invalid(format!(
                    "cannot modify persona_data for npc perspective '{}': npc persona is locked",
                    p.id
                )));
            }
        }
    }

    // source='oc': 自由保存
    std::fs::create_dir_all(dir)?;
    let path = Perspective::file_path(dir, &p.id);
    let json = serde_json::to_string_pretty(p).map_err(RealmsError::Serde)?;
    std::fs::write(&path, json).map_err(RealmsError::Io)?;
    Ok(())
}

/// 列出所有视角.
pub fn list_perspectives(dir: &Path) -> Result<Vec<Perspective>> {
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut perspectives = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(RealmsError::Io)? {
        let entry = entry.map_err(RealmsError::Io)?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let json = std::fs::read_to_string(&path).map_err(RealmsError::Io)?;
            if let Ok(p) = serde_json::from_str::<Perspective>(&json) {
                perspectives.push(p);
            }
        }
    }
    Ok(perspectives)
}

/// 删除视角.
pub fn delete_perspective(dir: &Path, id: &str) -> Result<()> {
    let path = Perspective::file_path(dir, id);
    if !path.exists() {
        return Err(RealmsError::not_found("perspective", id));
    }
    std::fs::remove_file(&path).map_err(RealmsError::Io)
}

/// 验证视角是否可在某 world 中使用.
///
/// - source='npc': world_id 必须匹配 (NPC 只属于其原生世界)
/// - source='oc': 无限制 (OC 可跨 world 复用)
pub fn validate_perspective_for_world(p: &Perspective, world_id: &str) -> Result<()> {
    if p.is_npc() {
        match &p.world_id {
            Some(wid) if wid == world_id => Ok(()),
            Some(wid) => Err(RealmsError::invalid(format!(
                "npc perspective '{}' belongs to world '{}', cannot use in '{}'",
                p.id, wid, world_id
            ))),
            None => Err(RealmsError::invalid(format!(
                "npc perspective '{}' has no world_id",
                p.id
            ))),
        }
    } else {
        // OC 可跨 world
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_npc() -> Perspective {
        Perspective {
            id: "pov_npc_li_xiaoyao".into(),
            source: "npc".into(),
            name: "李逍遥".into(),
            persona_data: "李逍遥 persona".into(),
            world_id: Some("xiatian_qixia_1".into()),
        }
    }

    fn sample_oc() -> Perspective {
        Perspective {
            id: "pov_oc_shixiong".into(),
            source: "oc".into(),
            name: "师兄".into(),
            persona_data: "OC persona".into(),
            world_id: None,
        }
    }

    #[test]
    fn save_and_load_oc() {
        let dir = tempfile::tempdir().expect("dir");
        let oc = sample_oc();
        save_perspective(dir.path(), &oc).expect("save");
        let loaded = load_perspective(dir.path(), "pov_oc_shixiong").expect("load");
        assert_eq!(loaded.name, "师兄");
        assert!(loaded.is_oc());
    }

    #[test]
    fn save_and_load_npc() {
        let dir = tempfile::tempdir().expect("dir");
        let npc = sample_npc();
        save_perspective(dir.path(), &npc).expect("save");
        let loaded = load_perspective(dir.path(), "pov_npc_li_xiaoyao").expect("load");
        assert_eq!(loaded.name, "李逍遥");
        assert!(loaded.is_npc());
    }

    #[test]
    fn npc_persona_is_locked() {
        let dir = tempfile::tempdir().expect("dir");
        let npc = sample_npc();
        save_perspective(dir.path(), &npc).expect("save");

        // 尝试修改 persona_data
        let mut modified = npc.clone();
        modified.persona_data = "modified persona".into();
        let err = save_perspective(dir.path(), &modified).unwrap_err();
        assert!(err.to_string().contains("locked"));
    }

    #[test]
    fn oc_persona_is_editable() {
        let dir = tempfile::tempdir().expect("dir");
        let oc = sample_oc();
        save_perspective(dir.path(), &oc).expect("save");

        // OC 可以修改
        let mut modified = oc.clone();
        modified.persona_data = "new persona".into();
        save_perspective(dir.path(), &modified).expect("re-save");

        let loaded = load_perspective(dir.path(), "pov_oc_shixiong").expect("load");
        assert_eq!(loaded.persona_data, "new persona");
    }

    #[test]
    fn validate_npc_for_world() {
        let npc = sample_npc();
        assert!(validate_perspective_for_world(&npc, "xiatian_qixia_1").is_ok());
        assert!(validate_perspective_for_world(&npc, "other_world").is_err());
    }

    #[test]
    fn validate_oc_for_any_world() {
        let oc = sample_oc();
        assert!(validate_perspective_for_world(&oc, "xiatian_qixia_1").is_ok());
        assert!(validate_perspective_for_world(&oc, "other_world").is_ok());
    }

    #[test]
    fn delete_and_list() {
        let dir = tempfile::tempdir().expect("dir");
        save_perspective(dir.path(), &sample_oc()).expect("save");
        assert_eq!(list_perspectives(dir.path()).unwrap().len(), 1);

        delete_perspective(dir.path(), "pov_oc_shixiong").expect("delete");
        assert!(list_perspectives(dir.path()).unwrap().is_empty());
        assert!(load_perspective(dir.path(), "pov_oc_shixiong").is_err());
    }

    #[test]
    fn invalid_source_rejected() {
        let dir = tempfile::tempdir().expect("dir");
        let bad = Perspective {
            id: "bad".into(),
            source: "invalid".into(),
            name: "x".into(),
            persona_data: "x".into(),
            world_id: None,
        };
        assert!(save_perspective(dir.path(), &bad).is_err());
    }
}
