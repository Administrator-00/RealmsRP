//! GM loader (M4.10) — 从文件系统加载 GM 文风模板

use std::path::Path;

use crate::error::{RealmsError, Result};

/// GM 模板 (一段 system_prompt .md 文件的内容)
#[derive(Debug, Clone)]
pub struct GmTemplate {
    /// GM id (文件名不含 .md)
    pub gm_id: String,
    /// .md 文件全文
    pub content: String,
}

/// 加载单个 GM 模板.
///
/// 查找顺序:
/// 1. `{gms_dir}/{world_id}/gms/{gm_id}.md` (world 专属 GM)
/// 2. `{gms_dir}/{gm_id}.md` (全局 GM)
pub fn load_gm(gms_dir: &Path, world_id: Option<&str>, gm_id: &str) -> Result<GmTemplate> {
    // 先查 world 专属目录
    if let Some(wid) = world_id {
        let world_gm_path = gms_dir.join(wid).join("gms").join(format!("{gm_id}.md"));
        if world_gm_path.exists() {
            let content = std::fs::read_to_string(&world_gm_path).map_err(RealmsError::Io)?;
            return Ok(GmTemplate {
                gm_id: gm_id.to_string(),
                content,
            });
        }
    }

    // 再查全局 gms/
    let global_path = gms_dir.join(format!("{gm_id}.md"));
    if global_path.exists() {
        let content = std::fs::read_to_string(&global_path).map_err(RealmsError::Io)?;
        return Ok(GmTemplate {
            gm_id: gm_id.to_string(),
            content,
        });
    }

    Err(RealmsError::not_found("gm", gm_id))
}

/// 列出所有可用 GM.
///
/// 返回 world 专属 + 全局 GM (world 专属优先, 同名时后者覆盖).
pub fn list_gms(gms_dir: &Path, world_id: Option<&str>) -> Result<Vec<GmTemplate>> {
    let mut gms = Vec::new();

    // 全局 GM
    if gms_dir.is_dir() {
        for entry in std::fs::read_dir(gms_dir).map_err(RealmsError::Io)? {
            let entry = entry.map_err(RealmsError::Io)?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                let gm_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let content = std::fs::read_to_string(&path).map_err(RealmsError::Io)?;
                gms.push(GmTemplate { gm_id, content });
            }
        }
    }

    // world 专属 GM
    if let Some(wid) = world_id {
        let world_gm_dir = gms_dir.join(wid).join("gms");
        if world_gm_dir.is_dir() {
            for entry in std::fs::read_dir(&world_gm_dir).map_err(RealmsError::Io)? {
                let entry = entry.map_err(RealmsError::Io)?;
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                    let gm_id = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let content = std::fs::read_to_string(&path).map_err(RealmsError::Io)?;
                    gms.push(GmTemplate { gm_id, content });
                }
            }
        }
    }

    Ok(gms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_nonexistent_gm_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(load_gm(dir.path(), None, "nonexistent").is_err());
    }

    #[test]
    fn load_global_gm() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("test_gm.md"), "# Test GM\n内容").unwrap();
        let gm = load_gm(dir.path(), None, "test_gm").unwrap();
        assert_eq!(gm.gm_id, "test_gm");
        assert!(gm.content.contains("# Test GM"));
    }
}
