<script setup lang="ts">
import { ref, computed } from 'vue';

export interface OcTemplateOption {
  templateId: string;
  name: string;
  affinity: number | null;
  trust: number | null;
  intimacy: number | null;
  typeLabel: string;
}

const props = defineProps<{
  npcs: { npcId: string; name: string }[];
  templates: OcTemplateOption[];
}>();

const emit = defineEmits<{
  create: [data: { name: string; background: string; personality: string; selections: Record<string, string> }];
}>();

// Step state
const step = ref(1);
const ocName = ref('');
const ocBackground = ref('');
const ocPersonality = ref('');
const selections = ref<Record<string, string>>({});

// Pre-fill default: first template
props.npcs.forEach(n => { selections.value[n.npcId] = props.templates[0]?.templateId || 'tpl_stranger'; });

const canNext = computed(() => {
  if (step === 1) return ocName.value.trim().length > 0;
  return true;
});

function next() { if (canNext.value && step < 3) step.value++; }
function prev() { if (step > 1) step.value--; }
function finish() {
  emit('create', {
    name: ocName.value,
    background: ocBackground.value,
    personality: ocPersonality.value,
    selections: { ...selections.value },
  });
}

function findTemplate(id: string): OcTemplateOption | undefined {
  return props.templates.find(t => t.templateId === id);
}
</script>

<template>
  <div class="oc-creator">
    <!-- progress bar -->
    <div class="oc-steps">
      <div class="oc-step" :class="{ active: step >= 1, done: step > 1 }">① 基本信息</div>
      <div class="oc-step" :class="{ active: step >= 2, done: step > 2 }">② 初始关系</div>
      <div class="oc-step" :class="{ active: step >= 3 }">③ 确认</div>
    </div>

    <!-- Step 1: OC info -->
    <div v-if="step === 1" class="oc-panel">
      <label class="oc-field">
        <span>OC 名称</span>
        <input v-model="ocName" placeholder="如：师兄" class="oc-input" />
      </label>
      <label class="oc-field">
        <span>背景</span>
        <textarea v-model="ocBackground" placeholder="天剑阁弟子，因师父遭人陷害而下山追凶…" class="oc-textarea" rows="3" />
      </label>
      <label class="oc-field">
        <span>性格</span>
        <textarea v-model="ocPersonality" placeholder="沉默寡言，剑术高超，内心柔软" class="oc-textarea" rows="2" />
      </label>
    </div>

    <!-- Step 2: Relationship templates -->
    <div v-if="step === 2" class="oc-panel">
      <div class="oc-section-title">为每个 NPC 选择初始关系</div>
      <div v-for="npc in npcs" :key="npc.npcId" class="oc-rel-row">
        <span class="oc-rel-npc">{{ npc.name }}</span>
        <select v-model="selections[npc.npcId]" class="oc-select">
          <option v-for="t in templates" :key="t.templateId" :value="t.templateId">{{ t.name }}</option>
        </select>
        <span class="oc-rel-preview" v-if="selections[npc.npcId]">
          {{ findTemplate(selections[npc.npcId])?.typeLabel || '-' }}
          <template v-if="findTemplate(selections[npc.npcId])?.affinity != null"> · 好感{{ (findTemplate(selections[npc.npcId])?.affinity ?? 0) >= 0 ? "+" : "" }}{{ findTemplate(selections[npc.npcId])?.affinity }}</template>
        </span>
      </div>
    </div>

    <!-- Step 3: Confirm -->
    <div v-if="step === 3" class="oc-panel">
      <div class="oc-section-title">确认 OC 信息</div>
      <div class="oc-summary">
        <div><strong>名称：</strong>{{ ocName }}</div>
        <div><strong>背景：</strong>{{ ocBackground || '（未填）' }}</div>
        <div><strong>性格：</strong>{{ ocPersonality || '（未填）' }}</div>
        <div class="oc-summary-rel">
          <strong>关系：</strong>
          <div v-for="npc in npcs" :key="npc.npcId">
            {{ npc.name }} → {{ findTemplate(selections[npc.npcId])?.name || '-' }}
          </div>
        </div>
      </div>
    </div>

    <!-- nav -->
    <div class="oc-nav">
      <button v-if="step > 1" @click="prev" class="oc-btn">上一步</button>
      <div class="oc-spacer" />
      <button v-if="step < 3" @click="next" :disabled="!canNext" class="oc-btn oc-btn-primary">下一步</button>
      <button v-if="step === 3" @click="finish" class="oc-btn oc-btn-primary">完成创建</button>
    </div>
  </div>
</template>

<style scoped>
.oc-creator { padding: 12px; max-width: 600px; margin: 0 auto; }
.oc-steps { display: flex; gap: 20px; margin-bottom: 16px; justify-content: center; }
.oc-step { font-size: 13px; color: #bbb; }
.oc-step.active { color: #3498db; font-weight: 600; }
.oc-step.done { color: #27ae60; }

.oc-panel { margin-bottom: 16px; }
.oc-field { display: block; margin-bottom: 12px; }
.oc-field span { display: block; font-size: 13px; color: #666; margin-bottom: 4px; }
.oc-input, .oc-textarea {
  width: 100%; border: 1px solid #ddd; border-radius: 8px; padding: 8px 10px;
  font-size: 14px; box-sizing: border-box;
}
.oc-textarea { resize: vertical; }

.oc-section-title { font-weight: 600; margin-bottom: 10px; color: #2c3e50; }

.oc-rel-row { display: flex; align-items: center; gap: 10px; margin-bottom: 8px; }
.oc-rel-npc { min-width: 70px; font-weight: 600; font-size: 14px; }
.oc-select { flex: 1; border: 1px solid #ddd; border-radius: 6px; padding: 6px 8px; font-size: 13px; }
.oc-rel-preview { font-size: 12px; color: #888; min-width: 120px; }

.oc-summary { font-size: 14px; line-height: 1.8; }
.oc-summary-rel { margin-top: 8px; }

.oc-nav { display: flex; gap: 10px; padding-top: 12px; border-top: 1px solid #eee; }
.oc-spacer { flex: 1; }
.oc-btn {
  padding: 8px 20px; border-radius: 8px; border: 1px solid #ddd;
  background: #fff; cursor: pointer; font-size: 14px;
}
.oc-btn-primary { background: #3498db; color: #fff; border-color: #3498db; }
.oc-btn:disabled { opacity: .5; cursor: not-allowed; }

@media (max-width: 767px) { .oc-creator { padding: 8px; } }
@media (min-width: 1024px) { .oc-creator { padding: 16px; } }
button { min-height: 44px; min-width: 44px; }
</style>
