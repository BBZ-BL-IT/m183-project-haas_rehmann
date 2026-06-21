import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { UserInfo, LoanResponse } from '@/types'
import { USE_MOCK, mockDelay, getMockUser, mockTakeLoan } from './mock'

export async function fetchUserInfo(): Promise<UserInfo> {
  if (USE_MOCK) {
    await mockDelay()
    return getMockUser()
  }
  const { data } = await apiClient.get<UserInfo>(endpoints.userInfo)
  return data
}

export async function takeLoan(amount: number): Promise<LoanResponse> {
  if (USE_MOCK) {
    await mockDelay()
    return mockTakeLoan(amount)
  }
  const { data } = await apiClient.post<LoanResponse>(endpoints.loan(amount))
  return data
}