<script setup lang="ts">
/**
 * Aide à la préparation des modèles (CV et lettre) au contrat FindMyJob,
 * affichée à l'endroit de l'import : ce que doit être le fichier, deux
 * boutons qui copient le prompt de conversion pour Claude (dans la langue
 * de l'interface) et les liens vers le contrat et les exemples sur GitHub.
 * Le prompt et les liens viennent du backend (source unique).
 */

// Importation des modules Vue / framework
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

// Importation des modules internes
import { backendErrorMessage } from '@/composables/backend_error_message'
import { currentUiLanguage } from '@/i18n'
import type { ReferenceDocLinks } from '@/types'

const { t } = useI18n()

type DocType = 'cv' | 'lettre_motivation'

/** Durée d'affichage de la confirmation « copié », en millisecondes. */
const COPIED_DURATION_MS = 2000

/** Petit bouton secondaire, comme ceux des cartes de modèle. */
const SM_BUTTON = 'h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed'

const links = ref<ReferenceDocLinks | null>(null)
const copied = ref<DocType | null>(null)
const copying = ref<DocType | null>(null)
const error = ref<string | null>(null)
let copiedTimer: ReturnType<typeof setTimeout> | null = null

onMounted(async () => {
  try {
    links.value = await invoke<ReferenceDocLinks>('get_reference_doc_links', { lang: currentUiLanguage() })
  } catch (e: unknown) {
    error.value = backendErrorMessage(e, t('error.fallback.unexpected'))
  }
})

/**
 * Copie le prompt de conversion d'un type de document dans le
 * presse-papier, puis affiche la confirmation sur son bouton.
 *
 * @param docType - Type de document du prompt.
 */
async function copyPrompt(docType: DocType): Promise<void> {
  error.value = null
  copying.value = docType
  try {
    const prompt = await invoke<string>('get_reference_prompt', { docType, lang: currentUiLanguage() })
    await writeText(prompt)
    copied.value = docType
    if (copiedTimer) clearTimeout(copiedTimer)
    copiedTimer = setTimeout(() => { copied.value = null }, COPIED_DURATION_MS)
  } catch (e: unknown) {
    error.value = backendErrorMessage(e, t('documents.preparation.copy_failed'))
  } finally {
    copying.value = null
  }
}

/**
 * Ouvre un lien de documentation dans le navigateur par défaut (un lien
 * `target="_blank"` ne fonctionne pas dans la webview).
 *
 * @param url - URL à ouvrir.
 */
async function openLink(url: string): Promise<void> {
  error.value = null
  try {
    await invoke('open_url', { url })
  } catch (e: unknown) {
    error.value = backendErrorMessage(e, t('error.fallback.open_link'))
  }
}
</script>

<template>
  <div class="text-[12.5px] text-neutral-600 leading-[1.5] flex flex-col gap-[10px]">
    <div class="text-[13px] font-semibold text-neutral-950">{{ t('documents.preparation.title') }}</div>
    <p class="m-0">{{ t('documents.preparation.intro') }}</p>
    <ol class="m-0 pl-5 list-decimal flex flex-col gap-[3px]">
      <li>{{ t('documents.preparation.step_copy') }}</li>
      <li>{{ t('documents.preparation.step_claude') }}</li>
      <li>{{ t('documents.preparation.step_import') }}</li>
    </ol>

    <div class="flex flex-wrap gap-2">
      <button
        v-for="docType in (['cv', 'lettre_motivation'] as const)"
        :key="docType"
        type="button"
        :class="SM_BUTTON"
        :disabled="copying !== null"
        @click="copyPrompt(docType)"
      >
        <template v-if="copied === docType">{{ t('documents.preparation.copied') }}</template>
        <template v-else>{{ docType === 'cv' ? t('documents.preparation.copy_cv') : t('documents.preparation.copy_lm') }}</template>
      </button>
    </div>

    <div v-if="links" class="flex flex-wrap gap-x-4 gap-y-1">
      <button type="button" class="p-0 bg-transparent border-none cursor-pointer text-blue-600 hover:underline font-sans text-[12.5px]" @click="openLink(links.contract)">{{ t('documents.preparation.contract_link') }}</button>
      <button type="button" class="p-0 bg-transparent border-none cursor-pointer text-blue-600 hover:underline font-sans text-[12.5px]" @click="openLink(links.cv_example)">{{ t('documents.preparation.cv_example_link') }}</button>
      <button type="button" class="p-0 bg-transparent border-none cursor-pointer text-blue-600 hover:underline font-sans text-[12.5px]" @click="openLink(links.lm_example)">{{ t('documents.preparation.lm_example_link') }}</button>
    </div>

    <div v-if="error" class="text-[12px] text-red-600">{{ error }}</div>
  </div>
</template>
