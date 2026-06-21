import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores'
import HomeView from '@/views/HomeView.vue'
import PlayView from '@/views/PlayView.vue'
import RegisterView from '@/views/RegisterView.vue'
import AdminView from '@/views/admin/AdminView.vue'
import NotFoundView from '@/views/NotFoundView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/register',
      name: 'register',
      component: RegisterView,
    },
    {
      path: '/play',
      name: 'play',
      component: PlayView,
      meta: { requiresAuth: true },
    },
    {
      path: '/admin',
      name: 'admin',
      component: AdminView,
      meta: { requiresAuth: true, requiresAdmin: true },
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'not-found',
      component: NotFoundView,
    },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()

  // Session nur beim ersten Navigationsaufruf prüfen
  if (!auth.hasCheckedSession) {
    await auth.checkSession()
  }

  if (to.meta.requiresAuth && !auth.isAuthenticated) {
    return { name: 'home' }
  }

  if (to.meta.requiresAdmin && !auth.isAdmin) {
    return { name: 'home' }
  }
})

export default router