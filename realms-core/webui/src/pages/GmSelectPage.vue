<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { listGms } from '../api';
import GmSwitcher, { type GmTemplate } from '../widgets/GmSwitcher.vue';

const router = useRouter();
const route = useRoute();
const worldId = (route.query.world_id as string) || '';
const perspectiveId = (route.query.perspective_id as string) || '';

const gms = ref<GmTemplate[]>([]);
const selected = ref('');
const loading = ref(true);
const error = ref('');

onMounted(async () => {
  if (!worldId) {
    error.value = '未选择世界';
    loading.value = false;
    return;
  }
  try {
    const data = await listGms(worldId);
    gms.value = data.map(g => ({
      gmId: g.gm_id,
      name: g.gm_id,
      preview: g.content_preview,
    }));
  } catch (e: any) {
    error.value = e.message;
  } finally {
    loading.value = false;
  }
});

function handleSelect(gmId: string) {
  selected.value = gmId;
  router.push({
    path: '/cycles',
    query: { world_id: worldId, perspective_id: perspectiveId, gm_id: gmId },
  });
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <GmSwitcher
      v-else
      v-model:selected="selected"
      :gms="gms"
      @select="handleSelect"
    />
  </div>
</template>

<style scoped>
.page { max-width: 800px; margin: 0 auto; }
.loading, .error { text-align: center; padding: 60px; color: #666; }
.error { color: #c0392b; }
</style>
