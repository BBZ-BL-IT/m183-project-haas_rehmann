// DTO types for the backend API. Keep in sync with backend/src/models.rs.

export type Role = 'user' | 'admin'

export interface UserInfo {
  username: string
  roles: Role[]
  balance: number
  total_spent: number
  total_profit: number
  highest_win_streak: number
  loans_taken: number
  loans_value: number
  loans_in_window: number
  loans_max: number
  loans_window_seconds: number
  loans_reset_at: string | null // RFC3339, or null when under the limit
}

// POST /spin
export interface SpinRequest {
  stake_amount: number
}

export interface SpinResponse {
  reels: number[]
  amount_earned: number
  balance: number
  total_spent: number
  total_profit: number
  highest_win_streak: number
}

// POST /loan/{amount}
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

// POST /admin/update/user (only provided fields change)
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

export interface ApiError {
  error: string
  message?: string
  status?: number
}
