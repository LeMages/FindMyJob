<script setup lang="ts">
/**
 * Section « Polices » des Paramètres : liste des familles de polices
 * importées par l'utilisateur pour la compilation Typst, import depuis un
 * dossier Google Fonts décompressé, suppression.
 */

// Importation des modules Vue / framework
import { onMounted, ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

// Importation des modules internes
import { useFonts } from '@/composables/useFonts'
import type { FontFamilyView } from '@/types'

const { fonts, loading, loadFonts, importFontFolder, deleteFont } = useFonts()

const importing = ref(false)
const importFeedback = ref<{ type: 'success' | 'error'; text: string } | null>(null)
const confirmDeleteFont = ref<FontFamilyView | null>(null)

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
    const error = await importFontFolder(selected)
    importFeedback.value = error
      ? { type: 'error', text: error }
      : { type: 'success', text: 'Police importée.' }
  } finally {
    importing.value = false
  }
}

/** Supprime la police en attente de confirmation. */
async function confirmDelete() {
  if (!confirmDeleteFont.value) return
  const error = await deleteFont(confirmDeleteFont.value.name)
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
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">POLICES</div>

    <div v-if="loading" class="text-[12px] text-neutral-400 text-center py-6 font-mono">Chargement…</div>
    <template v-else>
      <div v-if="fonts.length" class="mb-[14px]">
        <div
          v-for="font in fonts"
          :key="font.name"
          class="flex items-center gap-2 py-[7px] border-b border-neutral-50 last:border-b-0"
        >
          <span class="flex-1 text-[13px] text-neutral-950">{{ font.name }}</span>
          <span class="font-mono text-[11px] text-neutral-500 flex-none">{{ font.file_count }} fichier{{ font.file_count > 1 ? 's' : '' }}</span>
          <button
            class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
            type="button"
            title="Supprimer"
            @click="confirmDeleteFont = font"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="3 6 5 6 21 6"/>
              <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
              <path d="M10 11v6M14 11v6"/>
              <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/>
            </svg>
          </button>
        </div>
      </div>
      <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune police importée</div>
    </template>

    <div class="flex items-center justify-between gap-5 pt-2">
      <div>
        <div v-if="importFeedback" class="text-[12.5px] font-mono mb-[3px]" :class="importFeedback.type === 'success' ? 'text-green-600' : 'text-red-500'">
          {{ importFeedback.text }}
        </div>
        <div class="text-[12px] text-neutral-400 max-w-[520px] leading-[1.45]">
          Téléchargez une police sur
          <button type="button" class="text-blue-500 bg-transparent border-none p-0 cursor-pointer hover:underline" @click="openFontsGoogle">fonts.google.com</button>,
          décompressez le ZIP, puis sélectionnez le dossier ici.
        </div>
      </div>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3 disabled:opacity-50 disabled:cursor-not-allowed"
        :disabled="importing"
        @click="pickAndImportFolder"
      >{{ importing ? 'Import…' : 'Ajouter une police' }}</button>
    </div>
  </div>

  <!-- Confirmation de suppression -->
  <div
    v-if="confirmDeleteFont"
    class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]"
    @click="confirmDeleteFont = null"
  >
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6" @click.stop>
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">Supprimer la police</div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="confirmDeleteFont = null">×</button>
      </div>

      <div class="text-[13px] text-neutral-500 font-mono -mt-1 mb-[6px]">Cette action est irréversible.</div>
      <div class="bg-neutral-50 border border-neutral-200 rounded-9 px-[14px] py-3 mb-5 text-[13.5px]">
        <strong>{{ confirmDeleteFont.name }}</strong> sera supprimée définitivement.
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="confirmDeleteFont = null"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-surface-3 hover:text-neutral-800"
          type="button"
          @click="confirmDelete"
        >Supprimer</button>
      </div>
    </div>
  </div>
</template>
