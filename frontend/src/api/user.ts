import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { UserInfo, LoanResponse } from '@/types'

export async function fetchUserInfo(): Promise<UserInfo> {
  const { data } = await apiClient.get<UserInfo>(endpoints.userInfo)
  return data
}

export async function takeLoan(amount: number): Promise<LoanResponse> {
  const { data } = await apiClient.post<LoanResponse>(endpoints.loan(amount))
  return data
}