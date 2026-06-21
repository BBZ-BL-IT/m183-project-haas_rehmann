<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { useAuthStore } from '@/stores'

const auth = useAuthStore()
</script>

<template>
  <header class="app-header">
    <RouterLink to="/" class="brand">Grand Casino Rehmann</RouterLink>

    <nav class="nav">
      <template v-if="auth.isAuthenticated">
        <RouterLink to="/play">Slots</RouterLink>
        <RouterLink v-if="auth.isAdmin" to="/admin">Admin</RouterLink>
        <span class="user">{{ auth.user?.username }}</span>
        <button class="btn" @click="auth.logout">Logout</button>
      </template>
      <template v-else>
        <RouterLink to="/register" class="btn">Registrieren</RouterLink>
        <button class="btn btn-primary" @click="auth.login">Login</button>
      </template>
    </nav>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 2rem;
  background: #14171f;
  border-bottom: 1px solid #23262e;
}
.brand {
  font-weight: 700;
  font-size: 1.2rem;
  color: #e8e8ea;
  text-decoration: none;
}
.nav {
  display: flex;
  align-items: center;
  gap: 1.25rem;
}
.nav a {
  color: #b8b9bd;
  text-decoration: none;
}
.nav a.router-link-active {
  color: #e8e8ea;
}
.user {
  color: #b8b9bd;
  font-size: 0.9rem;
}
.btn {
  background: #23262e;
  color: #e8e8ea;
  border: 1px solid #2e323c;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  cursor: pointer;
  font: inherit;
}
.btn:hover {
  background: #2e323c;
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