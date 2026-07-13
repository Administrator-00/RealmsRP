<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue';

interface ChatMessage {
  id: string;
  role: 'user' | 'npc' | 'system';
  senderName: string;
  content: string;
  timestamp?: string;
}

const props = defineProps<{
  messages: ChatMessage[];
  npcNames?: Record<string, string>;
}>();

const emit = defineEmits<{
  send: [text: string];
}>();

const inputText = ref('');
const scrollEl = ref<HTMLElement | null>(null);

function scrollBottom() {
  nextTick(() => {
    const el = scrollEl.value;
    if (el) el.scrollTop = el.scrollHeight;
  });
}

onMounted(() => scrollBottom());

function handleSend() {
  const text = inputText.value.trim();
  if (!text) return;
  emit('send', text);
  inputText.value = '';
  scrollBottom();
}

function senderLabel(msg: ChatMessage): string {
  if (msg.role === 'user') return '你';
  if (msg.role === 'system') return '📢';
  return props.npcNames?.[msg.senderName] || msg.senderName;
}
</script>

<template>
  <div class="chat-widget">
    <!-- message list -->
    <div ref="scrollEl" class="chat-messages">
      <div v-if="!messages.length" class="chat-empty">等待对话...</div>
      <div
        v-for="msg in messages" :key="msg.id"
        class="chat-msg" :class="'msg-' + msg.role"
      >
        <div class="msg-sender">{{ senderLabel(msg) }}</div>
        <div class="msg-content">{{ msg.content }}</div>
      </div>
    </div>
    <!-- input -->
    <form class="chat-input-row" @submit.prevent="handleSend">
      <input
        v-model="inputText"
        class="chat-input"
        placeholder="输入消息..."
        aria-label="消息输入"
      />
      <button type="submit" class="chat-send-btn">发送</button>
    </form>
  </div>
</template>

<style scoped>
.chat-widget { display: flex; flex-direction: column; height: 100%; }
.chat-empty { color: #999; text-align: center; padding: 32px 0; }

.chat-messages {
  flex: 1; overflow-y: auto; padding: 12px;
  display: flex; flex-direction: column; gap: 8px;
}

.chat-msg { padding: 8px 12px; border-radius: 10px; max-width: 85%; }
.msg-user { align-self: flex-end; background: #dbeafe; }
.msg-npc { align-self: flex-start; background: #f0f0f0; }
.msg-system { align-self: center; background: #fff3cd; font-size: 12px; color: #856404; }

.msg-sender { font-size: 11px; color: #888; margin-bottom: 2px; }
.msg-content { line-height: 1.5; white-space: pre-wrap; }

.chat-input-row { display: flex; gap: 8px; padding: 8px 12px; border-top: 1px solid #eee; }
.chat-input { flex: 1; border: 1px solid #ddd; border-radius: 8px; padding: 8px 12px; font-size: 14px; }
.chat-send-btn {
  background: #3498db; color: #fff; border: none; border-radius: 8px;
  padding: 8px 16px; font-size: 14px; cursor: pointer;
}

@media (max-width: 767px) {
  .chat-messages { padding: 8px; }
  .chat-input { font-size: 16px; } /* 防 iOS zoom */
}
@media (min-width: 1024px) {
  .chat-msg { max-width: 65%; }
}
button { min-height: 44px; min-width: 44px; }
</style>
