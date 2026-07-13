<script setup lang="ts">
import { computed } from 'vue';
export interface GmTemplate {
  gmId: string;
  name: string;
  preview: string;
  worldId?: string;
}

const props = defineProps<{ gms: GmTemplate[] }>();
const emit = defineEmits<{ select: [gmId: string] }>();
const selected = defineModel<string>('selected');

function select(id: string) {
  selected.value = id;
  emit('select', id);
}

const selectedGm = computed(() => props.gms.find(g => g.gmId === selected.value));
</script>

<template>
  <div class="gm-switcher">
    <div class="gs-title">选择 GM（文风）</div>
    <div v-if="!gms.length" class="empty">暂无 GM 模板</div>
    <div class="gs-list">
      <button
        v-for="g in gms" :key="g.gmId"
        class="gs-item" :class="{ active: selected === g.gmId }"
        @click="select(g.gmId)"
      >
        <div class="gs-name">{{ g.name }}</div>
        <div class="gs-preview">{{ g.preview }}</div>
        <span v-if="g.worldId" class="gs-world-badge">{{ g.worldId }}</span>
      </button>
    </div>
    <div v-if="selectedGm" class="gs-preview-full">
      <div class="gs-preview-title">{{ selectedGm.name }} · 预览</div>
      <pre class="gs-preview-text">{{ selectedGm.preview }}</pre>
    </div>
  </div>
</template>

<style scoped>
.gm-switcher { padding: 12px; }
.gs-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; font-size: 16px; }
.empty { color: #999; text-align: center; padding: 24px; }

.gs-list { display: flex; flex-direction: column; gap: 8px; }
.gs-item {
  background: #fff; border: 2px solid #eee; border-radius: 10px;
  padding: 10px 14px; cursor: pointer; text-align: left; transition: border-color .2s;
  min-height: 44px;
}
.gs-item:hover { border-color: #3498db; }
.gs-item.active { border-color: #3498db; background: #ebf5fb; }

.gs-name { font-weight: 600; }
.gs-preview { font-size: 12px; color: #888; margin-top: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.gs-world-badge { font-size: 10px; background: #eee; padding: 1px 6px; border-radius: 4px; color: #888; margin-left: 6px; }

.gs-preview-full { margin-top: 12px; background: #fafafa; border-radius: 8px; padding: 10px; }
.gs-preview-title { font-weight: 600; margin-bottom: 6px; font-size: 13px; }
.gs-preview-text { font-size: 12px; color: #555; white-space: pre-wrap; margin: 0; }

@media (max-width: 767px) { .gm-switcher { padding: 8px; } }
@media (min-width: 1024px) { .gm-switcher { padding: 16px; } }
button { min-height: 44px; min-width: 44px; }
</style>
