import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from '@/app/App.vue'
import { router } from '@/app/router'
import { useThemeStore } from '@/stores/theme'
import '@/styles/globals.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
useThemeStore(pinia)
app.mount('#app')
