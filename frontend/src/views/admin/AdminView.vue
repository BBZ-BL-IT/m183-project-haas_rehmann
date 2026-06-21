<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { fetchUserList, updateUser, deleteUser } from '@/api/admin'
import { toApiError } from '@/api/client'
import type { AdminUserRow } from '@/types'

const users = ref<AdminUserRow[]>([])
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)

const editId = ref<number | null>(null)
const editUsername = ref('')
const editBalance = ref(0)
const editLoansValue = ref(0)
const editLoansTaken = ref(0)
const isSaving = ref(false)
const busyId = ref<number | null>(null)

async function load(): Promise<void> {
  isLoading.value = true
  errorMsg.value = null
  try {
    const res = await fetchUserList()
    users.value = res.users
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Laden fehlgeschlagen'
  } finally {
    isLoading.value = false
  }
}

function startEdit(u: AdminUserRow): void {
  editId.value = u.id
  editUsername.value = u.username
  editBalance.value = u.balance
  editLoansValue.value = u.loans_value
  editLoansTaken.value = u.loans_taken
}

function cancelEdit(): void {
  editId.value = null
}

async function save(): Promise<void> {
  if (editId.value === null) return
  isSaving.value = true
  errorMsg.value = null
  try {
    const res = await updateUser({
      id: editId.value,
      username: editUsername.value,
      balance: editBalance.value,
      loans_value: editLoansValue.value,
      loans_taken: editLoansTaken.value,
    })
    const row = users.value.find((u) => u.id === editId.value)
    if (row) {
      row.username = res.username
      row.balance = res.balance
      row.loans_value = res.loans_value
      row.loans_taken = res.loans_taken
    }
    editId.value = null
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Speichern fehlgeschlagen'
  } finally {
    isSaving.value = false
  }
}

async function removeUser(u: AdminUserRow): Promise<void> {
  if (!window.confirm(`Benutzer "${u.username}" wirklich löschen?`)) return
  busyId.value = u.id
  errorMsg.value = null
  try {
    await deleteUser(u.id)
    users.value = users.value.filter((x) => x.id !== u.id)
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Löschen fehlgeschlagen'
  } finally {
    busyId.value = null
  }
}

onMounted(load)
</script>

<template>
  <section class="admin">
    <h1>Admin · Benutzerverwaltung</h1>

    <div v-if="errorMsg" class="error">{{ errorMsg }}</div>
    <div v-if="isLoading" class="info">Lade Benutzer …</div>

    <table v-else class="user-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Username</th>
          <th>Guthaben</th>
          <th>Offene Kredite</th>
          <th>Kredite total</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in users" :key="u.id">
          <td>{{ u.id }}</td>

          <template v-if="editId === u.id">
            <td><input v-model="editUsername" class="cell-input" /></td>
            <td><input v-model.number="editBalance" type="number" class="cell-input" /></td>
            <td><input v-model.number="editLoansValue" type="number" class="cell-input" /></td>
            <td><input v-model.number="editLoansTaken" type="number" class="cell-input" /></td>
            <td class="actions">
              <button class="btn primary" :disabled="isSaving" @click="save">
                {{ isSaving ? '…' : 'Speichern' }}
              </button>
              <button class="btn" :disabled="isSaving" @click="cancelEdit">Abbrechen</button>
            </td>
          </template>

          <template v-else>
            <td>{{ u.username }}</td>
            <td>{{ u.balance }}</td>
            <td>{{ u.loans_value }}</td>
            <td>{{ u.loans_taken }}</td>
            <td class="actions">
              <button class="btn" @click="startEdit(u)">Bearbeiten</button>
              <button class="btn danger" :disabled="busyId === u.id" @click="removeUser(u)">
                {{ busyId === u.id ? '…' : 'Löschen' }}
              </button>
            </td>
          </template>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<style scoped>
.admin {
  max-width: 760px;
  margin: 2rem auto;
  padding: 0 1.5rem;
}
h1 {
  margin-bottom: 1.5rem;
}
.error {
  color: #e74c3c;
  margin-bottom: 1rem;
}
.info {
  color: #b8b9bd;
}
.user-table {
  width: 100%;
  border-collapse: collapse;
  background: #14171f;
  border: 1px solid #23262e;
  border-radius: 12px;
  overflow: hidden;
}
th,
td {
  text-align: left;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #23262e;
}
th {
  color: #8a8b91;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
tbody tr:last-child td {
  border-bottom: none;
}
.cell-input {
  background: #0b0d12;
  border: 1px solid #2e323c;
  color: #e8e8ea;
  padding: 0.4rem 0.6rem;
  border-radius: 6px;
  font: inherit;
  width: 100%;
  max-width: 160px;
}
.actions {
  display: flex;
  gap: 0.5rem;
}
.btn {
  background: #23262e;
  color: #e8e8ea;
  border: 1px solid #2e323c;
  padding: 0.4rem 0.9rem;
  border-radius: 6px;
  cursor: pointer;
  font: inherit;
}
.btn:hover {
  background: #2e323c;
}
.btn.primary {
  background: #c9a227;
  border-color: #c9a227;
  color: #0b0d12;
  font-weight: 600;
}
.btn.danger {
  border-color: #e74c3c;
  color: #e74c3c;
}
.btn.danger:hover {
  background: #e74c3c;
  color: #0b0d12;
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>