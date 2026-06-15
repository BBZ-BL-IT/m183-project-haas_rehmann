import { endpoints } from './endpoints'
import { USE_MOCK } from './mock'

// Login/Logout laufen über echte Browser-Redirects (kein XHR), weil der
// PKCE-Flow eine harte Navigation zu Kanidm und zurück braucht.
export function loginRedirect(): void {
  // --- TEMPORÄR (Mock): es gibt keinen IDP. Wir laden nur neu; checkSession()
  //     liefert dann den Mock-User, man ist "eingeloggt". ---
  if (USE_MOCK) {
    window.location.reload()
    return
  }
  // --- PRODUKTION: voller PKCE-Flow.
  //     Browser navigiert hart zum Backend-Endpoint /login. Das Backend
  //     erzeugt code_verifier + code_challenge (PKCE) sowie state, speichert
  //     sie serverseitig und leitet zu Kanidm weiter. Nach erfolgreichem
  //     Login redirected Kanidm zurück zum Backend-/callback; das Backend
  //     tauscht den Authorization-Code gegen Tokens, setzt das HttpOnly-
  //     Session-Cookie (JWT) und schickt den Browser zurück in die App. ---
  window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.login}`
}

export function logoutRedirect(): void {
  // --- TEMPORÄR (Mock): einfach zur Startseite ---
  if (USE_MOCK) {
    window.location.href = '/'
    return
  }
  // --- PRODUKTION: Backend löscht das HttpOnly-Cookie / beendet die Session
  //     und (optional) die Kanidm-Session. ---
  window.location.href = `${import.meta.env.VITE_API_BASE_URL}${endpoints.logout}`
}