import axios, { AxiosError } from 'axios'
import type { ApiError } from '@/types'

export const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  withCredentials: true, // HttpOnly-Session-Cookie wird automatisch mitgeschickt
  headers: { 'Content-Type': 'application/json' },
})

// Wird in main.ts registriert (nachdem Pinia/Router stehen). So vermeiden wir
// einen zirkulären Import zwischen client.ts und dem Auth-Store.
let onUnauthorized: (() => void) | null = null
export function setUnauthorizedHandler(fn: () => void): void {
  onUnauthorized = fn
}

// Zentrales 401-Handling: Backend lehnt die Session ab (JWT abgelaufen und
// Renewal fehlgeschlagen) -> globalen Logout-Callback auslösen.
apiClient.interceptors.response.use(
  (res) => res,
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      onUnauthorized?.()
    }
    return Promise.reject(error)
  },
)

// Helfer um Axios-Fehler in ein einheitliches Format zu mappen
export function toApiError(err: unknown): ApiError {
  if (err instanceof AxiosError) {
    return {
      error: err.code ?? 'request_failed',
      message: err.response?.data?.message ?? err.message,
      status: err.response?.status,
    }
  }
  return { error: 'unknown', message: String(err) }
}