import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { SpinRequest, SpinResponse } from '@/types'
import { USE_MOCK, mockDelay, mockSpin } from './mock'

export async function spin(req: SpinRequest): Promise<SpinResponse> {
  if (USE_MOCK) {
    await mockDelay(200)
    return mockSpin(req.stake_amount)
  }
  const { data } = await apiClient.post<SpinResponse>(endpoints.spin, req)
  return data
}