<script setup lang="ts">
import { useRoute } from 'vue-router';

const route = useRoute();

const tabs = [
  { path: '/', label: '世界' },
  { path: '/character', label: '角色' },
  { path: '/gm', label: 'GM' },
  { path: '/cycles', label: '周目' },
  { path: '/settings', label: '设置' },
];

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/';
  return route.path.startsWith(path);
}
</script>

<template>
  <div class="app-shell">
    <nav class="top-tabs">
      <router-link
        v-for="tab in tabs"
        :key="tab.path"
        :to="tab.path"
        class="tab"
        :class="{ active: isActive(tab.path) }"
      >
        {{ tab.label }}
      </router-link>
    </nav>
    <main class="content">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f6fa;
}

.top-tabs {
  display: flex;
  gap: 0;
  background: #fff;
  border-bottom: 2px solid #eee;
  padding: 0 16px;
  overflow-x: auto;
}

.tab {
  padding: 14px 20px;
  text-decoration: none;
  color: #666;
  font-weight: 500;
  border-bottom: 3px solid transparent;
  transition: color 0.2s, border-color 0.2s;
  white-space: nowrap;
  min-height: 44px;
  display: flex;
  align-items: center;
}

.tab:hover {
  color: #3498db;
}

.tab.active {
  color: #3498db;
  border-bottom-color: #3498db;
}

.content {
  flex: 1;
  padding: 16px;
  max-width: 1200px;
  width: 100%;
  margin: 0 auto;
  box-sizing: border-box;
}

@media (max-width: 767px) {
  .top-tabs {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    border-bottom: none;
    border-top: 2px solid #eee;
    padding: 0;
    z-index: 100;
    justify-content: space-around;
  }

  .tab {
    padding: 10px 12px;
    font-size: 13px;
    border-bottom: none;
    border-top: 3px solid transparent;
  }

  .tab.active {
    border-bottom: none;
    border-top-color: #3498db;
  }

  .content {
    padding: 16px 8px;
    padding-bottom: 60px;
  }
}
</style>
