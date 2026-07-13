<script setup lang="ts">
export interface NpcInfo {
  npcId: string;
  name: string;
  description: string;
  source: 'npc' | 'oc';
  avatar?: string;
}

const props = defineProps<{ npcs: NpcInfo[] }>();
const emit = defineEmits<{ select: [npcId: string] }>();
const selected = defineModel<string>('selected');

function select(id: string) {
  selected.value = id;
  emit('select', id);
}
</script>

<template>
  <div class="npc-selector">
    <div class="ns-title">选择角色</div>
    <div v-if="!npcs.length" class="empty">暂无可选角色</div>
    <div class="ns-grid">
      <button
        v-for="n in npcs" :key="n.npcId"
        class="ns-card" :class="{ active: selected === n.npcId }"
        @click="select(n.npcId)"
      >
        <div class="ns-avatar" v-if="n.avatar">{{ n.avatar }}</div>
        <div class="ns-avatar-placeholder" v-else>{{ n.name[0] }}</div>
        <div class="ns-name">{{ n.name }}</div>
        <div class="ns-desc">{{ n.description }}</div>
        <span class="ns-badge" :class="n.source">{{ n.source === 'npc' ? '内置' : 'OC' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.npc-selector { padding: 12px; }
.ns-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; font-size: 16px; }
.empty { color: #999; text-align: center; padding: 24px; }

.ns-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 10px; }

.ns-card {
  background: #fff; border: 2px solid #eee; border-radius: 12px;
  padding: 12px; cursor: pointer; text-align: center; transition: border-color .2s;
  min-height: 44px; position: relative;
}
.ns-card:hover { border-color: #3498db; }
.ns-card.active { border-color: #3498db; background: #ebf5fb; }

.ns-avatar-placeholder {
  width: 36px; height: 36px; border-radius: 50%; background: #eee;
  display: flex; align-items: center; justify-content: center; margin: 0 auto 6px;
  font-weight: 700; color: #666;
}
.ns-name { font-weight: 600; margin-bottom: 2px; }
.ns-desc { font-size: 11px; color: #666; margin-bottom: 4px; }
.ns-badge {
  font-size: 10px; padding: 1px 6px; border-radius: 6px; color: #fff;
}
.ns-badge.npc { background: #3498db; }
.ns-badge.oc { background: #e67e22; }

@media (max-width: 767px) {
  .npc-selector { padding: 8px; }
  .ns-grid { grid-template-columns: 1fr 1fr; }
}
@media (min-width: 1024px) {
  .npc-selector { padding: 16px; }
  .ns-grid { grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); }
}
button { min-height: 44px; min-width: 44px; }
</style>
