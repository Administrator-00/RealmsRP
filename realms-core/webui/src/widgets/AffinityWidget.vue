<script setup lang="ts">
import { computed } from 'vue';
import type { NpcRelationship } from '../types';

const props = defineProps<{ relationships: NpcRelationship[] }>();

const sorted = computed(() =>
  [...props.relationships].sort((a, b) => Math.abs(b.affinity) - Math.abs(a.affinity)),
);

function typeEmoji(t: string): string {
  const m: Record<string, string> = {
    stranger: '👤', friend: '🤝', close_friend: '💛',
    lover: '❤️', enemy: '⚔️', master: '📖', destined: '✨',
  };
  return m[t] || '👤';
}

function barColor(affinity: number): string {
  if (affinity >= 50) return '#2ecc71';
  if (affinity >= 0) return '#bdc3c7';
  if (affinity >= -50) return '#e67e22';
  return '#e74c3c';
}
</script>

<template>
  <div class="affinity-widget">
    <div v-if="!sorted.length" class="empty">暂无关系</div>
    <div v-for="r in sorted" :key="r.npcId" class="affinity-row">
      <span class="affinity-emoji">{{ typeEmoji(r.currentType) }}</span>
      <span class="affinity-name">{{ r.name }}</span>
      <div class="affinity-bar-track">
        <div
          class="affinity-bar-fill"
          :style="{ width: Math.abs(r.affinity) + '%', backgroundColor: barColor(r.affinity) }"
        />
      </div>
      <span class="affinity-value">{{ r.affinity >= 0 ? '+' : '' }}{{ r.affinity }}</span>
      <span class="affinity-type">{{ r.currentType }}</span>
    </div>
  </div>
</template>

<style scoped>
.affinity-widget { padding: 12px; font-size: 14px; }
.empty { color: #999; text-align: center; padding: 16px; }

.affinity-row {
  display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
}
.affinity-emoji { font-size: 16px; min-width: 24px; }
.affinity-name { min-width: 60px; font-weight: 600; }
.affinity-bar-track {
  flex: 1; height: 8px; background: #eee; border-radius: 4px; overflow: hidden;
}
.affinity-bar-fill { height: 100%; border-radius: 4px; transition: width .3s; }
.affinity-value { min-width: 40px; text-align: right; font-variant-numeric: tabular-nums; }
.affinity-type { color: #888; font-size: 12px; min-width: 50px; }

@media (max-width: 767px) {
  .affinity-widget { padding: 8px; }
  .affinity-name { min-width: 48px; font-size: 13px; }
}
@media (min-width: 1024px) {
  .affinity-widget { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
