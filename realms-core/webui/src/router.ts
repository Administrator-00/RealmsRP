import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    component: () => import('./pages/WorldSelectPage.vue'),
  },
  {
    path: '/settings',
    component: () => import('./pages/SettingsPage.vue'),
  },
  {
    path: '/character',
    component: () => import('./pages/CharacterSelectPage.vue'),
  },
  {
    path: '/oc/create',
    component: () => import('./pages/OcCreatePage.vue'),
  },
  {
    path: '/gm',
    component: () => import('./pages/GmSelectPage.vue'),
  },
  {
    path: '/cycles',
    component: () => import('./pages/CycleListPage.vue'),
  },
  {
    path: '/play/:cycleId',
    component: () => import('./pages/GamePlayPage.vue'),
  },
];

export default createRouter({
  history: createWebHistory(),
  routes,
});
