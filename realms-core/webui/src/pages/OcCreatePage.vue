<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { getWorld, createPerspective } from '../api';
import OcCreator, { type OcTemplateOption } from '../widgets/OcCreator.vue';

const router = useRouter();
const route = useRoute();
const worldId = (route.query.world_id as string) || '';

const npcList = ref<{ npcId: string; name: string }[]>([]);
const loading = ref(true);
const error = ref('');

const templates: OcTemplateOption[] = [
  { templateId: 'tpl_stranger', name: '萍水相逢', affinity: 0, trust: 0, intimacy: 0, typeLabel: 'stranger' },
  { templateId: 'tpl_childhood_friend', name: '青梅竹马', affinity: 60, trust: 80, intimacy: 70, typeLabel: 'close_friend' },
  { templateId: 'tpl_love_at_first_sight', name: '一见钟情', affinity: 70, trust: 30, intimacy: 50, typeLabel: 'lover' },
  { templateId: 'tpl_sworn_enemy', name: '宿敌', affinity: -80, trust: 10, intimacy: 0, typeLabel: 'enemy' },
  { templateId: 'tpl_master_student', name: '师徒', affinity: 40, trust: 90, intimacy: 50, typeLabel: 'master' },
  { templateId: 'tpl_destined', name: '命中注定', affinity: 50, trust: 50, intimacy: 50, typeLabel: 'destined' },
  { templateId: 'tpl_no_relation', name: '无关系', affinity: null, trust: null, intimacy: null, typeLabel: 'none' },
];

onMounted(async () => {
  if (!worldId) {
    error.value = '未选择世界';
    loading.value = false;
    return;
  }
  try {
    const world = await getWorld(worldId);
    npcList.value = Object.keys(world.npc_personas).map(id => ({ npcId: id, name: id }));
  } catch (e: any) {
    error.value = e.message;
  } finally {
    loading.value = false;
  }
});

async function handleCreate(data: {
  name: string;
  background: string;
  personality: string;
  selections: Record<string, string>;
}) {
  const personaData = [
    data.background ? `# 背景\n${data.background}` : '',
    data.personality ? `# 性格\n${data.personality}` : '',
  ].filter(Boolean).join('\n\n');

  const id = `pov_oc_${Date.now()}`;
  try {
    await createPerspective({
      id,
      source: 'oc',
      name: data.name,
      persona_data: personaData,
      world_id: worldId || undefined,
    });
    router.push({
      path: '/gm',
      query: { world_id: worldId, perspective_id: id },
    });
  } catch (e: any) {
    error.value = e.message;
  }
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="loading">加载中...</div>
    <div v-else-if="error" class="error">{{ error }}</div>
    <OcCreator
      v-else
      :npcs="npcList"
      :templates="templates"
      @create="handleCreate"
    />
  </div>
</template>

<style scoped>
.page { max-width: 700px; margin: 0 auto; }
.loading, .error { text-align: center; padding: 60px; color: #666; }
.error { color: #c0392b; }
</style>
