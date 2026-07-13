const BASE = 'http://localhost:3000/api';

export interface WorldInfo {
  world_id: string;
  setting_preview: string;
  npc_count: number;
  npc_ids: string[];
}

export interface WorldDetail {
  world_id: string;
  setting: string;
  npc_personas: Record<string, string>;
}

export interface GmTemplate {
  gm_id: string;
  content_preview: string;
}

export interface Perspective {
  id: string;
  source: 'npc' | 'oc';
  name: string;
  world_id?: string;
}

export interface CycleInfo {
  cycle_id: string;
  world_id: string;
  perspective_id: string;
  gm_id: string;
  cycle_name: string;
  cycle_description?: string;
  current_world_time?: string;
  total_events: number;
  created_at: string;
  updated_at: string;
  last_played_at?: string;
}

export interface CycleDetail {
  cycle: CycleInfo;
  character_states: CharacterState[];
  relationships: Relationship[];
  events: EventRow[];
}

export interface CharacterState {
  npc_id: string;
  hp: number;
  hp_max: number;
  mp: number;
  mp_max: number;
  location_id?: string;
  arc_phase?: string;
  perceived_mood: string;
  arcs: Record<string, number>;
  status_flags: Record<string, unknown>;
}

export interface Relationship {
  npc_id: string;
  affinity: number;
  trust: number;
  intimacy: number;
  respect: number;
  current_type?: string;
}

export interface EventRow {
  id: number;
  event_type: string;
  actor_id?: string;
  summary: string;
  world_time?: string;
}

export interface LlmConfig {
  provider: string;
  endpoint: string;
  api_key: string;
  model: string;
}

export interface AppConfig {
  llm: LlmConfig;
}

// === API functions ===

export async function listWorlds(): Promise<WorldInfo[]> {
  const res = await fetch(`${BASE}/worlds`);
  if (!res.ok) throw new Error(`listWorlds: ${res.status}`);
  return res.json();
}

export async function getWorld(id: string): Promise<WorldDetail> {
  const res = await fetch(`${BASE}/worlds/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`getWorld: ${res.status}`);
  return res.json();
}

export async function listGms(worldId?: string): Promise<GmTemplate[]> {
  const params = worldId ? `?world_id=${encodeURIComponent(worldId)}` : '';
  const res = await fetch(`${BASE}/gms${params}`);
  if (!res.ok) throw new Error(`listGms: ${res.status}`);
  return res.json();
}

export async function listPerspectives(): Promise<Perspective[]> {
  const res = await fetch(`${BASE}/perspectives`);
  if (!res.ok) throw new Error(`listPerspectives: ${res.status}`);
  return res.json();
}

export async function createPerspective(data: {
  id: string;
  source: string;
  name: string;
  persona_data: string;
  world_id?: string;
}): Promise<Perspective> {
  const res = await fetch(`${BASE}/perspectives`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!res.ok) throw new Error(`createPerspective: ${res.status}`);
  return res.json();
}

export async function listCycles(worldId: string): Promise<CycleInfo[]> {
  const res = await fetch(`${BASE}/cycles?world_id=${encodeURIComponent(worldId)}`);
  if (!res.ok) throw new Error(`listCycles: ${res.status}`);
  return res.json();
}

export async function createCycle(data: {
  world_id: string;
  perspective_id: string;
  gm_id: string;
  cycle_name: string;
  oc_selections?: Record<string, string | null>;
}): Promise<CycleInfo> {
  const res = await fetch(`${BASE}/cycles`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  });
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || `createCycle: ${res.status}`);
  }
  return res.json();
}

export async function getCycle(id: string): Promise<CycleDetail> {
  const res = await fetch(`${BASE}/cycles/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error(`getCycle: ${res.status}`);
  return res.json();
}

export async function deleteCycle(id: string): Promise<void> {
  const res = await fetch(`${BASE}/cycles/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!res.ok) throw new Error(`deleteCycle: ${res.status}`);
}

export async function processTurn(
  cycleId: string,
  userMessage: string,
): Promise<{ narrative: string; private_intent: string }> {
  const res = await fetch(`${BASE}/cycles/${encodeURIComponent(cycleId)}/turn`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user_message: userMessage }),
  });
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || `processTurn: ${res.status}`);
  }
  return res.json();
}

export async function getConfig(): Promise<AppConfig> {
  const res = await fetch(`${BASE}/config`);
  if (!res.ok) throw new Error(`getConfig: ${res.status}`);
  return res.json();
}

export async function updateConfig(config: AppConfig): Promise<AppConfig> {
  const res = await fetch(`${BASE}/config`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(config),
  });
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || `updateConfig: ${res.status}`);
  }
  return res.json();
}

export async function testConfig(): Promise<{ status: string; message: string }> {
  const res = await fetch(`${BASE}/config/test`, { method: 'POST' });
  if (!res.ok) throw new Error(`testConfig: ${res.status}`);
  return res.json();
}
