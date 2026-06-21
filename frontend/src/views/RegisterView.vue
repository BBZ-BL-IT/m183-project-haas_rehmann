<script setup lang="ts">
import { computed, ref } from 'vue'
import { register } from '@/api/register'
import { toApiError } from '@/api/client'

const username = ref('')
const isLoading = ref(false)
const errorMsg = ref<string | null>(null)
const resetUrl = ref<string | null>(null)

// Muss zu validate_kanidm_name im Backend passen: Kleinbuchstabe am Anfang,
// danach a-z 0-9 . _ - , max 20 Zeichen.
const USERNAME_RE = /^[a-z][a-z0-9._-]{0,19}$/
const isValid = computed(() => USERNAME_RE.test(username.value))

async function submit(): Promise<void> {
  if (!isValid.value || isLoading.value) return
  isLoading.value = true
  errorMsg.value = null
  resetUrl.value = null
  try {
    const res = await register({ username: username.value })
    resetUrl.value = res.reset_url
  } catch (err) {
    errorMsg.value = toApiError(err).message ?? 'Registrierung fehlgeschlagen'
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <section class="register">
    <h1>Account erstellen</h1>

    <template v-if="!resetUrl">
      <p class="info">
        Wähle einen Benutzernamen (max. 20 Zeichen, klein, Buchstaben/Ziffern und
        <code>. _ -</code>). Danach erhältst du einen Link, um dein Passwort zu setzen.
      </p>

      <div class="form">
        <input
          v-model="username"
          class="input"
          placeholder="benutzername"
          autocomplete="off"
          :disabled="isLoading"
          @keyup.enter="submit"
        />
        <button class="btn primary" :disabled="!isValid || isLoading" @click="submit">
          {{ isLoading ? 'Wird erstellt …' : 'Registrieren' }}
        </button>
      </div>
      <p v-if="username && !isValid" class="hint">
        Ungültiger Benutzername.
      </p>
      <div v-if="errorMsg" class="msg error">{{ errorMsg }}</div>
    </template>

    <template v-else>
      <div class="msg success">Account <strong>{{ username }}</strong> erstellt! 🎉</div>
      <p class="info">Setze jetzt dein Passwort über diesen Link (Zertifikatswarnung akzeptieren):</p>
      <a class="reset-link" :href="resetUrl" target="_blank" rel="noopener">{{ resetUrl }}</a>
      <p class="info">Danach kannst du dich über <RouterLink to="/">Login</RouterLink> anmelden.</p>
    </template>
  </section>
</template>

<style scoped>
.register {
  max-width: 560px;
  margin: 3rem auto;
  padding: 0 1.5rem;
}
h1 {
  margin-bottom: 1rem;
}
.info {
  color: #b8b9bd;
  line-height: 1.5;
}
code {
  background: #0b0d12;
  padding: 0 0.3rem;
  border-radius: 4px;
}
.form {
  display: flex;
  gap: 0.75rem;
  margin: 1.25rem 0 0.5rem;
}
.input {
  flex: 1;
  background: #0b0d12;
  border: 1px solid #2e323c;
  color: #e8e8ea;
  padding: 0.6rem 0.8rem;
  border-radius: 6px;
  font: inherit;
}
.btn {
  border: 1px solid #2e323c;
  background: #23262e;
  color: #e8e8ea;
  padding: 0.6rem 1.25rem;
  border-radius: 6px;
  cursor: pointer;
  font: inherit;
}
.btn.primary {
  background: #c9a227;
  border-color: #c9a227;
  color: #0b0d12;
  font-weight: 700;
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.hint {
  color: #e0a23a;
  font-size: 0.85rem;
}
.msg {
  margin: 1rem 0;
  font-size: 1rem;
}
.msg.success {
  color: #2ecc71;
}
.msg.error {
  color: #e74c3c;
}
.reset-link {
  display: block;
  word-break: break-all;
  color: #c9a227;
  background: #14171f;
  border: 1px solid #23262e;
  padding: 0.75rem;
  border-radius: 8px;
  margin: 0.75rem 0;
}
</style>
