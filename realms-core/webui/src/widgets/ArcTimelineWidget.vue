<script setup lang="ts">
import { computed } from 'vue';
import type { ArcTimelineNode } from '../types';

const props = defineProps<{ timeline: ArcTimelineNode[]; dimension?: string }>();

const filtered = computed(() => {
  if (!props.dimension) return props.timeline;
  return props.timeline.filter((n) => n.dimension === props.dimension);
});

const dimLabels: Record<string, string> = {
  trust_user: '信任', affection_user: '爱慕', hostility_user: '敌意',
  courage: '勇气', maturity: '成熟',
};
</script>

<template>
  <div class="arc-timeline-widget">
    <div class="timeline-dim" v-if="dimension">{{ dimLabels[dimension] || dimension }}</div>
    <div v-if="!filtered.length" class="empty">无弧光数据</div>
    <div class="timeline-scroll">
      <div v-for="node in filtered" :key="node.worldTime + node.dimension" class="timeline-node">
        <div class="node-dot" :style="{ left: ((node.value / 100) * 100) + '%' }" />
        <div class="node-info">
          <div class="node-label">{{ node.label }}</div>
          <div class="node-time">{{ node.worldTime }}</div>
        </div>
        <div class="node-bar">
          <div class="node-bar-fill" :style="{ width: ((node.value / 100) * 100) + '%' }" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.arc-timeline-widget { padding: 12px; }
.timeline-dim { font-weight: 600; margin-bottom: 8px; color: #2c3e50; }
.empty { color: #999; text-align: center; padding: 16px; }

.timeline-scroll { max-height: 260px; overflow-y: auto; }
.timeline-node { margin-bottom: 14px; position: relative; }
.node-dot {
  position: absolute; top: 2px; width: 10px; height: 10px; border-radius: 50%;
  background: #3498db; z-index: 1; transition: left .3s;
}
.node-info { display: flex; justify-content: space-between; margin-left: 16px; margin-bottom: 4px; }
.node-label { font-size: 13px; color: #333; }
.node-time { font-size: 11px; color: #999; }

.node-bar { height: 6px; background: #eee; border-radius: 3px; }
.node-bar-fill { height: 100%; background: #3498db; border-radius: 3px; transition: width .3s; }

@media (max-width: 767px) {
  .arc-timeline-widget { padding: 8px; }
}
@media (min-width: 1024px) {
  .arc-timeline-widget { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
