<script setup lang="ts">
import { computed, ref } from 'vue'
import { spin } from '@/api/slot'
import { toApiError } from '@/api/client'
import { useAuthStore } from '@/stores'

const auth = useAuthStore()

// Mapping Zahl → Emoji. Welche Zahl das Backend für welches Symbol nutzt,
// ist bei uns Konvention. Falls dein Backend andere Zahlen liefert,
// hier ergänzen.
const SYMBOLS: Record<number, string> = {
  1: '🍒',
  2: '🍋',
  3: '🍊',
  4: '🍇',
  5: '🔔',
  6: '⭐',
  7: '💎',
}
const symbolFor = (n: number) => SYMBOLS[n] ?? '❓'

const stake = ref(10)
const isSpinning = ref(false)
const errorMsg = ref<string | null>(null)
const lastWin = ref<number | null>(null)

// Anfangs-Reels (rein optisch)
const reels = ref<number[][]>([
  [1, 2, 3],
  [4, 5, 6],
  [7, 1, 2],
])

const canSpin = computed(() => {
  if (!auth.user || isSpinning.value) return false
  if (stake.value <= 0) return false
  return stake.value <= auth.user.balance
})

async function handleSpin(): Promise<void> {
  if (!canSpin.value) return
  isSpinning.value = true
  errorMsg.value = null
  lastWin.value = null

  // Kleine Rolling-Animation: random Symbole bis Backend antwortet
  const animation = window.setInterval(() => {
    reels.value = Array.from({ length: 3 }, () =>
      Array.from({ length: 3 }, () => Math.floor(Math.random() * 7) + 1),
    )
  }, 80)

  try {
    const result = await spin({ stake_amount: stake.value })

    // Mindestens 600ms Animation, damit es sich anfühlt wie Spin
    window.setTimeout(() => {
      window.clearInterval(animation)
      reels.value = result.pattern
      lastWin.value = result.amount_earned

      // Lokal die User-Stats anpassen, ohne /user/info neu zu laden
      if (auth.user) {
        auth.patchUser({
          balance: auth.user.balance - stake.value + result.amount_earned,
          total_spent: auth.user.total_spent + stake.value,
          total_win: auth.user.total_win + result.amount_earned,
        })
      }
      isSpinning.value = false
    }, 600)
  } catch (err) {
    window.clearInterval(animation)
    errorMsg.value = toApiError(err).message ?? 'Spin fehlgeschlagen'
    isSpinning.value = false
  }
}
</script>

<template>
  <div class="slot">
    <div class="reels">
      <div v-for="(row, rowIdx) in reels" :key="rowIdx" class="reel-row">
        <div
          v-for="(cell, colIdx) in row"
          :key="colIdx"
          class="reel-cell"
          :class="{ payline: rowIdx === 1 }"
        >
          {{ symbolFor(cell) }}
        </div>
      </div>
    </div>

    <div v-if="lastWin !== null" class="result" :class="{ win: lastWin > 0 }">
      <template v-if="lastWin > 0">Gewinn: +{{ lastWin }} 🎉</template>
      <template v-else>Kein Gewinn</template>
    </div>

    <div v-if="errorMsg" class="error">{{ errorMsg }}</div>

    <div class="controls">
      <label class="stake-label">
        Einsatz
        <input
          v-model.number="stake"
          type="number"
          min="1"
          :disabled="isSpinning"
          class="stake-input"
        />
      </label>
      <button class="spin-btn" :disabled="!canSpin" @click="handleSpin">
        {{ isSpinning ? 'Spinning …' : 'Spin' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.slot {
  background: #14171f;
  border: 1px solid #23262e;
  border-radius: 12px;
  padding: 2rem;
}
.reels {
  display: grid;
  grid-template-rows: repeat(3, 1fr);
  gap: 8px;
  background: #0b0d12;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1.5rem;
}
.reel-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}
.reel-cell {
  background: #1c1f28;
  border: 2px solid #2e323c;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 3rem;
  aspect-ratio: 1;
}
.reel-cell.payline {
  border-color: #c9a227;
}
.result {
  text-align: center;
  font-size: 1.4rem;
  margin-bottom: 1rem;
  color: #b8b9bd;
}
.result.win {
  color: #c9a227;
  font-weight: 700;
}
.error {
  text-align: center;
  color: #e74c3c;
  margin-bottom: 1rem;
}
.controls {
  display: flex;
  gap: 1rem;
  align-items: end;
  justify-content: center;
}
.stake-label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  color: #b8b9bd;
}
.stake-input {
  background: #0b0d12;
  border: 1px solid #2e323c;
  color: #e8e8ea;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  font: inherit;
  width: 100px;
}
.spin-btn {
  background: #c9a227;
  color: #0b0d12;
  border: none;
  padding: 0.75rem 2rem;
  font-size: 1.1rem;
  font-weight: 700;
  border-radius: 6px;
  cursor: pointer;
}
.spin-btn:disabled {
  background: #2e323c;
  color: #5a5b60;
  cursor: not-allowed;
}
.spin-btn:not(:disabled):hover {
  background: #d8b13a;
}
</style>