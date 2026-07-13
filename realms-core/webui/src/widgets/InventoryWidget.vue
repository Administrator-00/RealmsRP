<script setup lang="ts">
import { computed } from 'vue';
import type { InventoryItem } from '../types';

const props = defineProps<{ items: InventoryItem[] }>();

const grouped = computed(() => {
  const map: Record<string, InventoryItem[]> = {};
  for (const item of props.items) {
    (map[item.type] ||= []).push(item);
  }
  return map;
});

const typeLabels: Record<string, string> = {
  weapon: '⚔️ 武器', potion: '🧪 药品', key: '🔑 关键',
  armor: '🛡️ 防具', book: '📖 书籍', treasure: '💎 宝物', food: '🍖 食物',
};
</script>

<template>
  <div class="inventory-widget">
    <div v-if="!items.length" class="empty">背包空空</div>
    <div v-for="(group, type) in grouped" :key="type" class="item-group">
      <div class="group-label">{{ typeLabels[type] || type }}</div>
      <div v-for="item in group" :key="item.id" class="item-row">
        <span class="item-name">{{ item.name }}</span>
        <span v-if="item.quantity > 1" class="item-qty">×{{ item.quantity }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inventory-widget { padding: 12px; font-size: 14px; }
.empty { color: #999; text-align: center; padding: 16px; }

.item-group { margin-bottom: 10px; }
.group-label { font-weight: 600; color: #555; margin-bottom: 4px; font-size: 13px; }

.item-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: 4px 8px; border-radius: 6px; background: #fafafa; margin: 2px 0;
}
.item-name { color: #333; }
.item-qty { color: #999; font-size: 12px; }

@media (max-width: 767px) {
  .inventory-widget { padding: 8px; }
}
@media (min-width: 1024px) {
  .inventory-widget { padding: 16px; }
}
button { min-height: 44px; min-width: 44px; }
</style>
