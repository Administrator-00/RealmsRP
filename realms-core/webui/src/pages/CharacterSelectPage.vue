<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { getWorld, listPerspectives } from '../api';
import NpcSelector, { type NpcInfo } from '../widgets/NpcSelector.vue';
import PerspectiveSwitcher, { type PerspectiveInfo } from '../widgets/PerspectiveSwitcher.vue';

const router = useRouter();
const route = useRoute();
const worldId = (route.query.world_id as string) || '';

const npcs = ref<NpcInfo[]>([]);
const perspectives = ref<PerspectiveInfo[]>([]);
const selectedNpc = ref('');
const selectedPerspective = ref('');
const loading = ref(true);
const error = ref('');

onMounted(async () => {
  if (!worldId) {
    error.value = '未选择世界，请先回到世界选择页';
    loading.value = false;
    return;
  }
  try {
    const [world, persps] = await Promise.all([
      getWorld(worldId),
      listPerspectives(),
    ]);
    npcs.value = Object.entries(world.npc_personas).map(([npcId, persona]) => ({
      npcId,
      name: npcId,
      description: persona.slice(0, 80),
      source: 'npc' as const,
    }));
    perspectives.value = persps
      .filter(p => p.world_id === worldId || p.source === 'oc')
      .map(p => ({
        id: p.id,
        name: p.name,
        source: p.source as 'npc' | 'oc',
        worldId: p.world_id,
      }));
  } catch (e: any) {
    error.value = e.message;
  } finally {
    loading.value = false;
  }
});

function handleNpcSelect(npcId: string) {
  selectedNpc.value = npcId;
  const perspectiveId = `pov_npc_${npcId}`;
  const existing = perspectives.value.find(p => p.id === perspectiveId);
  if (existing) {
    selectedPerspective.value = perspectiveId;
  }
  goToGm(perspectiveId);
}

function handlePerspectiveSelect(id: string) {
  selectedPerspective.value = id;
  goToGm(id);
}

function goToGm(perspectiveId: string) {
  router.push({
    path: '/gm',
    query: { world_id: worldId, perspective_id: perspectiveId },
  });
}

function createOc() {
  router.push({ path: '/oc/create', query: { world_id: worldId } });
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <template v-else>
      <NpcSelector
        v-model:selected="selectedNpc"
        :npcs="npcs"
        @select="handleNpcSelect"
      />

      <div v-if="perspectives.length" style="margin-top: 20px;">
        <PerspectiveSwitcher
          v-model:selected="selectedPerspective"
          :perspectives="perspectives"
          @select="handlePerspectiveSelect"
        />
      </div>

      <button class="btn-create-oc" @click="createOc">
        + 创建 OC
      </button>
    </template>
  </div>
</template>

<style scoped>
.page { max-width: 800px; margin: 0 auto; }
.loading, .error { text-align: center; padding: 60px; color: #666; }
.error { color: #c0392b; }
.btn-create-oc {
  display: block; margin: 16px auto; padding: 10px 24px;
  border-radius: 8px; border: 2px dashed #ccc; background: #fff;
  color: #666; font-size: 14px; cursor: pointer; min-height: 44px;
}
.btn-create-oc:hover { border-color: #3498db; color: #3498db; }
</style>
