import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchUserInfo } from '@/api/user'
import { loginRedirect, logoutRedirect } from '@/api/auth'
import { toApiError } from '@/api/client'
import { USE_MOCK, getMockUser } from '@/api/mock'
import type { UserInfo } from '@/types'

const MOCK_SESSION_KEY = 'mock_logged_in'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserInfo | null>(null)
  const isLoading = ref(false)
  const hasCheckedSession = ref(false)

  const isAuthenticated = computed(() => user.value !== null)
  const isAdmin = computed(() => user.value?.roles.includes('admin') ?? false)

  async function checkSession(): Promise<void> {
    if (USE_MOCK) {
      user.value = sessionStorage.getItem(MOCK_SESSION_KEY) ? getMockUser() : null
      hasCheckedSession.value = true
      return
    }
    // Probe /user/info; a 401 means there's no active session.
    isLoading.value = true
    try {
      user.value = await fetchUserInfo()
    } catch (err) {
      const e = toApiError(err)
      if (e.status !== 401) {
        console.error('Session check failed:', e)
      }
      user.value = null
    } finally {
      isLoading.value = false
      hasCheckedSession.value = true
    }
  }

  // Merge fresh fields into the user after an action (e.g. /spin or /loan).
  function patchUser(partial: Partial<UserInfo>): void {
    if (user.value) {
      user.value = { ...user.value, ...partial }
    }
  }

  function clearSession(): void {
    if (USE_MOCK) sessionStorage.removeItem(MOCK_SESSION_KEY)
    user.value = null
    hasCheckedSession.value = true
  }

  function login(): void {
    if (USE_MOCK) {
      sessionStorage.setItem(MOCK_SESSION_KEY, '1')
      user.value = getMockUser()
      hasCheckedSession.value = true
      return
    }
    loginRedirect()
  }

  function logout(): void {
    if (USE_MOCK) {
      sessionStorage.removeItem(MOCK_SESSION_KEY)
      user.value = null
      return
    }
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
    clearSession,
    login,
    logout,
  }
})
