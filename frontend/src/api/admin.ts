import { apiClient } from './client'
import { endpoints } from './endpoints'
import type {
  AdminUserListResponse,
  AdminUpdateUserRequest,
  AdminUpdateUserResponse,
} from '@/types'

export async function fetchUserList(): Promise<AdminUserListResponse> {
  const { data } = await apiClient.get<AdminUserListResponse>(endpoints.adminUserList)
  return data
}

export async function updateUser(
  req: AdminUpdateUserRequest,
): Promise<AdminUpdateUserResponse> {
  const { data } = await apiClient.post<AdminUpdateUserResponse>(
    endpoints.adminUpdateUser,
    req,
  )
  return data
}