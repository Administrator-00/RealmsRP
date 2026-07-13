<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getConfig, updateConfig, testConfig, type AppConfig } from '../api';

const config = ref<AppConfig | null>(null);
const form = ref({ provider: 'OpenAI', endpoint: '', api_key: '', model: '' });
const status = ref<'idle' | 'testing' | 'saving' | 'ok' | 'error'>('idle');
const statusMsg = ref('');

onMounted(async () => {
  try {
    config.value = await getConfig();
    form.value = {
      provider: config.value.llm.provider,
      endpoint: config.value.llm.endpoint,
      api_key: config.value.llm.api_key,
      model: config.value.llm.model,
    };
  } catch (e: any) {
    status.value = 'error';
    statusMsg.value = e.message;
  }
});

async function handleSave() {
  status.value = 'saving';
  try {
    const updated = await updateConfig({ llm: { ...form.value } });
    config.value = updated;
    status.value = 'ok';
    statusMsg.value = '配置已保存';
  } catch (e: any) {
    status.value = 'error';
    statusMsg.value = e.message;
  }
}

async function handleTest() {
  status.value = 'testing';
  statusMsg.value = '测试连接中...';
  try {
    const result = await testConfig();
    status.value = result.status === 'ok' ? 'ok' : 'error';
    statusMsg.value = result.message;
  } catch (e: any) {
    status.value = 'error';
    statusMsg.value = e.message;
  }
}
</script>

<template>
  <div class="settings-page">
    <h2>设置</h2>

    <div class="section">
      <h3>LLM Provider</h3>
      <div class="form-group">
        <label>Provider</label>
        <select v-model="form.provider" class="input">
          <option value="OpenAI">OpenAI</option>
          <option value="Anthropic">Anthropic</option>
        </select>
      </div>
      <div class="form-group">
        <label>Endpoint</label>
        <input v-model="form.endpoint" type="url" class="input" placeholder="https://api.deepseek.com/v1/chat/completions" />
      </div>
      <div class="form-group">
        <label>API Key</label>
        <input v-model="form.api_key" type="password" class="input" placeholder="sk-..." />
      </div>
      <div class="form-group">
        <label>Model</label>
        <input v-model="form.model" class="input" placeholder="deepseek-chat" />
      </div>

      <div class="button-row">
        <button class="btn btn-secondary" @click="handleTest" :disabled="status === 'testing'">
          测试连接
        </button>
        <button class="btn btn-primary" @click="handleSave" :disabled="status === 'saving'">
          保存配置
        </button>
      </div>

      <div v-if="statusMsg" class="status" :class="status">
        <span v-if="status === 'testing' || status === 'saving'" class="spinner" />
        {{ status === 'ok' ? '✅' : status === 'error' ? '❌' : '⏳' }}
        {{ statusMsg }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page { max-width: 560px; }
h2 { margin-bottom: 20px; color: #2c3e50; }
h3 { margin-bottom: 12px; color: #555; font-size: 15px; }

.section {
  background: #fff;
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 16px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}

.form-group {
  margin-bottom: 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 13px;
  color: #888;
  font-weight: 500;
}

.input {
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.input:focus {
  outline: none;
  border-color: #3498db;
}

.button-row {
  display: flex;
  gap: 10px;
  margin-top: 16px;
}

.btn {
  padding: 10px 20px;
  border-radius: 8px;
  border: none;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  min-height: 44px;
  min-width: 44px;
  transition: background 0.2s;
}

.btn-primary {
  background: #3498db;
  color: #fff;
}
.btn-primary:hover { background: #2980b9; }
.btn-primary:disabled { opacity: 0.6; }

.btn-secondary {
  background: #eee;
  color: #333;
}
.btn-secondary:hover { background: #ddd; }
.btn-secondary:disabled { opacity: 0.6; }

.status {
  margin-top: 14px;
  padding: 10px;
  border-radius: 8px;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.status.ok { background: #e8f8e8; color: #2a6e2a; }
.status.error { background: #ffe8e8; color: #8b0000; }
.status.idle { display: none; }
.status.testing, .status.saving { background: #e8f0ff; color: #2c3e50; }

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid #ccc;
  border-top-color: #3498db;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
