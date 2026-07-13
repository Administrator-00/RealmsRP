// === Realms widget 共享类型 (映射 Rust character_states / npc_user_relationships / arcs) ===

/** character_states 行投影 */
export interface CharacterState {
  cycleId: string;
  npcId: string;
  hp: number;
  hpMax: number;
  mp: number;
  mpMax: number;
  locationId?: string;
  statusFlags: Record<string, boolean | number | string>;
  arcs: Record<string, number>;
  arcPhase?: string;
  perceivedMood: string;
}

/** npc_user_relationships 行投影 */
export interface NpcRelationship {
  cycleId: string;
  npcId: string;
  name: string;
  affinity: number;   // -100..100
  trust: number;       // 0..100
  intimacy: number;    // 0..100
  currentType: string; // stranger/friend/lover/...
}

/** 物品 */
export interface InventoryItem {
  id: string;
  name: string;
  type: 'weapon' | 'potion' | 'key' | 'armor' | 'book' | 'treasure' | 'food';
  quantity: number;
  properties?: Record<string, string>;
}

/** 弧光时间线节点 */
export interface ArcTimelineNode {
  worldTime: string;
  dimension: string;
  value: number;
  label: string;
  summary: string;
}

/** 情感地图节点 */
export interface EmotionalNode {
  npcId: string;
  npcName: string;
  emotionType: string;  // love/hate/fear/respect/...
  intensity: number;    // 0..1
}

/** Blueprint patch (RFC6902) */
export interface PatchOp {
  op: 'add' | 'replace' | 'remove';
  path: string;
  value?: unknown;
}
