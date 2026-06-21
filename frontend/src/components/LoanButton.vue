<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import { takeLoan } from '@/api/user'
import { toApiError } from '@/api/client'
import { useAuthStore } from '@/stores'

const auth = useAuthStore()

const amount = ref(1000)
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)
const successMsg = ref<string | null>(null)

// ticks every second so the countdown updates live
const now = ref(Date.now())
const timer = window.setInterval(() => (now.value = Date.now()), 1000)
onUnmounted(() => window.clearInterval(timer))

const maxLoans = computed(() => auth.user?.loans_max ?? 3)
const usedLoans = computed(() => auth.user?.loans_in_window ?? 0)
const remainingLoans = computed(() => Math.max(0, maxLoans.value - usedLoans.value))

// Seconds until the limit frees a slot again (or 0).
const resetInSeconds = computed(() => {
  const at = auth.user?.loans_reset_at
  if (!at) return 0
  return Math.max(0, Math.floor((new Date(at).getTime() - now.value) / 1000))
})

const countdown = computed(() => {
  const s = resetInSeconds.value
  if (s <= 0) return ''
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  const sec = s % 60
  const pad = (n: number) => String(n).padStart(2, '0')
  return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${pad(m)}:${pad(sec)}`
})

const limitReached = computed(() => remainingLoans.value <= 0)

const canRequest = computed(() => {
  if (!auth.user || isLoading.value) return false
  if (amount.value <= 0) return false
  return !limitReached.value
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
      loans_value: result.loans_value,
      loans_taken: result.loans_taken,
      loans_in_window: result.loans_in_window,
      loans_max: result.loans_max,
      loans_reset_at: result.loans_reset_at,
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
    <p class="info">{{ remainingLoans }} von {{ maxLoans }} Krediten übrig.</p>

    <div class="form">
      <input
        v-model.number="amount"
        type="number"
        :min="1"
        :disabled="isLoading || limitReached"
        class="amount-input"
      />
      <button class="loan-btn" :disabled="!canRequest" @click="requestLoan">
        {{ isLoading ? 'Wird verarbeitet …' : 'Kredit anfordern' }}
      </button>
    </div>

    <div v-if="limitReached && countdown" class="msg countdown">
      Limit erreicht – nächster Kredit in <strong>{{ countdown }}</strong>
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
.msg.countdown { color: #c9a227; }
.msg.countdown strong { font-variant-numeric: tabular-nums; }
</style>