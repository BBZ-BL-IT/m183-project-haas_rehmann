import { endpoints } from './endpoints'
import { USE_MOCK } from './mock'

// Login/logout are hard browser redirects (not XHR): the PKCE flow needs a real
// navigation to Kanidm and back.
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
