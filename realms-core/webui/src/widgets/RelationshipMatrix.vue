<script setup lang="ts">
import { computed } from 'vue';

interface MatrixRow {
  npcId: string;
  npcName: string;
  affinity: number;
  trust: number;
  intimacy: number;
  currentType?: string;
}

const props = defineProps<{
  cycleName: string;
  rows: MatrixRow[];
}>();

const dims: (keyof MatrixRow)[] = ['affinity', 'trust', 'intimacy'];

function barColor(val: number, maxAbs: number): string {
  const pct = maxAbs > 0 ? Math.abs(val) / maxAbs : 0;
  if (val >= 50) return `rgba(46,204,113,${0.3 + pct * 0.7})`;
  if (val >= 0) return `rgba(189,195,199,${0.3 + pct * 0.7})`;
  return `rgba(231,76,60,${0.3 + pct * 0.7})`;
}

const maxAbs = computed(() => {
  let m = 0;
  for (const r of props.rows) {
    for (const d of dims) {
      m = Math.max(m, Math.abs(r[d] as number));
    }
  }
  return m || 1;
});
</script>

<template>
  <div class="rel-matrix-widget">
    <div class="rm-title">{{ cycleName }} · 关系矩阵</div>
    <div v-if="!rows.length" class="empty">暂无关系数据</div>
    <div v-else class="rm-grid">
      <div v-for="row in rows" :key="row.npcId" class="rm-card">
        <div class="rm-npc-name">{{ row.npcName }}</div>
        <div v-for="d in dims" :key="d" class="rm-bar-row">
          <span class="rm-bar-label">{{ d === 'affinity' ? '好感' : d === 'trust' ? '信任' : '亲密' }}</span>
          <div class="rm-bar-track">
            <div
              class="rm-bar-fill"
              :style="{
                width: (Math.abs(row[d]) / 100 * 100) + '%',
                background: barColor(row[d], maxAbs),
              }"
            />
          </div>
          <span class="rm-bar-val" :class="row[d] >= 0 ? 'positive' : 'negative'">
            {{ row[d] >= 0 ? '+' : '' }}{{ row[d] }}
          </span>
        </div>
        <div class="rm-type" v-if="row.currentType">{{ row.currentType }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.rel-matrix-widget { padding: 12px; }
.rm-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; }
.empty { color: #999; text-align: center; padding: 16px; }

.rm-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 10px; }

.rm-card {
  background: #fafafa; border-radius: 10px; padding: 10px;
  border: 1px solid #eee;
}
.rm-npc-name { font-weight: 600; margin-bottom: 6px; font-size: 14px; }

.rm-bar-row { display: flex; align-items: center; gap: 6px; margin: 3px 0; }
.rm-bar-label { font-size: 11px; color: #888; min-width: 28px; }
.rm-bar-track { flex: 1; height: 6px; background: #eee; border-radius: 3px; overflow: hidden; }
.rm-bar-fill { height: 100%; border-radius: 3px; transition: width .3s; }
.rm-bar-val { font-size: 12px; min-width: 32px; text-align: right; font-variant-numeric: tabular-nums; }

.positive { color: #27ae60; }
.negative { color: #e74c3c; }

.rm-type { margin-top: 4px; font-size: 11px; color: #888; }

@media (max-width: 767px) {
  .rel-matrix-widget { padding: 8px; }
  .rm-grid { grid-template-columns: 1fr; }
}
@media (min-width: 1024px) {
  .rel-matrix-widget { padding: 16px; }
  .rm-grid { grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); }
}
</style>
