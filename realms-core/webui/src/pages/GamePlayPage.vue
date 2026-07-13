<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import { getCycle, processTurn, type CycleDetail } from '../api';
import type { CharacterState, NpcRelationship, InventoryItem, ArcTimelineNode, EmotionalNode } from '../types';
import ChatWidget from '../widgets/ChatWidget.vue';
import StateWidget from '../widgets/StateWidget.vue';
import AffinityWidget from '../widgets/AffinityWidget.vue';
import InventoryWidget from '../widgets/InventoryWidget.vue';
import ArcTimelineWidget from '../widgets/ArcTimelineWidget.vue';
import EmotionalMap from '../widgets/EmotionalMap.vue';

const route = useRoute();
const cycleId = (route.params.cycleId as string) || '';

const cycleDetail = ref<CycleDetail | null>(null);
const messages = ref<{ id: string; role: 'user' | 'npc' | 'system'; senderName: string; content: string }[]>([]);
const sending = ref(false);
const loading = ref(true);
const error = ref('');
const activePanel = ref<'chat' | 'state' | 'rel' | 'items' | 'arcs'>('chat');

let msgIdCounter = 0;

const characterStates = computed<CharacterState[]>(() => {
  if (!cycleDetail.value) return [];
  return cycleDetail.value.character_states.map(cs => ({
    cycleId: cycleId,
    npcId: cs.npc_id,
    hp: cs.hp,
    hpMax: cs.hp_max,
    mp: cs.mp,
    mpMax: cs.mp_max,
    locationId: cs.location_id,
    statusFlags: (cs.status_flags as Record<string, boolean | number | string>) || {},
    arcs: (cs.arcs as Record<string, number>) || {},
    arcPhase: cs.arc_phase,
    perceivedMood: cs.perceived_mood,
  }));
});

const relationships = computed<NpcRelationship[]>(() => {
  if (!cycleDetail.value) return [];
  return cycleDetail.value.relationships.map(r => ({
    cycleId: cycleId,
    npcId: r.npc_id,
    name: r.npc_id,
    affinity: r.affinity,
    trust: r.trust,
    intimacy: r.intimacy,
    currentType: r.current_type || 'stranger',
  }));
});

const arcTimeline = computed<ArcTimelineNode[]>(() => {
  if (!cycleDetail.value) return [];
  return cycleDetail.value.character_states.flatMap(cs =>
    Object.entries(cs.arcs as Record<string, number>).map(([dim, val]) => ({
      worldTime: cycleDetail.value!.cycle.current_world_time || '',
      dimension: dim,
      value: val,
      label: cs.arc_phase || dim,
      summary: `${cs.npc_id}: ${dim}=${val}`,
    })),
  );
});

const emotionalNodes = computed<EmotionalNode[]>(() => {
  if (!cycleDetail.value) return [];
  return cycleDetail.value.relationships.map(r => ({
    npcId: r.npc_id,
    npcName: r.npc_id,
    emotionType: r.affinity > 0 ? 'love' : 'hate',
    intensity: Math.abs(r.affinity) / 100,
  }));
});

onMounted(async () => {
  try {
    cycleDetail.value = await getCycle(cycleId);
  } catch (e: any) {
    error.value = e.message;
  } finally {
    loading.value = false;
  }
});

async function handleSend(text: string) {
  if (sending.value) return;
  const id = String(++msgIdCounter);
  messages.value.push({ id, role: 'user', senderName: '你', content: text });
  sending.value = true;
  error.value = '';

  try {
    const result = await processTurn(cycleId, text);
    messages.value.push({
      id: String(++msgIdCounter),
      role: 'system',
      senderName: 'GM',
      content: result.narrative,
    });
    cycleDetail.value = await getCycle(cycleId);
  } catch (e: any) {
    error.value = e.message;
  } finally {
    sending.value = false;
  }
}
</script>

<template>
  <div class="gameplay">
    <div v-if="loading" class="loading">加载周目...</div>
    <div v-else-if="error && !cycleDetail" class="error">{{ error }}</div>

    <template v-else>
      <!-- Chat -->
      <div class="chat-panel" :class="{ hidden: activePanel !== 'chat' }">
        <ChatWidget
          :messages="messages"
          @send="handleSend"
        />
        <div v-if="error" class="inline-error">{{ error }}</div>
      </div>

      <!-- Side panels -->
      <div class="side-panels">
        <!-- State -->
        <div v-if="activePanel === 'state'" class="panel">
          <h3>角色状态</h3>
          <div v-if="!characterStates.length" class="empty">无数据</div>
          <StateWidget
            v-for="cs in characterStates"
            :key="cs.npcId"
            :state="cs"
          />
        </div>

        <!-- Affinities -->
        <div v-if="activePanel === 'rel'" class="panel">
          <AffinityWidget :relationships="relationships" />
        </div>

        <!-- Items -->
        <div v-if="activePanel === 'items'" class="panel">
          <InventoryWidget :items="[]" />
          <p class="empty-hint">物品系统即将上线</p>
        </div>

        <!-- Arcs -->
        <div v-if="activePanel === 'arcs'" class="panel">
          <ArcTimelineWidget :timeline="arcTimeline" />
        </div>
      </div>

      <!-- Mobile tabs -->
      <div class="mobile-tabs">
        <button :class="{ active: activePanel === 'chat' }" @click="activePanel = 'chat'">聊天</button>
        <button :class="{ active: activePanel === 'state' }" @click="activePanel = 'state'">状态</button>
        <button :class="{ active: activePanel === 'rel' }" @click="activePanel = 'rel'">好感</button>
        <button :class="{ active: activePanel === 'items' }" @click="activePanel = 'items'">物品</button>
        <button :class="{ active: activePanel === 'arcs' }" @click="activePanel = 'arcs'">弧光</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.gameplay {
  display: flex;
  flex-direction: column;
  max-width: 900px;
  margin: 0 auto;
  min-height: calc(100vh - 120px);
  gap: 16px;
}

.loading, .error {
  text-align: center;
  padding: 60px;
  color: #666;
}
.error { color: #c0392b; }
.inline-error {
  color: #c0392b;
  padding: 8px;
  margin-top: 8px;
  background: #ffe8e8;
  border-radius: 6px;
  font-size: 13px;
}

.chat-panel {
  flex: 1;
  min-height: 300px;
}
.chat-panel.hidden { display: none; }

.side-panels {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.panel {
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
  overflow: hidden;
}

.panel h3 {
  font-size: 14px;
  color: #555;
  padding: 12px 12px 0;
  margin: 0;
}

.empty { color: #999; text-align: center; padding: 24px; }
.empty-hint { text-align: center; color: #999; font-size: 12px; padding: 16px; }

/* Mobile tabs */
.mobile-tabs {
  display: none;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: #fff;
  border-top: 1px solid #eee;
  z-index: 50;
}

.mobile-tabs button {
  flex: 1;
  padding: 10px 6px;
  border: none;
  background: none;
  font-size: 12px;
  color: #666;
  cursor: pointer;
  min-height: 44px;
}
.mobile-tabs button.active {
  color: #3498db;
  font-weight: 600;
}

@media (max-width: 767px) {
  .chat-panel.hidden { display: none; }
  .mobile-tabs { display: flex; }
  .side-panels {
    display: block;
    margin-bottom: 56px;
  }
}

@media (min-width: 768px) {
  .mobile-tabs { display: none; }
  .chat-panel.hidden { display: block; }
  .gameplay {
    display: grid;
    grid-template-columns: 1fr 320px;
    gap: 16px;
    align-items: start;
  }
  .side-panels {
    grid-template-columns: 1fr;
    position: sticky;
    top: 16px;
    max-height: calc(100vh - 120px);
    overflow-y: auto;
  }
}
</style>
