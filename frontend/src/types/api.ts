// Zentrale DTO-Typen für die Backend-Schnittstelle.
// Wenn das Backend Felder umbenennt, nur hier anpassen.

export type Role = 'user' | 'admin'

export interface UserInfo {
  username: string
  roles: Role[]
  balance: number
  // --- Stats ---
  total_spent: number
  total_profit: number // kann negativ sein
  highest_win_streak: number
  loans_taken: number // Lebenszeit-Anzahl
  loans_value: number // offener/geschuldeter Betrag
  // --- Kredit-Limit ---
  loans_in_window: number // Anzahl im aktuellen Fenster
  loans_max: number
  loans_window_seconds: number
  loans_reset_at: string | null // RFC3339; wann der nächste Slot frei wird (null = unter Limit)
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
  // Autoritative Werte nach dem Spin.
  balance: number
  total_spent: number
  total_profit: number
  highest_win_streak: number
}

// POST /loan/{amount}  (amount steckt im URL-Pfad)
export interface LoanResponse {
  balance: number
  loans_value: number
  loans_taken: number
  loans_in_window: number
  loans_max: number
  loans_reset_at: string | null
}

// GET /admin/userlist
export interface AdminUserRow {
  id: number
  username: string
  balance: number
  loans_value: number
  loans_taken: number
}

export interface AdminUserListResponse {
  users: AdminUserRow[]
}

// POST /admin/update/user  (nur gesetzte Felder werden geändert)
export interface AdminUpdateUserRequest {
  id: number
  username?: string
  balance?: number
  loans_value?: number
  loans_taken?: number
}

export interface AdminUpdateUserResponse {
  id: number
  username: string
  balance: number
  loans_value: number
  loans_taken: number
}

// API-Fehlerformat. Falls dein Backend ein anderes Format liefert (z.B. `{ message }`), hier anpassen.
export interface ApiError {
  error: string
  message?: string
  status?: number
}
