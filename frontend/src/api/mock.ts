// =====================================================================
//  TEMPORÄRER MOCK-LAYER  —  NUR FÜR DEV OHNE BACKEND
// ---------------------------------------------------------------------
//  Solange Backend + Kanidm (PKCE) noch nicht laufen, liefern die
//  Services feste Test-Daten aus dieser Datei. Gesteuert über das
//  Env-Flag VITE_USE_MOCK (siehe .env.example).
//
//  >>> FÜR PRODUKTION: VITE_USE_MOCK=false setzen. Dann gehen alle
//      Services über apiClient an das echte Backend, und Login/Logout
//      laufen über den echten PKCE-Redirect zu Kanidm.
//  >>> Diese ganze Datei kann am Ende gelöscht werden, sobald das
//      Backend steht (zusammen mit den `if (USE_MOCK)`-Zweigen).
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

// Veränderbarer In-Memory-Zustand, damit Aktionen "wirken" (bis Reload).
const userState: UserInfo = {
  appname: 'TestSpieler',
  roles: ['user', 'admin'], // admin drin, damit die Admin-View testbar ist
  balance: 5000,
  loans_total_amount: 0,
  loans_taken: 0,
  loans_total_owed: 0,
  total_spent: 0,
  total_win: 0,
}

export function getMockUser(): UserInfo {
  return { ...userState }
}

export function mockTakeLoan(amount: number): LoanResponse {
  userState.balance += amount
  userState.loans_total_amount += amount
  userState.loans_taken += 1
  userState.loans_total_owed += amount
  return {
    balance: userState.balance,
    loans_total_amount: userState.loans_total_amount,
    loans_taken: userState.loans_taken,
    loans_total_owed: userState.loans_total_owed,
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

  userState.balance += amount_earned - stake
  userState.total_spent += stake
  userState.total_win += amount_earned
  return {
    reels,
    amount_earned,
    balance: userState.balance,
    total_spent: userState.total_spent,
    total_win: userState.total_win,
  }
}

const adminUsers: AdminUserRow[] = [
  { id: 1, appname: 'TestSpieler', balance: 5000 },
  { id: 2, appname: 'blabla', balance: 9092 },
  { id: 3, appname: 'highroller', balance: 250000 },
]

export function mockUserList(): AdminUserListResponse {
  return { users: adminUsers.map((u) => ({ ...u })) }
}

export function mockUpdateUser(req: AdminUpdateUserRequest): AdminUpdateUserResponse {
  const row = adminUsers.find((u) => u.id === req.id)
  if (!row) throw new Error(`Mock: User ${req.id} nicht gefunden`)
  row.appname = req.appname
  if (req.balance !== undefined) row.balance = req.balance
  return { appname: row.appname, balance: row.balance }
}