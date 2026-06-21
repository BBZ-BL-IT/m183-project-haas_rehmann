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
// Der Client wählt nur den Einsatz – das Backend würfelt die Walzen und
// berechnet den Gewinn (Server ist die einzige Wahrheit; niemals dem Client
// die Walzen/Auszahlung anvertrauen).
export interface SpinRequest {
  stake_amount: number
}

// Das Backend liefert genau 3 Zahlen (eine Reihe). Sind alle gleich -> grosser
// Gewinn; genau zwei gleich -> Einsatz zurück; sonst verloren.
export interface SpinResponse {
  reels: number[] // genau 3 Zahlen (Symbole 1..7)
  amount_earned: number // Brutto-Gewinn (0 bei Niete)
  // Autoritative Werte nach dem Spin, damit das Frontend Server-Wahrheit
  // anzeigt statt lokal zu rechnen.
  balance: number
  total_spent: number
  total_win: number
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