<script setup lang="ts">
import { computed } from 'vue';

export interface WorldInfo {
  worldId: string;
  name: string;
  description: string;
  npcCount: number;
}

const props = defineProps<{ worlds: WorldInfo[] }>();
const emit = defineEmits<{ select: [worldId: string] }>();

const selected = defineModel<string>('selected');

function select(id: string) {
  selected.value = id;
  emit('select', id);
}
</script>

<template>
  <div class="world-selector">
    <div class="ws-title">选择一个世界</div>
    <div v-if="!worlds.length" class="empty">暂无可用世界</div>
    <div class="ws-grid">
      <button
        v-for="w in worlds" :key="w.worldId"
        class="ws-card" :class="{ active: selected === w.worldId }"
        @click="select(w.worldId)"
      >
        <div class="ws-name">{{ w.name }}</div>
        <div class="ws-desc">{{ w.description }}</div>
        <div class="ws-npc">{{ w.npcCount }} 位 NPC</div>
      </button>
    </div>
  </div>
</template>

<style scoped>
.world-selector { padding: 12px; }
.ws-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; font-size: 16px; }
.empty { color: #999; text-align: center; padding: 24px; }

.ws-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 10px; }

.ws-card {
  background: #fff; border: 2px solid #eee; border-radius: 12px;
  padding: 14px; cursor: pointer; text-align: left; transition: border-color .2s;
  min-height: 44px;
}
.ws-card:hover { border-color: #3498db; }
.ws-card.active { border-color: #3498db; background: #ebf5fb; }

.ws-name { font-weight: 600; margin-bottom: 4px; }
.ws-desc { font-size: 12px; color: #666; margin-bottom: 6px; }
.ws-npc { font-size: 11px; color: #999; }

@media (max-width: 767px) {
  .world-selector { padding: 8px; }
  .ws-grid { grid-template-columns: 1fr; }
}
@media (min-width: 1024px) {
  .world-selector { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
