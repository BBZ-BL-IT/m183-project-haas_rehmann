import axios, { AxiosError } from 'axios'
import type { ApiError } from '@/types'

export const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  withCredentials: true, // Session-Cookie wird automatisch mitgeschickt
  headers: { 'Content-Type': 'application/json' },
})

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