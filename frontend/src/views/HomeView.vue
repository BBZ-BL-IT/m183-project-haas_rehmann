<script setup lang="ts">
import { useAuthStore } from '@/stores'
import { RouterLink } from 'vue-router'

const auth = useAuthStore()
</script>

<template>
  <section class="hero">
    <h1>Grand Casino Rehmann</h1>

    <div v-if="auth.isLoading">Session wird geprüft …</div>

    <template v-else-if="auth.isAuthenticated">
      <p>Willkommen zurück, <strong>{{ auth.user?.appname }}</strong>.</p>
      <RouterLink to="/play" class="btn btn-primary">Zum Slot</RouterLink>
    </template>

    <template v-else>
      <p>Melde dich an, um zu spielen.</p>
      <button class="btn btn-primary" @click="auth.login">Login mit Kanidm</button>
    </template>
  </section>
</template>

<style scoped>
.hero {
  max-width: 640px;
  margin: 4rem auto;
  text-align: center;
  padding: 0 1.5rem;
}
h1 {
  font-size: 2.5rem;
  margin-bottom: 1rem;
}
p {
  color: #b8b9bd;
  margin-bottom: 2rem;
}
.btn {
  display: inline-block;
  background: #23262e;
  color: #e8e8ea;
  border: 1px solid #2e323c;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  font: inherit;
  text-decoration: none;
}
.btn-primary {
  background: #c9a227;
  border-color: #c9a227;
  color: #0b0d12;
  font-weight: 600;
}
.btn-primary:hover {
  background: #d8b13a;
}
</style>