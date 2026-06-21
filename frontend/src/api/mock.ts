// =====================================================================
//  TEMPORÄRER MOCK-LAYER  —  NUR FÜR DEV OHNE BACKEND
// ---------------------------------------------------------------------
//  Solange Backend + Kanidm (PKCE) noch nicht laufen, liefern die
//  Services feste Test-Daten aus dieser Datei. Gesteuert über das
//  Env-Flag VITE_USE_MOCK (siehe .env.example).
//
//  >>> FÜR PRODUKTION: VITE_USE_MOCK=false setzen.
// =====================================================================

import type {
  UserInfo,
  SpinResponse,
  LoanResponse,
  AdminUserRow,
  AdminUserListResponse,
  AdminUpdateUserRequest,
  AdminUpdateUserResponse,
} from '@/types'

export const USE_MOCK = import.meta.env.VITE_USE_MOCK === 'true'

// künstliche Latenz, damit Lade-Zustände (Spinner etc.) sichtbar werden
export const mockDelay = (ms = 400) => new Promise<void>((r) => setTimeout(r, ms))

const LOANS_MAX = 3
const LOANS_WINDOW = 86400

// Veränderbarer In-Memory-Zustand, damit Aktionen "wirken" (bis Reload).
const userState: UserInfo = {
  username: 'TestSpieler',
  roles: ['user', 'admin'], // admin drin, damit die Admin-View testbar ist
  balance: 5000,
  total_spent: 0,
  total_profit: 0,
  highest_win_streak: 0,
  loans_taken: 0,
  loans_value: 0,
  loans_in_window: 0,
  loans_max: LOANS_MAX,
  loans_window_seconds: LOANS_WINDOW,
  loans_reset_at: null,
}

// Mock-Win-Streak-Zähler.
let currentStreak = 0

export function getMockUser(): UserInfo {
  return { ...userState }
}

export function mockTakeLoan(amount: number): LoanResponse {
  userState.balance += amount
  userState.loans_value += amount
  userState.loans_taken += 1
  userState.loans_in_window += 1
  if (userState.loans_in_window >= userState.loans_max) {
    userState.loans_reset_at = new Date(Date.now() + LOANS_WINDOW * 1000).toISOString()
  }
  return {
    balance: userState.balance,
    loans_value: userState.loans_value,
    loans_taken: userState.loans_taken,
    loans_in_window: userState.loans_in_window,
    loans_max: userState.loans_max,
    loans_reset_at: userState.loans_reset_at,
  }
}

// Spiegelt die Backend-Logik (game.rs): 3 Walzen, gestaffelte Auszahlung.
const TRIPLE_MULTIPLIER: Record<number, number> = {
  7: 50,
  6: 25,
  5: 15,
  4: 10,
  3: 8,
  2: 6,
  1: 5,
}

export function mockSpin(stake: number): SpinResponse {
  const reels = Array.from({ length: 3 }, () => Math.floor(Math.random() * 7) + 1)
  const [a = 0, b = 0, c = 0] = reels

  let amount_earned = 0
  if (a === b && b === c) {
    amount_earned = stake * (TRIPLE_MULTIPLIER[a] ?? 5) // drei Gleiche
  } else if (a === b || b === c || a === c) {
    amount_earned = stake // genau zwei Gleiche -> Einsatz zurück
  }

  // Win-Streak: netto-positiver Spin zählt.
  if (amount_earned > stake) {
    currentStreak += 1
    if (currentStreak > userState.highest_win_streak) userState.highest_win_streak = currentStreak
  } else {
    currentStreak = 0
  }

  userState.balance += amount_earned - stake
  userState.total_spent += stake
  userState.total_profit += amount_earned - stake
  return {
    reels,
    amount_earned,
    balance: userState.balance,
    total_spent: userState.total_spent,
    total_profit: userState.total_profit,
    highest_win_streak: userState.highest_win_streak,
  }
}

let adminUsers: AdminUserRow[] = [
  { id: 1, username: 'TestSpieler', balance: 5000, loans_value: 0, loans_taken: 0 },
  { id: 2, username: 'blabla', balance: 9092, loans_value: 1000, loans_taken: 2 },
  { id: 3, username: 'highroller', balance: 250000, loans_value: 50000, loans_taken: 9 },
]

export function mockUserList(): AdminUserListResponse {
  return { users: adminUsers.map((u) => ({ ...u })) }
}

export function mockUpdateUser(req: AdminUpdateUserRequest): AdminUpdateUserResponse {
  const row = adminUsers.find((u) => u.id === req.id)
  if (!row) throw new Error(`Mock: User ${req.id} nicht gefunden`)
  if (req.username !== undefined) row.username = req.username
  if (req.balance !== undefined) row.balance = req.balance
  if (req.loans_value !== undefined) row.loans_value = req.loans_value
  if (req.loans_taken !== undefined) row.loans_taken = req.loans_taken
  return { ...row }
}

export function mockDeleteUser(id: number): void {
  adminUsers = adminUsers.filter((u) => u.id !== id)
}
