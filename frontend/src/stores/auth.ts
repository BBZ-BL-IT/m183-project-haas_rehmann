import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchUserInfo } from '@/api/user'
import { loginRedirect, logoutRedirect } from '@/api/auth'
import { toApiError } from '@/api/client'
import type { UserInfo } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserInfo | null>(null)
  const isLoading = ref(false)
  const hasCheckedSession = ref(false)

  const isAuthenticated = computed(() => user.value !== null)
  const isAdmin = computed(() => user.value?.roles.includes('admin') ?? false)

  // Probiert /user/info. Bei 401 ist kein Cookie/Session aktiv.
  async function checkSession(): Promise<void> {
    isLoading.value = true
    try {
      user.value = await fetchUserInfo()
    } catch (err) {
      const e = toApiError(err)
      if (e.status !== 401) {
        console.error('Session-Check fehlgeschlagen:', e)
      }
      user.value = null
    } finally {
      isLoading.value = false
      hasCheckedSession.value = true
    }
  }

  // Aktualisiert die User-Daten nach einer Aktion (z.B. nach /spin oder /loan).
  function patchUser(partial: Partial<UserInfo>): void {
    if (user.value) {
      user.value = { ...user.value, ...partial }
    }
  }

  function login(): void {
    loginRedirect()
  }

  function logout(): void {
    user.value = null
    logoutRedirect()
  }

  return {
    user,
    isLoading,
    hasCheckedSession,
    isAuthenticated,
    isAdmin,
    checkSession,
    patchUser,
    login,
    logout,
  }
})