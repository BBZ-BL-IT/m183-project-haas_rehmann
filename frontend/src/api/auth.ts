import { endpoints } from './endpoints'

// Session-Cookie-Flow: Browser navigiert direkt zu /login auf dem Backend.
// Backend macht PKCE mit Kanidm und redirected danach zurück mit gesetztem Cookie.
export function loginRedirect(): void {
  window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.login}`
}

export function logoutRedirect(): void {
  window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.logout}`
}