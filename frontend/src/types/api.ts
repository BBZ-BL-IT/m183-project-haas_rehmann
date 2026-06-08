// Zentrale DTO-Typen für die Backend-Schnittstelle.
// Wenn das Backend Felder umbenennt, nur hier anpassen.

export type Role = 'user' | 'admin'

export interface UserInfo {
  appname: string
  roles: Role[]
  balance: number
  loans_total_amount: number
  loans_taken: number
  loans_total_owed: number
  total_spent: number
  total_win: number
}

// POST /spin
export interface SpinRequest {
  stake_amount: number
}

// Backend liefert ein 3x3 Raster zurück.
export type SpinPattern = number[][]

export interface SpinResponse {
  pattern: SpinPattern
  amount_earned: number
}

// POST /loan/{amount}  (amount steckt im URL-Pfad)
export interface LoanResponse {
  balance: number
  loans_total_amount: number
  loans_taken: number
  loans_total_owed: number
}

// GET /admin/userlist
export interface AdminUserRow {
  id: number
  appname: string
  balance: number
}

export interface AdminUserListResponse {
  users: AdminUserRow[]
}

// POST /admin/update/user
export interface AdminUpdateUserRequest {
  id: number
  appname: string
  balance?: number  // optional, das Projektantrag-Dokument sagt admin darf auch Guthaben anpassen
}

export interface AdminUpdateUserResponse {
  appname: string
  balance: number
}

// API-Fehlerformat. Falls dein Backend ein anderes Format liefert (z.B. `{ message }`), hier anpassen.
export interface ApiError {
  error: string
  message?: string
  status?: number
}