<script setup lang="ts">
export interface CycleInfo {
  cycleId: string;
  cycleName: string;
  worldId: string;
  perspectiveName: string;
  gmName: string;
  lastPlayedAt?: string;
  totalEvents: number;
}

const props = defineProps<{
  cycles: CycleInfo[];
  worldName: string;
}>();

const emit = defineEmits<{
  select: [cycleId: string];
  duplicate: [cycleId: string];
  delete: [cycleId: string];
}>();
</script>

<template>
  <div class="cycle-list">
    <div class="cl-title">{{ worldName }} · 周目列表</div>
    <div v-if="!cycles.length" class="empty">暂无周目，创建一个吧</div>
    <div v-for="c in cycles" :key="c.cycleId" class="cl-card">
      <div class="cl-main">
        <div class="cl-name">{{ c.cycleName }}</div>
        <div class="cl-meta">
          <span>{{ c.perspectiveName }}</span>
          <span class="cl-sep">·</span>
          <span>{{ c.gmName }}</span>
          <span class="cl-sep">·</span>
          <span>{{ c.totalEvents }} 事件</span>
        </div>
        <div class="cl-time" v-if="c.lastPlayedAt">{{ c.lastPlayedAt }}</div>
      </div>
      <div class="cl-actions">
        <button class="cl-btn cl-btn-primary" @click="emit('select', c.cycleId)">继续</button>
        <button class="cl-btn" @click="emit('duplicate', c.cycleId)">复制</button>
        <button class="cl-btn cl-btn-danger" @click="emit('delete', c.cycleId)">删除</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cycle-list { padding: 12px; }
.cl-title { font-weight: 700; margin-bottom: 10px; color: #2c3e50; font-size: 16px; }
.empty { color: #999; text-align: center; padding: 24px; }

.cl-card {
  display: flex; justify-content: space-between; align-items: center;
  background: #fff; border: 1px solid #eee; border-radius: 10px;
  padding: 12px 16px; margin-bottom: 8px;
}
.cl-name { font-weight: 600; font-size: 15px; margin-bottom: 3px; }
.cl-meta { font-size: 12px; color: #888; }
.cl-sep { margin: 0 4px; }
.cl-time { font-size: 11px; color: #aaa; margin-top: 2px; }

.cl-actions { display: flex; gap: 6px; flex-shrink: 0; }
.cl-btn {
  padding: 6px 12px; border-radius: 6px; border: 1px solid #ddd;
  background: #fff; cursor: pointer; font-size: 12px;
}
.cl-btn-primary { background: #3498db; color: #fff; border-color: #3498db; }
.cl-btn-danger { border-color: #e74c3c; color: #e74c3c; }

@media (max-width: 767px) {
  .cycle-list { padding: 8px; }
  .cl-card { flex-direction: column; align-items: stretch; gap: 8px; }
  .cl-actions { justify-content: flex-end; }
}
@media (min-width: 1024px) {
  .cycle-list { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
