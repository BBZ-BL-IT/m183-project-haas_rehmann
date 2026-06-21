import { apiClient } from './client'
import { endpoints } from './endpoints'
import type { RegisterRequest, RegisterResponse } from '@/types'
import { USE_MOCK, mockDelay, mockRegister } from './mock'

// Registriert einen neuen Account. Das Backend legt die Kanidm-Person an und
// liefert einen Credential-Reset-Link zurück, über den der/die Neue ein
// Passwort setzt – danach kann normal eingeloggt werden.
export async function register(req: RegisterRequest): Promise<RegisterResponse> {
  // --- TEMPORÄRE TEST-DATEN ---
  if (USE_MOCK) {
    await mockDelay()
    return mockRegister(req)
  }
  // --- PRODUKTION ---
  const { data } = await apiClient.post<RegisterResponse>(endpoints.register, req)
  return data
}
