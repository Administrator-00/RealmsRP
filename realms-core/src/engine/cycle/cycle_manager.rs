//! Cycle manager (M4.8) — 周目生命周期管理
//!
//! 串接 World loader / GM loader / DB queries,
//! 提供创建、加载、删除周目的高层 API.

use std::path::Path;

use crate::db::pool::DbPool;
use crate::db::queries::cycle::{self, Cycle};
use crate::engine::cycle::gm_loader::{self, GmTemplate};
use crate::engine::cycle::world_loader::{self, WorldData};
use crate::error::{RealmsError, Result};

/// 创建新周目上下文
pub struct NewCycleContext {
    /// world 数据目录 (worlds/)
    pub worlds_dir: Box<Path>,
    /// GM 数据目录 (gms/)
    pub gms_dir: Box<Path>,
    /// DB 连接池
    pub pool: DbPool,
}

/// 周目运行时上下文 (World + GM + Cycle)
pub struct CycleContext {
    /// 周目元数据
    pub cycle: Cycle,
    /// 世界观数据
    pub world: WorldData,
    /// GM 文风模板
    pub gm: GmTemplate,
}

impl NewCycleContext {
    /// 创建新的上下文.
    pub fn new(
        worlds_dir: impl Into<Box<Path>>,
        gms_dir: impl Into<Box<Path>>,
        pool: DbPool,
    ) -> Self {
        Self {
            worlds_dir: worlds_dir.into(),
            gms_dir: gms_dir.into(),
            pool,
        }
    }

    /// 创建新周目:
    /// 1. 加载 World (setting.md + npcs/)
    /// 2. 加载 GM 模板
    /// 3. 写 cycles 行
    /// 4. 初始化 character_states (为每个 NPC 创建默认行)
    pub fn start_new_cycle(
        &self,
        cycle_id: &str,
        world_id: &str,
        perspective_id: &str,
        gm_id: &str,
        cycle_name: &str,
    ) -> Result<CycleContext> {
        // 1. 加载 World
        let world = world_loader::load_world(&self.worlds_dir, world_id)?;

        // 2. 加载 GM
        let gm = gm_loader::load_gm(&self.gms_dir, Some(world_id), gm_id)?;

        // 3. 创建 cycle 行
        let cycle = cycle::create_cycle(
            &self.pool,
            cycle_id,
            world_id,
            perspective_id,
            gm_id,
            cycle_name,
        )?;

        // 4. 初始化 character_states: 为每个 NPC INSERT 默认行
        let conn = self
            .pool
            .get()
            .map_err(|e| RealmsError::internal(format!("pool: {e}")))?;
        for npc_id in world.npc_personas.keys() {
            conn.execute(
                "INSERT OR IGNORE INTO character_states (cycle_id, npc_id) VALUES (?1, ?2)",
                rusqlite::params![cycle_id, npc_id],
            )
            .map_err(RealmsError::Database)?;
        }

        Ok(CycleContext { cycle, world, gm })
    }

    /// 加载已有周目的运行时上下文.
    pub fn load_cycle_context(&self, cycle_id: &str) -> Result<CycleContext> {
        let cycle = cycle::load_cycle(&self.pool, cycle_id)?;
        let world = world_loader::load_world(&self.worlds_dir, &cycle.world_id)?;
        let gm = gm_loader::load_gm(&self.gms_dir, Some(&cycle.world_id), &cycle.gm_id)?;
        Ok(CycleContext { cycle, world, gm })
    }

    /// 列出某 world 下所有周目.
    pub fn list_world_cycles(&self, world_id: &str) -> Result<Vec<Cycle>> {
        cycle::list_cycles(&self.pool, world_id)
    }

    /// 删除周目 (级联).
    pub fn delete_cycle(&self, cycle_id: &str) -> Result<()> {
        cycle::delete_cycle(&self.pool, cycle_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::init_memory_pool;

    fn setup() -> (NewCycleContext, tempfile::TempDir, tempfile::TempDir) {
        let worlds_dir = tempfile::tempdir().expect("worlds_dir");
        let gms_dir = tempfile::tempdir().expect("gms_dir");
        let pool = init_memory_pool().expect("pool");

        // 创建 world 数据
        let w = worlds_dir.path().join("test_world");
        std::fs::create_dir_all(w.join("npcs")).unwrap();
        std::fs::write(w.join("setting.md"), "# 测试世界\n设定正文").unwrap();
        std::fs::write(w.join("npcs").join("npc_a.md"), "NPC A persona").unwrap();
        std::fs::write(w.join("npcs").join("npc_b.md"), "NPC B persona").unwrap();

        // 创建 GM 模板
        std::fs::write(gms_dir.path().join("test_gm.md"), "# Test GM").unwrap();

        let ctx = NewCycleContext::new(
            worlds_dir.path().to_path_buf(),
            gms_dir.path().to_path_buf(),
            pool,
        );
        (ctx, worlds_dir, gms_dir)
    }

    #[test]
    fn start_new_cycle_initializes_npcs() {
        let (ctx, _wd, _gd) = setup();
        let result = ctx
            .start_new_cycle("c1", "test_world", "pov_test", "test_gm", "周目1")
            .expect("start");

        assert_eq!(result.cycle.cycle_id, "c1");
        assert!(result.world.setting.contains("测试世界"));
        assert_eq!(result.gm.gm_id, "test_gm");

        // 验证 character_states 初始化
        let conn = ctx.pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM character_states WHERE cycle_id='c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2, "应为 2 个 NPC 创建了初始行");
    }

    #[test]
    fn load_cycle_context_after_create() {
        let (ctx, _wd, _gd) = setup();
        ctx.start_new_cycle("c1", "test_world", "p", "test_gm", "周目")
            .expect("start");

        let loaded = ctx.load_cycle_context("c1").expect("load");
        assert_eq!(loaded.cycle.cycle_name, "周目");
        assert!(loaded.world.setting.contains("测试世界"));
    }
}
