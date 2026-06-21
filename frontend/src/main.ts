import './assets/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import { setUnauthorizedHandler } from '@/api/client'
import { useAuthStore } from '@/stores'

const app = createApp(App)
app.use(createPinia())
app.use(router)

// On 401: clear local auth state and leave protected pages.
const auth = useAuthStore()
setUnauthorizedHandler(() => {
  auth.clearSession()
  if (router.currentRoute.value.meta.requiresAuth) {
    router.push({ name: 'home' })
  }
})

app.mount('#app')