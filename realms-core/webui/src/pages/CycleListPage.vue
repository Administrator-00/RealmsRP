<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { listCycles, createCycle, deleteCycle } from '../api';
import CycleList, { type CycleInfo } from '../widgets/CycleList.vue';

const router = useRouter();
const route = useRoute();
const worldId = (route.query.world_id as string) || '';
const perspectiveId = (route.query.perspective_id as string) || '';
const gmId = (route.query.gm_id as string) || '';

const cycles = ref<CycleInfo[]>([]);
const loading = ref(true);
const error = ref('');
const creating = ref(false);

onMounted(async () => {
  if (!worldId) {
    error.value = '未选择世界';
    loading.value = false;
    return;
  }
  await loadCycles();
});

async function loadCycles() {
  try {
    const data = await listCycles(worldId);
    cycles.value = data.map(c => ({
      cycleId: c.cycle_id,
      cycleName: c.cycle_name,
      worldId: c.world_id,
      perspectiveName: c.perspective_id,
      gmName: c.gm_id,
      lastPlayedAt: c.last_played_at,
      totalEvents: c.total_events,
    }));
    loading.value = false;
  } catch (e: any) {
    error.value = e.message;
    loading.value = false;
  }
}

function handleSelect(cycleId: string) {
  router.push(`/play/${cycleId}`);
}

async function handleDelete(cycleId: string) {
  if (!confirm('确定删除此周目？')) return;
  try {
    await deleteCycle(cycleId);
    cycles.value = cycles.value.filter(c => c.cycleId !== cycleId);
  } catch (e: any) {
    error.value = e.message;
  }
}

async function handleCreate() {
  if (!worldId || !perspectiveId || !gmId) {
    error.value = '缺少必要参数';
    return;
  }
  creating.value = true;
  try {
    const cycle = await createCycle({
      world_id: worldId,
      perspective_id: perspectiveId,
      gm_id: gmId,
      cycle_name: `周目 ${cycles.value.length + 1}`,
    });
    router.push(`/play/${cycle.cycle_id}`);
  } catch (e: any) {
    error.value = e.message;
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <CycleList
      v-else
      :cycles="cycles"
      :world-name="worldId"
      @select="handleSelect"
      @delete="handleDelete"
    />
    <button
      class="btn-create"
      @click="handleCreate"
      :disabled="creating"
    >
      {{ creating ? '创建中...' : '+ 开新周目' }}
    </button>
  </div>
</template>

<style scoped>
.page { max-width: 700px; margin: 0 auto; }
.loading, .error { text-align: center; padding: 60px; color: #666; }
.error { color: #c0392b; }
.btn-create {
  display: block; margin: 16px auto; padding: 12px 32px;
  border-radius: 8px; border: none; background: #3498db;
  color: #fff; font-size: 15px; font-weight: 500; cursor: pointer;
  min-height: 44px; min-width: 44px; width: 100%;
}
.btn-create:hover { background: #2980b9; }
.btn-create:disabled { opacity: 0.6; }
</style>
