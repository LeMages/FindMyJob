<script setup lang="ts">
/**
 * Carte d'une section du profil, éditable section par section : crayon en
 * lecture, « + Ajouter » pendant l'édition d'une liste, pied Annuler / Valider
 * pendant l'édition. Les autres cartes sont estompées tant qu'une carte est
 * en cours d'édition, et leur crayon est masqué (une seule à la fois).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

// Importation des composants
import ProfileEditButton from './ProfileEditButton.vue'

// Importation des types
import type { ProfileCardMode } from './profile_sections'

defineProps<{
  /** Titre de la section (omis si l'en-tête est fourni par le slot `header`). */
  title?: string
  /** Mode d'affichage de la carte. */
  mode: ProfileCardMode
  /** `true` pour une liste : « + Ajouter » à la place du crayon pendant l'édition. */
  list?: boolean
  /** Des modifications sont en attente (active Valider). */
  dirty: boolean
  /** Sauvegarde en cours. */
  saving: boolean
  /** Erreur de sauvegarde à afficher dans le pied. */
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  add: []
  cancel: []
  save: []
}>()

const { t } = useI18n()
</script>

<template>
  <div
    class="bg-surface-1 border rounded-xl px-[22px] py-5 mb-[18px] transition-opacity duration-150"
    :class="[mode === 'editing' ? 'border-neutral-400' : 'border-neutral-200', mode === 'dimmed' && 'opacity-55']"
  >
    <slot name="header">
      <div class="flex items-center justify-between min-h-[30px]">
        <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500">{{ title }}</div>
        <ProfileEditButton v-if="mode === 'idle'" @click="emit('edit')" />
        <button
          v-else-if="mode === 'editing' && list"
          class="h-[30px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-surface-2 font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="emit('add')"
        >{{ t('common.add') }}</button>
      </div>
    </slot>

    <slot />

    <div v-if="mode === 'editing'" class="flex flex-wrap items-center justify-end gap-[10px] mt-4 pt-[14px] border-t border-neutral-100">
      <span v-if="error" class="font-mono text-[12px] text-red-500 flex-1 min-w-[160px]">{{ error }}</span>
      <span v-else class="flex-1" />
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="emit('cancel')"
      >{{ t('common.cancel') }}</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !dirty || saving }"
        type="button"
        :disabled="!dirty || saving"
        @click="emit('save')"
      >{{ saving ? t('profile.saving') : t('common.validate') }}</button>
    </div>
  </div>
</template>
