import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { SpinRequest, SpinResponse } from '@/types'

export async function spin(req: SpinRequest): Promise<SpinResponse> {
  const { data } = await apiClient.post<SpinResponse>(endpoints.spin, req)
  return data
}