<script setup lang="ts">
/**
 * Section « Polices » des Paramètres : familles de polices importées par
 * l'utilisateur pour la compilation Typst (nom exact à écrire dans `font:`,
 * variantes, homonymes), import depuis un dossier Google Fonts décompressé
 * (bouton dans l'en-tête), une ligne par police, aide en pied de carte.
 */

// Importation des modules Vue / framework
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Importation des composants
import SettingsSection from '@/components/SettingsSection.vue'
import TrashIcon from '@/views/settings/components/TrashIcon.vue'

// Importation des modules internes
import { useFonts } from '@/composables/useFonts'
import type { FontFolderView } from '@/types'

const { t } = useI18n()

const { fonts, loading, loadFonts, importFontFolder, deleteFont } = useFonts()

const importing = ref(false)
const importFeedback = ref<{ type: 'success' | 'error'; text: string } | null>(null)
const confirmDeleteFont = ref<FontFolderView | null>(null)

/** Noms de famille d'un dossier, entre guillemets, pour les messages. */
function quotedFamilies(folder: FontFolderView): string {
  return folder.families.map((f) => `"${f.name}"`).join(', ')
}

onMounted(() => {
  void loadFonts()
})

/** Ouvre le sélecteur de dossier natif, puis importe le dossier choisi. */
async function pickAndImportFolder() {
  const selected = await open({ directory: true, multiple: false })
  if (!selected) return

  importing.value = true
  importFeedback.value = null
  try {
    const result = await importFontFolder(selected)
    importFeedback.value = result.error !== null
      ? { type: 'error', text: result.error }
      : { type: 'success', text: t('settings.fonts.imported', { families: quotedFamilies(result.imported) }) }
  } finally {
    importing.value = false
  }
}

/** Supprime la police en attente de confirmation. */
async function confirmDelete() {
  if (!confirmDeleteFont.value) return
  const error = await deleteFont(confirmDeleteFont.value.folder)
  if (error) {
    importFeedback.value = { type: 'error', text: error }
  }
  confirmDeleteFont.value = null
}

/** Ouvre fonts.google.com dans le navigateur par défaut de l'utilisateur. */
async function openFontsGoogle() {
  try {
    await invoke('open_url', { url: 'https://fonts.google.com' })
  } catch (e: unknown) {
    console.error("Impossible d'ouvrir ce lien.", e)
  }
}
</script>

<template>
  <SettingsSection :title="t('settings.fonts.section')" :description="t('settings.fonts.description')">
    <template #action>
      <button
        type="button"
        class="h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="importing"
        @click="pickAndImportFolder"
      >{{ importing ? t('settings.fonts.importing') : t('settings.fonts.add') }}</button>
    </template>

    <div
      v-if="importFeedback"
      class="mx-5 mt-3 px-3 py-2 rounded-lg text-[12.5px]"
      :class="importFeedback.type === 'success' ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-600'"
    >{{ importFeedback.text }}</div>

    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono border-b border-neutral-100">{{ t('common.loading') }}</div>
    <div v-else-if="!fonts.length" class="px-5 py-[22px] text-[12.5px] text-neutral-500 border-b border-neutral-100">{{ t('settings.fonts.empty') }}</div>
    <div
      v-for="font in fonts"
      v-else
      :key="font.folder"
      class="flex items-center gap-[14px] px-5 py-3 border-b border-neutral-100"
    >
      <div class="w-[38px] h-[38px] rounded-lg bg-neutral-100 flex items-center justify-center text-[16px] font-semibold text-neutral-700 flex-none font-[Georgia,serif]">Aa</div>
      <div class="flex-1 min-w-0">
        <div v-for="family in font.families" :key="family.name" class="flex items-baseline gap-x-[10px] flex-wrap">
          <span class="text-[13.5px] font-medium text-neutral-950 select-all" :title="t('settings.fonts.name_hint')">{{ family.name }}</span>
          <span class="font-mono text-[11px] text-neutral-500">{{ family.variants.join(' · ') }}</span>
          <div v-if="font.shadowed_families.includes(family.name)" class="basis-full flex items-center gap-[6px] text-[12px] text-amber-700 mt-[3px]">
            <svg class="w-[13px] h-[13px] flex-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" />
            </svg>
            <span>{{ t('settings.fonts.shadowed_warning') }}</span>
          </div>
        </div>
      </div>
      <span class="text-[11.5px] text-neutral-500 flex-none font-mono">{{ t('settings.fonts.file_count', { n: font.file_count }, font.file_count) }}</span>
      <button
        type="button"
        class="w-8 h-8 rounded-lg border border-transparent bg-transparent text-neutral-400 flex items-center justify-center cursor-pointer flex-none hover:text-red-600 hover:bg-red-50 hover:border-red-200"
        :title="t('common.delete')"
        :aria-label="t('common.delete')"
        @click="confirmDeleteFont = font"
      ><TrashIcon /></button>
    </div>

    <div class="px-5 py-3 text-[12px] text-neutral-500 leading-normal bg-neutral-50 rounded-b-xl">
      <i18n-t keypath="settings.fonts.help" tag="span" scope="global">
        <template #link>
          <button type="button" class="text-blue-500 bg-transparent border-none p-0 cursor-pointer hover:underline font-sans text-[12px]" @click="openFontsGoogle">fonts.google.com</button>
        </template>
        <template #font>
          <code class="font-mono">font:</code>
        </template>
      </i18n-t>
    </div>
  </SettingsSection>

  <!-- Confirmation de suppression -->
  <div
    v-if="confirmDeleteFont"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100] p-4"
    @click="confirmDeleteFont = null"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-w-full max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">{{ t('settings.fonts.delete_title') }}</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="confirmDeleteFont = null">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[6px]">{{ t('common.irreversible') }}</div>
      <div class="bg-neutral-50 border border-neutral-200 rounded-9 px-[14px] py-3 mb-5 text-[13.5px]">
        <strong>{{ quotedFamilies(confirmDeleteFont) || confirmDeleteFont.folder }}</strong>
        {{ t('settings.fonts.delete_body', confirmDeleteFont.families.length) }}
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="confirmDeleteFont = null"
        >{{ t('common.cancel') }}</button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-surface-3 hover:text-neutral-800"
          type="button"
          @click="confirmDelete"
        >{{ t('common.delete') }}</button>
      </div>
    </div>
  </div>
</template>
