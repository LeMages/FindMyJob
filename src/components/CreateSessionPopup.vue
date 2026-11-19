<script setup lang="ts">
/**
 * Popup globale de création de session, ouverte depuis la barre latérale ou
 * l'écran de suivi (voir `state.createSessionPopup`).
 */

// Importation des modules Vue / framework
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'

// Importation des modules internes
import { state, cancelCreateSession, confirmCreateSession } from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'

const { t } = useI18n()

const nom = ref('')
const ville = ref('')
const zipCode = ref('')
const saving = ref(false)
const error = ref<string | null>(null)

/** Réinitialise le formulaire à chaque ouverture de la popup. */
function onOpen() {
  nom.value = ''
  ville.value = ''
  zipCode.value = ''
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
    await confirmCreateSession({
      nom: trimmed,
      ville: ville.value.trim() || null,
      zip_code: zipCode.value.trim() || null,
    })
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
  <AppModal v-if="state.createSessionPopup" :title="t('session.create.title')" :width="420" backdrop-close @close="cancelCreateSession">
    <div @vue:mounted="onOpen">
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

      <div class="flex flex-wrap gap-[14px] mb-[14px]">
        <div class="flex-1 min-w-[160px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('session.create.city_optional') }}</label>
          <input
            v-model="ville"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            :placeholder="t('session.params.city_placeholder')"
            @keydown="onKeydown"
          />
        </div>
        <div class="w-[140px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('session.create.zip_code_optional') }}</label>
          <input
            v-model="zipCode"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            :placeholder="t('session.params.zip_code_placeholder')"
            @keydown="onKeydown"
          />
        </div>
      </div>

      <div v-if="error" class="text-red-500 text-[12px] mt-[6px]">{{ error }}</div>
    </div>

    <template #footer>
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
    </template>
  </AppModal>
</template>
