<script setup lang="ts">
/**
 * Popup globale de création de session, ouverte depuis la barre latérale ou
 * l'écran de suivi (voir `state.createSessionPopup`).
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules internes
import { state, cancelCreateSession, confirmCreateSession } from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'

const { t } = useI18n()

const nom = ref('')
const saving = ref(false)
const error = ref<string | null>(null)

/** Réinitialise le formulaire à chaque ouverture de la popup. */
function onOpen() {
  nom.value = ''
  error.value = null
  saving.value = false
}

/** Crée la session saisie via le store applicatif. */
async function submit() {
  const trimmed = nom.value.trim()
  if (!trimmed) return
  saving.value = true
  error.value = null
  try {
    await confirmCreateSession(trimmed)
    nom.value = ''
  } catch (e: unknown) {
    error.value = backendErrorMessage(e, t('error.fallback.create_session'))
  } finally {
    saving.value = false
  }
}

/** Raccourcis clavier : Entrée pour valider, Échap pour annuler. */
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') submit()
  if (e.key === 'Escape') cancelCreateSession()
}
</script>

<template>
  <div
    v-if="state.createSessionPopup"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]"
    @click="cancelCreateSession"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[420px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop @vue:mounted="onOpen">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">{{ t('session.create.title') }}</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" @click="cancelCreateSession">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[18px]">
        {{ t('session.create.intro') }}
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('session.create.name') }}</label>
        <input
          v-model="nom"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
          :placeholder="t('session.create.placeholder')"
          autofocus
          @keydown="onKeydown"
        />
      </div>

      <div v-if="error" class="text-red-500 text-[12px] mt-[6px]">{{ error }}</div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          @click="cancelCreateSession"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white hover:bg-neutral-800"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': saving || !nom.trim() }"
          :disabled="saving || !nom.trim()"
          @click="submit"
        >
          {{ saving ? t('onboarding.creating') : t('session.create.submit') }}
        </button>
      </div>
    </div>
  </div>
</template>
