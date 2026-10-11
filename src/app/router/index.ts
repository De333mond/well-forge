import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/features/home/HomeView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [{ path: '/', component: HomeView }],
})
