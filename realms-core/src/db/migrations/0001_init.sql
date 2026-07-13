-- =============================================================================
-- Realms 完整 Schema v3
-- 基于 3.md §2.2, 适配 SQLite (FTS5 / JSON1 / WAL / foreign_keys)
-- =============================================================================

-- A. 元数据 (schema version tracking)
CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT
);
INSERT OR IGNORE INTO meta(key, value) VALUES ('schema_version', '3');
INSERT OR IGNORE INTO meta(key, value) VALUES ('product_version', '1.0');

-- =============================================================================
-- B. 周目 (Cycle)
-- =============================================================================
CREATE TABLE IF NOT EXISTS cycles (
    cycle_id              TEXT PRIMARY KEY,
    world_id              TEXT NOT NULL,
    perspective_id        TEXT NOT NULL,
    gm_id                 TEXT NOT NULL,
    cycle_name            TEXT NOT NULL,
    cycle_description     TEXT,
    current_world_time    TEXT,
    current_location_id   TEXT,
    total_events          INTEGER DEFAULT 0,
    total_playtime_seconds INTEGER DEFAULT 0,
    created_at            TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at            TEXT NOT NULL DEFAULT (datetime('now')),
    last_played_at        TEXT
);
CREATE INDEX IF NOT EXISTS idx_cycles_world   ON cycles(world_id);
CREATE INDEX IF NOT EXISTS idx_cycles_updated ON cycles(updated_at DESC);

-- =============================================================================
-- C. 内置 NPC 当前状态 (以 cycle 隔离)
-- =============================================================================
CREATE TABLE IF NOT EXISTS character_states (
    cycle_id      TEXT NOT NULL,
    npc_id        TEXT NOT NULL,
    arcs          TEXT NOT NULL DEFAULT '{}',       -- JSON: {trust_user:50, courage:70, ...}
    hp            INTEGER DEFAULT 100,
    hp_max        INTEGER DEFAULT 100,
    mp            INTEGER DEFAULT 50,
    mp_max        INTEGER DEFAULT 50,
    location_id   TEXT,
    status_flags  TEXT NOT NULL DEFAULT '{}',       -- JSON: {poisoned:true, ...}
    arc_phase     TEXT,
    perceived_mood TEXT NOT NULL DEFAULT 'neutral',
    last_event_id INTEGER,
    updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (cycle_id, npc_id)
);

-- =============================================================================
-- D. NPC↔User 关系 (以 cycle 隔离)
-- =============================================================================
CREATE TABLE IF NOT EXISTS npc_user_relationships (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id                TEXT NOT NULL,
    npc_id                  TEXT NOT NULL,
    affinity                INTEGER DEFAULT 0,      -- -100..100
    trust                   INTEGER DEFAULT 0,       -- 0..100
    intimacy                INTEGER DEFAULT 0,       -- 0..100
    respect                 INTEGER DEFAULT 0,       -- 0..100
    current_type            TEXT,                    -- stranger/friend/lover/enemy/...
    initial_template        TEXT,                    -- tpl_childhood_friend etc.
    initial_setup_event_id  INTEGER,
    first_met_at            TEXT,
    last_changed_event_id   INTEGER,
    updated_at              TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(cycle_id, npc_id)
);
CREATE INDEX IF NOT EXISTS idx_rels_npc ON npc_user_relationships(cycle_id, npc_id);

-- 关系变更历史
CREATE TABLE IF NOT EXISTS relationship_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id    TEXT NOT NULL,
    npc_id      TEXT NOT NULL,
    field       TEXT NOT NULL,                       -- affinity/trust/intimacy/respect/type
    old_value   TEXT,
    new_value   TEXT,
    delta       INTEGER,
    event_id    INTEGER,
    world_time  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_rel_hist ON relationship_history(cycle_id, npc_id, created_at DESC);

-- =============================================================================
-- E. 事件溯源 (单一真相源)
-- =============================================================================
CREATE TABLE IF NOT EXISTS domain_events (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id      TEXT NOT NULL,
    event_type    TEXT NOT NULL,                     -- setup/arc_increment/relationship_change/...
    actor_id      TEXT,
    actor_type    TEXT,                              -- npc/user/system
    target_id     TEXT,
    target_type   TEXT,                              -- npc/user/item/location
    location_id   TEXT,
    arc_dimension TEXT,
    delta         REAL,
    world_time    TEXT,
    importance    REAL DEFAULT 0.5,
    summary       TEXT,
    payload       TEXT DEFAULT '{}',                 -- JSON
    is_milestone  INTEGER DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_events_cycle_time   ON domain_events(cycle_id, world_time DESC);
CREATE INDEX IF NOT EXISTS idx_events_cycle_actor  ON domain_events(cycle_id, actor_id);
CREATE INDEX IF NOT EXISTS idx_events_cycle_target ON domain_events(cycle_id, target_id);
CREATE INDEX IF NOT EXISTS idx_events_cycle_type   ON domain_events(cycle_id, event_type);
CREATE INDEX IF NOT EXISTS idx_events_milestone    ON domain_events(cycle_id, is_milestone) WHERE is_milestone = 1;

-- =============================================================================
-- F. 三层记忆
-- =============================================================================
-- F.1 情景记忆
CREATE TABLE IF NOT EXISTS episodic_memories (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id          TEXT NOT NULL,
    character_id      TEXT NOT NULL,
    event_id          INTEGER,
    content           TEXT NOT NULL,
    emotional_valence REAL DEFAULT 0.0,              -- -1..1
    vividness         REAL DEFAULT 1.0,              -- 0..1, 衰减
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    last_accessed_at  TEXT,
    access_count      INTEGER DEFAULT 0
);

-- F.2 语义记忆
CREATE TABLE IF NOT EXISTS semantic_memories (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id                TEXT NOT NULL,
    character_id            TEXT,                    -- NULL = 公共知识
    category                TEXT NOT NULL,           -- fact/rule/person/place/lore/rumor
    subject                 TEXT,
    predicate               TEXT,
    object_text             TEXT,
    content                 TEXT NOT NULL,
    confidence              REAL DEFAULT 1.0,
    source_event_id         INTEGER,
    contradicting_event_ids TEXT DEFAULT '[]',       -- JSON array
    created_at              TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at              TEXT
);

-- F.3 情感记忆
CREATE TABLE IF NOT EXISTS emotional_memories (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id           TEXT NOT NULL,
    character_id       TEXT NOT NULL,
    target_id          TEXT,
    emotion_type       TEXT NOT NULL,                -- love/hate/fear/respect/gratitude/guilt/longing
    intensity          REAL DEFAULT 0.5,             -- 0..1
    context            TEXT,
    trigger_event_id   INTEGER,
    created_at         TEXT NOT NULL DEFAULT (datetime('now')),
    last_reinforced_at TEXT,
    reinforcement_count INTEGER DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_emo_target ON emotional_memories(cycle_id, character_id, target_id);

-- =============================================================================
-- G. 物品栏 (cycle 隔离)
-- =============================================================================
CREATE TABLE IF NOT EXISTS items (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id         TEXT NOT NULL,
    owner_id         TEXT,                           -- npc_id / user_perspective_id / NULL(公共)
    item_name        TEXT NOT NULL,
    item_type        TEXT,                           -- weapon/potion/key/armor/book/treasure/food
    quantity         INTEGER DEFAULT 1,
    properties       TEXT DEFAULT '{}',              -- JSON
    acquired_event_id INTEGER,
    acquired_at      TEXT,
    acquired_from    TEXT
);
CREATE INDEX IF NOT EXISTS idx_items_owner ON items(cycle_id, owner_id);

-- =============================================================================
-- H. 地点 (cycle 隔离, 已发现)
-- =============================================================================
CREATE TABLE IF NOT EXISTS locations (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    cycle_id         TEXT NOT NULL,
    location_name    TEXT NOT NULL,
    parent_id        INTEGER REFERENCES locations(id),
    location_type    TEXT,                           -- realm/city/town/dungeon/wilderness/building/room
    description      TEXT,
    properties       TEXT DEFAULT '{}',              -- JSON
    discovered_event_id INTEGER,
    discovered_at    TEXT
);

-- =============================================================================
-- I. 弧光维度定义 (全局)
-- =============================================================================
CREATE TABLE IF NOT EXISTS arc_dimensions (
    dimension_id   TEXT PRIMARY KEY,
    dimension_name TEXT NOT NULL,
    category       TEXT NOT NULL,                    -- user_relationship/personality/skill/moral
    min_value      REAL DEFAULT 0,
    max_value      REAL DEFAULT 100,
    milestones     TEXT NOT NULL DEFAULT '[]',       -- JSON array: [{"at":0,"label":"..."}, ...]
    description    TEXT,
    world_id       TEXT                              -- NULL = 全局通用
);

INSERT OR IGNORE INTO arc_dimensions(dimension_id, dimension_name, category, milestones) VALUES
('trust_user',     '对 user 的信任', 'user_relationship',
 '[{"at":0,"label":"陌生人"},{"at":20,"label":"初识"},{"at":50,"label":"朋友"},{"at":80,"label":"生死之交"},{"at":100,"label":"以命相托"}]'),
('affection_user', '对 user 的爱慕', 'user_relationship',
 '[{"at":0,"label":"无感"},{"at":30,"label":"萌芽"},{"at":60,"label":"暗恋"},{"at":90,"label":"表白"}]'),
('hostility_user', '对 user 的敌意', 'user_relationship',
 '[{"at":0,"label":"无"},{"at":40,"label":"不满"},{"at":70,"label":"厌恶"},{"at":100,"label":"死敌"}]'),
('courage',        '勇气',           'personality',
 '[{"at":0,"label":"怯懦"},{"at":30,"label":"平常"},{"at":60,"label":"勇敢"},{"at":100,"label":"无畏"}]'),
('maturity',       '成熟度',         'personality',
 '[{"at":0,"label":"天真"},{"at":30,"label":"成长中"},{"at":60,"label":"成熟"},{"at":100,"label":"通透"}]');

-- =============================================================================
-- J. 全文检索 FTS5
-- =============================================================================
-- domain_events FTS
CREATE VIRTUAL TABLE IF NOT EXISTS events_fts USING fts5(
    summary,
    content='domain_events',
    content_rowid='id',
    tokenize='unicode61'
);

-- episodic_memories FTS
CREATE VIRTUAL TABLE IF NOT EXISTS episodic_fts USING fts5(
    content,
    content='episodic_memories',
    content_rowid='id',
    tokenize='unicode61'
);

-- semantic_memories FTS
CREATE VIRTUAL TABLE IF NOT EXISTS semantic_fts USING fts5(
    content,
    content='semantic_memories',
    content_rowid='id',
    tokenize='unicode61'
);

-- =============================================================================
-- K. FTS5 同步触发器
-- =============================================================================
-- events_fts
CREATE TRIGGER IF NOT EXISTS tr_events_ai AFTER INSERT ON domain_events BEGIN
    INSERT INTO events_fts(rowid, summary) VALUES (new.id, new.summary);
END;
CREATE TRIGGER IF NOT EXISTS tr_events_ad AFTER DELETE ON domain_events BEGIN
    INSERT INTO events_fts(events_fts, rowid, summary) VALUES ('delete', old.id, old.summary);
END;
CREATE TRIGGER IF NOT EXISTS tr_events_au AFTER UPDATE ON domain_events BEGIN
    INSERT INTO events_fts(events_fts, rowid, summary) VALUES ('delete', old.id, old.summary);
    INSERT INTO events_fts(rowid, summary) VALUES (new.id, new.summary);
END;

-- episodic_fts
CREATE TRIGGER IF NOT EXISTS tr_episodic_ai AFTER INSERT ON episodic_memories BEGIN
    INSERT INTO episodic_fts(rowid, content) VALUES (new.id, new.content);
END;
CREATE TRIGGER IF NOT EXISTS tr_episodic_ad AFTER DELETE ON episodic_memories BEGIN
    INSERT INTO episodic_fts(episodic_fts, rowid, content) VALUES ('delete', old.id, old.content);
END;
CREATE TRIGGER IF NOT EXISTS tr_episodic_au AFTER UPDATE ON episodic_memories BEGIN
    INSERT INTO episodic_fts(episodic_fts, rowid, content) VALUES ('delete', old.id, old.content);
    INSERT INTO episodic_fts(rowid, content) VALUES (new.id, new.content);
END;

-- semantic_fts
CREATE TRIGGER IF NOT EXISTS tr_semantic_ai AFTER INSERT ON semantic_memories BEGIN
    INSERT INTO semantic_fts(rowid, content) VALUES (new.id, new.content);
END;
CREATE TRIGGER IF NOT EXISTS tr_semantic_ad AFTER DELETE ON semantic_memories BEGIN
    INSERT INTO semantic_fts(semantic_fts, rowid, content) VALUES ('delete', old.id, old.content);
END;
CREATE TRIGGER IF NOT EXISTS tr_semantic_au AFTER UPDATE ON semantic_memories BEGIN
    INSERT INTO semantic_fts(semantic_fts, rowid, content) VALUES ('delete', old.id, old.content);
    INSERT INTO semantic_fts(rowid, content) VALUES (new.id, new.content);
END;
