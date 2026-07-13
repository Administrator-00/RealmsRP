<script setup lang="ts">
import { computed } from 'vue';
import type { CharacterState } from '../types';

const props = defineProps<{ state: CharacterState }>();

const hpPct = computed(() =>
  props.state.hpMax > 0 ? Math.round((props.state.hp / props.state.hpMax) * 100) : 0,
);
const mpPct = computed(() =>
  props.state.mpMax > 0 ? Math.round((props.state.mp / props.state.mpMax) * 100) : 0,
);

const statusFlags = computed(() => {
  const flags = props.state.statusFlags;
  return Object.entries(flags).filter(([, v]) => v === true || (typeof v === 'string' && v !== 'false'));
});

const moodEmoji = computed(() => {
  const m = props.state.perceivedMood;
  const map: Record<string, string> = { happy: '😊', sad: '😢', angry: '😠', neutral: '😐' };
  return map[m] || '😐';
});
</script>

<template>
  <div class="state-widget" :data-npc="state.npcId">
    <!-- HP bar -->
    <div class="bar-row">
      <span class="bar-label">HP</span>
      <div class="bar-track" role="progressbar" :aria-valuenow="state.hp" :aria-valuemax="state.hpMax">
        <div class="bar-fill hp-fill" :style="{ width: hpPct + '%' }" />
      </div>
      <span class="bar-value">{{ state.hp }}/{{ state.hpMax }}</span>
    </div>
    <!-- MP bar -->
    <div class="bar-row">
      <span class="bar-label">MP</span>
      <div class="bar-track" role="progressbar" :aria-valuenow="state.mp" :aria-valuemax="state.mpMax">
        <div class="bar-fill mp-fill" :style="{ width: mpPct + '%' }" />
      </div>
      <span class="bar-value">{{ state.mp }}/{{ state.mpMax }}</span>
    </div>
    <!-- info rows -->
    <div class="info-row" v-if="state.locationId">
      <span class="info-label">📍</span>
      <span class="info-text">{{ state.locationId }}</span>
    </div>
    <div class="info-row">
      <span class="info-label">😶</span>
      <span class="info-text">{{ moodEmoji }} {{ state.perceivedMood }}</span>
    </div>
    <div class="info-row" v-if="state.arcPhase">
      <span class="info-label">📈</span>
      <span class="info-text">{{ state.arcPhase }}</span>
    </div>
    <!-- status flags -->
    <div class="flags-row" v-if="statusFlags.length">
      <span v-for="[k] in statusFlags" :key="k" class="flag-tag">{{ k }}</span>
    </div>
  </div>
</template>

<style scoped>
.state-widget { padding: 12px; font-size: 14px; }

/* bars */
.bar-row { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
.bar-label { width: 28px; font-weight: 600; color: #666; }
.bar-track { flex: 1; height: 10px; background: #eee; border-radius: 5px; overflow: hidden; }
.bar-fill { height: 100%; border-radius: 5px; transition: width .3s; }
.hp-fill { background: #e74c3c; }
.mp-fill { background: #3498db; }
.bar-value { min-width: 60px; text-align: right; font-variant-numeric: tabular-nums; }

/* info */
.info-row { display: flex; align-items: center; gap: 6px; margin: 4px 0; }
.info-label { font-size: 14px; }
.info-text { color: #333; }

/* flags */
.flags-row { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; }
.flag-tag {
  background: #fdebd0; color: #b9770e; font-size: 12px;
  padding: 2px 8px; border-radius: 10px;
}

/* responsive: phone < 768px */
@media (max-width: 767px) {
  .state-widget { padding: 8px; font-size: 13px; }
  .bar-track { height: 12px; }
}

/* responsive: tablet ≥ 768px */
@media (min-width: 768px) {
  .bar-track { height: 10px; }
}

/* responsive: PC ≥ 1024px */
@media (min-width: 1024px) {
  .state-widget { padding: 16px; }
}

/* touch: button ≥ 44px (iOS HIG) */
button { min-height: 44px; min-width: 44px; }
</style>
