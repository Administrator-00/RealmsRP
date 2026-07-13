<script setup lang="ts">
import type { EmotionalNode } from '../types';

defineProps<{ nodes: EmotionalNode[] }>();

const emotionMap: Record<string, string> = {
  love: '❤️', hate: '💢', fear: '😨', respect: '🙏',
  gratitude: '🥹', guilt: '😞', longing: '💭',
};

function emotionLabel(e: string): string {
  const m: Record<string, string> = {
    love: '爱慕', hate: '憎恨', fear: '恐惧', respect: '尊敬',
    gratitude: '感激', guilt: '内疚', longing: '思念',
  };
  return m[e] || e;
}

function intensityColor(i: number): string {
  if (i >= 0.7) return '#e74c3c';
  if (i >= 0.4) return '#f39c12';
  return '#95a5a6';
}
</script>

<template>
  <div class="emotional-map-widget">
    <div v-if="!nodes.length" class="empty">无情感数据</div>
    <div v-for="n in nodes" :key="n.npcId + n.emotionType" class="emotion-row">
      <span class="emoji">{{ emotionMap[n.emotionType] || '🤔' }}</span>
      <span class="npc-name">{{ n.npcName }}</span>
      <div class="intensity-track">
        <div
          class="intensity-fill"
          :style="{ width: (n.intensity * 100) + '%', backgroundColor: intensityColor(n.intensity) }"
        />
      </div>
      <span class="emotion-label">{{ emotionLabel(n.emotionType) }}</span>
    </div>
  </div>
</template>

<style scoped>
.emotional-map-widget { padding: 12px; }
.empty { color: #999; text-align: center; padding: 16px; }

.emotion-row {
  display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
}
.emoji { font-size: 18px; min-width: 28px; text-align: center; }
.npc-name { min-width: 56px; font-weight: 600; font-size: 13px; }
.intensity-track {
  flex: 1; height: 8px; background: #eee; border-radius: 4px; overflow: hidden;
}
.intensity-fill { height: 100%; border-radius: 4px; transition: width .3s; }
.emotion-label { color: #888; font-size: 12px; min-width: 40px; }

@media (max-width: 767px) {
  .emotional-map-widget { padding: 8px; }
}
@media (min-width: 1024px) {
  .emotional-map-widget { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
