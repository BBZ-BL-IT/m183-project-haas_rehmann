<script setup lang="ts">
import SlotMachine from '@/components/SlotMachine.vue'
import UserStats from '@/components/UserStats.vue'
import LoanButton from '@/components/LoanButton.vue'
</script>

<template>
  <section class="play">
    <UserStats />
    <SlotMachine />
    <LoanButton />
  </section>
</template>

<style scoped>
.play {
  max-width: 720px;
  margin: 2rem auto;
  padding: 0 1.5rem;
}
</style>