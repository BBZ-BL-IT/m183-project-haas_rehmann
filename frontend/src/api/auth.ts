import { endpoints } from './endpoints'
import { USE_MOCK } from './mock'

// Login/Logout laufen über echte Browser-Redirects (kein XHR), weil der
// PKCE-Flow eine harte Navigation zu Kanidm und zurück braucht.
export function loginRedirect(): void {
   if (USE_MOCK) {
    window.location.reload()
    return
  }
   window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.login}`
}

export function logoutRedirect(): void {
   if (USE_MOCK) {
    window.location.href = '/'
    return
  }
   window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.logout}`
}