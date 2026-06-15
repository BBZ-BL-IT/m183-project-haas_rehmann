<script setup lang="ts">
import { useAuthStore } from '@/stores'
import SlotMachine from '@/components/SlotMachine.vue'

const auth = useAuthStore()
</script>

<template>
  <section class="play">
    <header class="play-header">
      <div class="stat">
        <span class="stat-label">Guthaben</span>
        <span class="stat-value">{{ auth.user?.balance ?? 0 }}</span>
      </div>
    </header>
    <SlotMachine />
  </section>
</template>

<style scoped>
.play {
  max-width: 720px;
  margin: 2rem auto;
  padding: 0 1.5rem;
}
.play-header {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 1.5rem;
}
.stat {
  background: #14171f;
  border: 1px solid #23262e;
  border-radius: 8px;
  padding: 0.75rem 1.25rem;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.stat-label {
  font-size: 0.75rem;
  text-transform: uppercase;
  color: #8a8b91;
  letter-spacing: 0.05em;
}
.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: #c9a227;
}
</style>