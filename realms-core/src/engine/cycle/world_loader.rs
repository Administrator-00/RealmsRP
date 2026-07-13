//! World loader (M4.9) — 从文件系统加载 World 数据

use std::collections::HashMap;
use std::path::Path;

use crate::error::{RealmsError, Result};

/// 加载到内存的 World 数据
#[derive(Debug, Clone)]
pub struct WorldData {
    /// world id (目录名)
    pub world_id: String,
    /// setting.md 全文
    pub setting: String,
    /// NPC 人设 (npc_id → persona 文本)
    pub npc_personas: HashMap<String, String>,
}

/// 加载 world 目录下的所有数据.
///
/// 读取: `{worlds_dir}/{world_id}/setting.md` + `npcs/*.md`
/// map.json / rules.json / lorebook.json 暂未读取 (后续增量支持).
pub fn load_world(worlds_dir: &Path, world_id: &str) -> Result<WorldData> {
    let world_dir = worlds_dir.join(world_id);
    if !world_dir.is_dir() {
        return Err(RealmsError::not_found("world", world_id));
    }

    let setting_path = world_dir.join("setting.md");
    let setting = std::fs::read_to_string(&setting_path).map_err(RealmsError::Io)?;

    let npcs_dir = world_dir.join("npcs");
    let mut npc_personas = HashMap::new();
    if npcs_dir.is_dir() {
        for entry in std::fs::read_dir(&npcs_dir).map_err(RealmsError::Io)? {
            let entry = entry.map_err(RealmsError::Io)?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let npc_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let persona = std::fs::read_to_string(&path).map_err(RealmsError::Io)?;
                npc_personas.insert(npc_id, persona);
            }
        }
    }

    Ok(WorldData {
        world_id: world_id.to_string(),
        setting,
        npc_personas,
    })
}

/// 列出 worlds_dir 下所有 world id.
pub fn list_worlds(worlds_dir: &Path) -> Result<Vec<String>> {
    if !worlds_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut ids = Vec::new();
    for entry in std::fs::read_dir(worlds_dir).map_err(RealmsError::Io)? {
        let entry = entry.map_err(RealmsError::Io)?;
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            if let Some(name) = entry.file_name().to_str() {
                ids.push(name.to_string());
            }
        }
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_worlds_finds_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(dir.path().join("my_world")).unwrap();
        let ids = list_worlds(dir.path()).unwrap();
        assert!(ids.contains(&"my_world".to_string()));
    }

    #[test]
    fn load_nonexistent_world_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(load_world(dir.path(), "nonexistent").is_err());
    }
}
