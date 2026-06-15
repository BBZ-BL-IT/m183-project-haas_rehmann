/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL: string
  readonly VITE_OAUTH_LOGIN_PATH: string
  readonly VITE_OAUTH_LOGOUT_PATH: string
  readonly VITE_USE_MOCK: string // 'true' => Services liefern Test-Daten statt Backend-Calls
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}