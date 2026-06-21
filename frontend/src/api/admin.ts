import { apiClient } from './client'
import { endpoints } from './endpoints'
import type {
  AdminUserListResponse,
  AdminUpdateUserRequest,
  AdminUpdateUserResponse,
} from '@/types'
import { USE_MOCK, mockDelay, mockUserList, mockUpdateUser, mockDeleteUser } from './mock'

export async function fetchUserList(): Promise<AdminUserListResponse> {
  // --- TEMPORÄRE TEST-DATEN ---
  if (USE_MOCK) {
    await mockDelay()
    return mockUserList()
  }
  // --- PRODUKTION (Backend prüft serverseitig die admin-Rolle) ---
  const { data } = await apiClient.get<AdminUserListResponse>(endpoints.adminUserList)
  return data
}

export async function updateUser(
  req: AdminUpdateUserRequest,
): Promise<AdminUpdateUserResponse> {
  // --- TEMPORÄRE TEST-DATEN ---
  if (USE_MOCK) {
    await mockDelay()
    return mockUpdateUser(req)
  }
  // --- PRODUKTION ---
  const { data } = await apiClient.post<AdminUpdateUserResponse>(
    endpoints.adminUpdateUser,
    req,
  )
  return data
}

export async function deleteUser(id: number): Promise<void> {
  // --- TEMPORÄRE TEST-DATEN ---
  if (USE_MOCK) {
    await mockDelay()
    mockDeleteUser(id)
    return
  }
  // --- PRODUKTION ---
  await apiClient.post(endpoints.adminDeleteUser(id))
}