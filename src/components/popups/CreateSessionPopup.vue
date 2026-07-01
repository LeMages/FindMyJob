<script setup lang="ts">
import { ref } from 'vue'
import { state, cancelCreateSession, confirmCreateSession } from '@/stores/app'

const nom = ref('')
const saving = ref(false)
const error = ref<string | null>(null)

function onOpen() {
  nom.value = ''
  error.value = null
  saving.value = false
}

async function submit() {
  const trimmed = nom.value.trim()
  if (!trimmed) return
  saving.value = true
  error.value = null
  try {
    await confirmCreateSession(trimmed)
    nom.value = ''
  } catch (e: unknown) {
    error.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : 'Impossible de créer la session.'
  } finally {
    saving.value = false
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') submit()
  if (e.key === 'Escape') cancelCreateSession()
}
</script>

<template>
  <div v-if="state.createSessionPopup" class="popup-backdrop" @click="cancelCreateSession">
    <div class="popup-box" style="width:420px" @click.stop @vue:mounted="onOpen">
      <div class="popup-hd">
        <div class="popup-title">Nouvelle session</div>
        <button class="popup-close" @click="cancelCreateSession">×</button>
      </div>

      <div class="sub" style="margin:-4px 0 18px">
        Une session regroupe un ensemble de candidatures (ex : « Stage M2 – 2025 »).
      </div>

      <div class="field">
        <label class="flabel">Nom de la session</label>
        <input
          v-model="nom"
          class="input"
          placeholder="Ex. Stage M2 – 2025"
          autofocus
          @keydown="onKeydown"
        />
      </div>

      <div v-if="error" style="color:#b94a48;font-size:12px;margin-top:6px">{{ error }}</div>

      <div class="popup-foot" style="margin-top:20px">
        <button class="btn" @click="cancelCreateSession">Annuler</button>
        <button
          class="btn pri"
          :disabled="saving || !nom.trim()"
          :class="{ disabled: saving || !nom.trim() }"
          @click="submit"
        >
          {{ saving ? 'Création…' : 'Créer la session' }}
        </button>
      </div>
    </div>
  </div>
</template>
