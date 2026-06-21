import axios, { AxiosError } from 'axios'
import type { ApiError } from '@/types'

export const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  withCredentials: true, // send the HttpOnly session cookie
  headers: { 'Content-Type': 'application/json' },
})

// Registered from main.ts to avoid a circular import with the auth store.
let onUnauthorized: (() => void) | null = null
export function setUnauthorizedHandler(fn: () => void): void {
  onUnauthorized = fn
}

// On 401 the session is gone → trigger the global logout callback.
apiClient.interceptors.response.use(
  (res) => res,
  (error: AxiosError) => {
    if (error.response?.status === 401) {
      onUnauthorized?.()
    }
    return Promise.reject(error)
  },
)

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
