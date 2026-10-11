import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '@/features/home/HomeView.vue'
import SettingsView from '@/features/settings/SettingsView.vue'
import WellMapView from '@/features/wells/WellMapView.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', component: HomeView },
    { path: '/initial-placement', component: WellMapView },
    { path: '/well-map', component: WellMapView },
    { path: '/settings', component: SettingsView },
  ],
})
