<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { fetchUserList, updateUser } from '@/api/admin'
import { toApiError } from '@/api/client'
import type { AdminUserRow } from '@/types'

const users = ref<AdminUserRow[]>([])
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)

const editId = ref<number | null>(null)
const editAppname = ref('')
const editBalance = ref(0)
const isSaving = ref(false)

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
  editAppname.value = u.appname
  editBalance.value = u.balance
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
      appname: editAppname.value,
      balance: editBalance.value,
    })
    const row = users.value.find((u) => u.id === editId.value)
    if (row) {
      row.appname = res.appname
      row.balance = res.balance
    }
    editId.value = null
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Speichern fehlgeschlagen'
  } finally {
    isSaving.value = false
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
          <th>Appname</th>
          <th>Guthaben</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="u in users" :key="u.id">
          <td>{{ u.id }}</td>

          <template v-if="editId === u.id">
            <td><input v-model="editAppname" class="cell-input" /></td>
            <td><input v-model.number="editBalance" type="number" class="cell-input" /></td>
            <td class="actions">
              <button class="btn primary" :disabled="isSaving" @click="save">
                {{ isSaving ? '…' : 'Speichern' }}
              </button>
              <button class="btn" :disabled="isSaving" @click="cancelEdit">Abbrechen</button>
            </td>
          </template>

          <template v-else>
            <td>{{ u.appname }}</td>
            <td>{{ u.balance }}</td>
            <td class="actions">
              <button class="btn" @click="startEdit(u)">Bearbeiten</button>
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
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>