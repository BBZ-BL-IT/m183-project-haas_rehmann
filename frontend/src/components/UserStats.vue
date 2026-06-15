<script setup lang="ts">
import { useAuthStore } from '@/stores'

const auth = useAuthStore()
</script>

<template>
  <div v-if="auth.user" class="stats">
    <div class="primary">
      <span class="label">Guthaben</span>
      <span class="primary-value">{{ auth.user.balance }}</span>
    </div>
    <div class="grid">
      <div class="cell">
        <span class="label">Eingesetzt</span>
        <span class="value">{{ auth.user.total_spent }}</span>
      </div>
      <div class="cell">
        <span class="label">Gewonnen</span>
        <span class="value">{{ auth.user.total_win }}</span>
      </div>
      <div class="cell">
        <span class="label">Kredite (heute)</span>
        <span class="value">{{ auth.user.loans_taken }} / 3</span>
      </div>
      <div class="cell">
        <span class="label">Offene Schuld</span>
        <span class="value debt">{{ auth.user.loans_total_owed }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats {
  background: #14171f;
  border: 1px solid #23262e;
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 1.5rem;
}
.primary {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid #23262e;
  margin-bottom: 1rem;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}
.cell {
  display: flex;
  flex-direction: column;
  align-items: center;
}
.label {
  font-size: 0.75rem;
  text-transform: uppercase;
  color: #8a8b91;
  letter-spacing: 0.05em;
  margin-bottom: 0.25rem;
}
.value {
  font-size: 1.25rem;
  font-weight: 600;
  color: #e8e8ea;
}
.primary-value {
  font-size: 2.25rem;
  font-weight: 700;
  color: #c9a227;
}
.debt {
  color: #e74c3c;
}
</style>