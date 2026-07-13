<script setup lang="ts">
import { computed } from 'vue';

interface CycleComparison {
  cycleId: string;
  cycleName: string;
  npcId: string;
  affinity: number;
  trust: number;
  intimacy: number;
  currentType?: string;
  initialTemplate?: string;
}

const props = defineProps<{
  npcName: string;
  comparisons: CycleComparison[];
}>();

const dims = ['affinity', 'trust', 'intimacy'] as const;

const colorMap: Record<string, string> = {
  stranger: '#bdc3c7',
  friend: '#3498db',
  close_friend: '#2ecc71',
  lover: '#e74c3c',
  enemy: '#e67e22',
  master: '#9b59b6',
  destined: '#f1c40f',
};
</script>

<template>
  <div class="cycle-compare-widget">
    <div class="cc-title">{{ npcName }} · 跨周目对比</div>
    <div v-if="!comparisons.length" class="empty">无数据</div>
    <div v-else class="cc-table-wrap">
      <table class="cc-table">
        <thead>
          <tr>
            <th>周目</th>
            <th v-for="d in dims" :key="d" class="num-col">{{ d }}</th>
            <th>关系</th>
            <th>初始模板</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in comparisons" :key="row.cycleId">
            <td class="name-col">{{ row.cycleName }}</td>
            <td v-for="d in dims" :key="d" class="num-col">
              <span :class="row[d] >= 0 ? 'positive' : 'negative'">
                {{ row[d] >= 0 ? '+' : '' }}{{ row[d] }}
              </span>
            </td>
            <td>
              <span class="type-badge" :style="{ background: colorMap[row.currentType || ''] || '#ccc' }">
                {{ row.currentType || '-' }}
              </span>
            </td>
            <td class="tpl-col">{{ row.initialTemplate || '-' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.cycle-compare-widget { padding: 12px; }
.cc-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; }
.empty { color: #999; text-align: center; padding: 16px; }

.cc-table-wrap { overflow-x: auto; }
.cc-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.cc-table th, .cc-table td { padding: 6px 10px; text-align: left; border-bottom: 1px solid #eee; }
.cc-table th { color: #888; font-weight: 600; font-size: 12px; }
.num-col { text-align: right; font-variant-numeric: tabular-nums; }
.name-col { font-weight: 600; min-width: 70px; }
.tpl-col { color: #999; font-size: 12px; }

.positive { color: #27ae60; }
.negative { color: #e74c3c; }

.type-badge {
  display: inline-block; padding: 1px 8px; border-radius: 8px;
  color: #fff; font-size: 11px;
}

@media (max-width: 767px) {
  .cycle-compare-widget { padding: 8px; }
  .cc-table th, .cc-table td { padding: 4px 6px; font-size: 12px; }
}
@media (min-width: 1024px) {
  .cycle-compare-widget { padding: 16px; }
}
</style>
