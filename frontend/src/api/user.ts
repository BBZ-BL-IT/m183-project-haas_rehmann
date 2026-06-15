import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { UserInfo, LoanResponse } from '@/types'
import { USE_MOCK, mockDelay, getMockUser, mockTakeLoan } from './mock'

export async function fetchUserInfo(): Promise<UserInfo> {
  // --- TEMPORÄRE TEST-DATEN (kein Backend nötig) ---
  if (USE_MOCK) {
    await mockDelay()
    return getMockUser()
  }
  // --- PRODUKTION: echter Backend-Call.
  //     Das HttpOnly-Session-Cookie geht dank withCredentials automatisch mit;
  //     das Frontend kann den Cookie/JWT NICHT lesen, deshalb dieser dedizierte
  //     Endpoint, der die User-Daten zurückgibt. ---
  const { data } = await apiClient.get<UserInfo>(endpoints.userInfo)
  return data
}

export async function takeLoan(amount: number): Promise<LoanResponse> {
  // --- TEMPORÄRE TEST-DATEN ---
  if (USE_MOCK) {
    await mockDelay()
    return mockTakeLoan(amount)
  }
  // --- PRODUKTION ---
  const { data } = await apiClient.post<LoanResponse>(endpoints.loan(amount))
  return data
}