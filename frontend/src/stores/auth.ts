import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { fetchUserInfo } from '@/api/user'
import { loginRedirect, logoutRedirect } from '@/api/auth'
import { toApiError } from '@/api/client'
import { USE_MOCK, getMockUser } from '@/api/mock'
import type { UserInfo } from '@/types'

// Merkt sich im Mock-Modus den Login, damit ein Reload eingeloggt bleibt.
const MOCK_SESSION_KEY = 'mock_logged_in'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserInfo | null>(null)
  const isLoading = ref(false)
  const hasCheckedSession = ref(false)

  const isAuthenticated = computed(() => user.value !== null)
  const isAdmin = computed(() => user.value?.roles.includes('admin') ?? false)

async function checkSession(): Promise<void> {
    // --- TEMPORAER (Mock): keine Backend-Session. Login-Status kommt aus
    //     sessionStorage, damit ein Reload eingeloggt bleibt. ---
    if (USE_MOCK) {
      user.value = sessionStorage.getItem(MOCK_SESSION_KEY) ? getMockUser() : null
      hasCheckedSession.value = true
      return
    }
    // --- PRODUKTION: probiert /user/info. Bei 401 ist keine Session aktiv. ---
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

  // Lokalen Auth-Zustand leeren (z.B. wenn das Backend mit 401 antwortet).
  function clearSession(): void {
    if (USE_MOCK) sessionStorage.removeItem(MOCK_SESSION_KEY)
    user.value = null
    hasCheckedSession.value = true
  }

  function login(): void {
    // --- TEMPORAER (Mock): direkt mit vorgegebenen Daten einloggen.
    //     Kein Redirect, kein Request. ---
    if (USE_MOCK) {
      sessionStorage.setItem(MOCK_SESSION_KEY, '1')
      user.value = getMockUser()
      hasCheckedSession.value = true
      return
    }
    // --- PRODUKTION: PKCE-Redirect zum Backend/IDP. ---
    loginRedirect()
  }

  function logout(): void {
    // --- TEMPORAER (Mock): nur lokalen Zustand leeren. ---
    if (USE_MOCK) {
      sessionStorage.removeItem(MOCK_SESSION_KEY)
      user.value = null
      return
    }
    // --- PRODUKTION: Backend beendet Session / loescht Cookie. ---
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