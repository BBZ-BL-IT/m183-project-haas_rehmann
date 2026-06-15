<script setup lang="ts">
import { computed, ref } from 'vue'
import { takeLoan } from '@/api/user'
import { toApiError } from '@/api/client'
import { useAuthStore } from '@/stores'

const auth = useAuthStore()

const MAX_AMOUNT = 10000
const MAX_LOANS_PER_DAY = 3

const amount = ref(1000)
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)
const successMsg = ref<string | null>(null)

const remainingLoans = computed(() => {
  if (!auth.user) return 0
  return Math.max(0, MAX_LOANS_PER_DAY - auth.user.loans_taken)
})

const canRequest = computed(() => {
  if (!auth.user || isLoading.value) return false
  if (amount.value <= 0 || amount.value > MAX_AMOUNT) return false
  return remainingLoans.value > 0
})

async function requestLoan(): Promise<void> {
  if (!canRequest.value) return
  isLoading.value = true
  errorMsg.value = null
  successMsg.value = null

  try {
    const result = await takeLoan(amount.value)
    auth.patchUser({
      balance: result.balance,
      loans_total_amount: result.loans_total_amount,
      loans_taken: result.loans_taken,
      loans_total_owed: result.loans_total_owed,
    })
    successMsg.value = `Kredit über ${amount.value} aufgenommen.`
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Kredit fehlgeschlagen'
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="loan-card">
    <h3>Kredit aufnehmen</h3>
    <p class="info">
      {{ remainingLoans }} von {{ MAX_LOANS_PER_DAY }} Krediten heute übrig.
      Max. {{ MAX_AMOUNT }} pro Kredit.
    </p>

    <div class="form">
      <input
        v-model.number="amount"
        type="number"
        :min="1"
        :max="MAX_AMOUNT"
        :disabled="isLoading || remainingLoans === 0"
        class="amount-input"
      />
      <button class="loan-btn" :disabled="!canRequest" @click="requestLoan">
        {{ isLoading ? 'Wird verarbeitet …' : 'Kredit anfordern' }}
      </button>
    </div>

    <div v-if="successMsg" class="msg success">{{ successMsg }}</div>
    <div v-if="errorMsg" class="msg error">{{ errorMsg }}</div>
  </div>
</template>

<style scoped>
.loan-card {
  background: #14171f;
  border: 1px solid #23262e;
  border-radius: 12px;
  padding: 1.5rem;
  margin-top: 1.5rem;
}
h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1.1rem;
}
.info {
  color: #b8b9bd;
  font-size: 0.9rem;
  margin: 0 0 1rem 0;
}
.form {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}
.amount-input {
  background: #0b0d12;
  border: 1px solid #2e323c;
  color: #e8e8ea;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  font: inherit;
  flex: 1;
  max-width: 150px;
}
.loan-btn {
  background: #2e323c;
  color: #e8e8ea;
  border: 1px solid #3a3e48;
  padding: 0.5rem 1.25rem;
  border-radius: 6px;
  font: inherit;
  cursor: pointer;
}
.loan-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.loan-btn:not(:disabled):hover {
  background: #3a3e48;
}
.msg {
  margin-top: 1rem;
  font-size: 0.9rem;
}
.msg.success { color: #2ecc71; }
.msg.error { color: #e74c3c; }
</style>