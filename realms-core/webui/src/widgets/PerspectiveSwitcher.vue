<script setup lang="ts">
export interface PerspectiveInfo {
  id: string;
  name: string;
  source: 'npc' | 'oc';
  worldId?: string;
}

const props = defineProps<{ perspectives: PerspectiveInfo[] }>();
const emit = defineEmits<{ select: [id: string] }>();
const selected = defineModel<string>('selected');

function select(id: string) {
  selected.value = id;
  emit('select', id);
}
</script>

<template>
  <div class="perspective-switcher">
    <div class="ps-title">切换视角</div>
    <div v-if="!perspectives.length" class="empty">暂无视角</div>
    <div class="ps-list">
      <button
        v-for="p in perspectives" :key="p.id"
        class="ps-item" :class="{ active: selected === p.id }"
        @click="select(p.id)"
      >
        <span class="ps-name">{{ p.name }}</span>
        <span class="ps-badge" :class="p.source">{{ p.source === 'npc' ? '内置' : 'OC' }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.perspective-switcher { padding: 12px; }
.ps-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; font-size: 16px; }
.empty { color: #999; text-align: center; padding: 24px; }

.ps-list { display: flex; flex-wrap: wrap; gap: 8px; }
.ps-item {
  display: flex; align-items: center; gap: 6px;
  background: #fff; border: 2px solid #eee; border-radius: 10px;
  padding: 8px 14px; cursor: pointer; transition: border-color .2s;
  min-height: 44px;
}
.ps-item:hover { border-color: #3498db; }
.ps-item.active { border-color: #3498db; background: #ebf5fb; }

.ps-name { font-weight: 600; font-size: 14px; }
.ps-badge {
  font-size: 10px; padding: 1px 6px; border-radius: 6px; color: #fff;
}
.ps-badge.npc { background: #3498db; }
.ps-badge.oc { background: #e67e22; }

@media (max-width: 767px) { .perspective-switcher { padding: 8px; } }
@media (min-width: 1024px) { .perspective-switcher { padding: 16px; } }
button { min-height: 44px; min-width: 44px; }
</style>
