<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { listWorlds } from '../api';
import WorldSelector, { type WorldInfo } from '../widgets/WorldSelector.vue';

const router = useRouter();
const worlds = ref<WorldInfo[]>([]);
const selected = ref('');
const loading = ref(true);
const error = ref('');

onMounted(async () => {
  try {
    const data = await listWorlds();
    worlds.value = data.map(w => ({
      worldId: w.world_id,
      name: w.world_id,
      description: w.setting_preview,
      npcCount: w.npc_count,
    }));
  } catch (e: any) {
    error.value = e.message;
  } finally {
    loading.value = false;
  }
});

function handleSelect(worldId: string) {
  selected.value = worldId;
  router.push({ path: '/character', query: { world_id: worldId } });
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <WorldSelector
      v-else
      v-model:selected="selected"
      :worlds="worlds"
      @select="handleSelect"
    />
  </div>
</template>

<style scoped>
.page { max-width: 800px; margin: 0 auto; }
.loading, .error { text-align: center; padding: 60px; color: #666; }
.error { color: #c0392b; }
</style>
