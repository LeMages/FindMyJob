<script setup lang="ts">
/**
 * Carte « Le poste » de la fiche candidature. Tant que la candidature est
 * « à postuler », formulaire modifiable d'un simple clic (grille de deux
 * colonnes) ; ensuite, fiche de lecture compacte (paires libellé / valeur,
 * lien de l'offre cliquable, mention « Figé depuis l'envoi »). « Voir
 * l'offre » ouvre le contenu de l'offre dans les deux cas. Propre à
 * `DetailView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import EditableField from '@/components/EditableField.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import { openDetailPopup, showToast, updateDetailCandidature } from '@/stores/app'
import { backendErrorMessage } from '@/composables/backend_error_message'
import { sourceLabel, sourceOptions, typeContratOptions } from '@/composables/candidature_display'
import type { CandidatureDetail } from '@/types'

const props = defineProps<{
  candidature: CandidatureDetail
  /** Les champs sont modifiables (« à postuler » et session active). */
  editable: boolean
  /** Infobulle des champs bloqués par une session en lecture seule, sinon `null`. */
  lockedHint: string | null
}>()

const { t } = useI18n()

/** Formulaire tant que la candidature n'est pas envoyée, fiche de lecture ensuite. */
const isDraft = computed(() => props.candidature.status === 'a_postuler')

/** Paires libellé / valeur de la fiche de lecture. */
const readOnlyRows = computed(() => {
  const c = props.candidature
  const contract = c.type_contrat
    ? (typeContratOptions().find((o) => o.value === c.type_contrat)?.label ?? c.type_contrat)
    : null
  return [
    { key: 'company', label: t('detail.job.company'), value: c.entreprise },
    { key: 'location', label: t('detail.job.location'), value: c.localisation },
    { key: 'contract', label: t('detail.job.contract'), value: contract },
    { key: 'salary', label: t('detail.job.salary'), value: c.remuneration },
    { key: 'platform', label: t('detail.job.platform'), value: c.source ? sourceLabel(c.source) : null },
  ]
})

/** Lien de l'offre sans le protocole ni « www. », pour l'affichage. */
const shortUrl = computed(() => (props.candidature.url_offre ?? '').replace(/^https?:\/\/(www\.)?/, ''))

/**
 * Ouvre le lien de l'offre dans le navigateur par défaut, via la commande
 * Tauri `open_url` (un lien `target="_blank"` ne fonctionne pas dans la webview).
 */
async function openOfferLink() {
  const url = props.candidature.url_offre
  if (!url) return
  try {
    await invoke('open_url', { url })
  } catch (e: unknown) {
    showToast(backendErrorMessage(e, t('error.fallback.open_link')), 'error')
  }
}

const LABEL = 'text-[12px] text-neutral-600 mb-[5px] block'
const SMALL_BUTTON = 'h-8 px-3 rounded-lg border border-neutral-300 bg-surface-2 text-[12.5px] font-medium text-neutral-900 inline-flex items-center gap-[6px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3'
</script>

<template>
  <SectionCard :title="t('detail.job.card_title')">
    <template #hint>
      <template v-if="isDraft">{{ t('detail.job.editable_hint') }}</template>
      <template v-else><UiIcon name="lock" /> {{ t('detail.job.frozen_hint') }}</template>
    </template>
    <template #actions>
      <button type="button" :class="SMALL_BUTTON" @click="openDetailPopup('offer')">{{ t('detail.job.view_offer') }}</button>
    </template>

    <!-- À postuler : formulaire modifiable d'un simple clic -->
    <div v-if="isDraft" class="grid grid-cols-1 sm:grid-cols-2 gap-x-4 gap-y-[14px] [&>div]:min-w-0">
      <div class="sm:col-span-2">
        <label :class="LABEL">{{ t('detail.job.title') }}</label>
        <EditableField
          single-click
          :model-value="candidature.poste"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :placeholder="t('detail.job.title_placeholder')"
          :on-save="v => updateDetailCandidature({ poste: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.company') }}</label>
        <EditableField
          single-click
          :model-value="candidature.entreprise"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :placeholder="t('detail.job.company_placeholder')"
          :on-save="v => updateDetailCandidature({ entreprise: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.location') }}</label>
        <EditableField
          single-click
          :model-value="candidature.localisation"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :placeholder="t('detail.job.location_placeholder')"
          :on-save="v => updateDetailCandidature({ localisation: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.contract') }}</label>
        <EditableField
          type="select"
          :options="typeContratOptions()"
          :model-value="candidature.type_contrat"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :on-save="v => updateDetailCandidature({ type_contrat: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.salary') }}</label>
        <EditableField
          single-click
          :model-value="candidature.remuneration"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :placeholder="t('detail.job.salary_placeholder')"
          :on-save="v => updateDetailCandidature({ remuneration: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.platform') }}</label>
        <EditableField
          type="select"
          :options="sourceOptions()"
          :model-value="candidature.source"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          :on-save="v => updateDetailCandidature({ source: v })"
        />
      </div>
      <div>
        <label :class="LABEL">{{ t('detail.job.link') }}</label>
        <EditableField
          single-click
          :model-value="candidature.url_offre"
          :editable="editable"
          :locked-hint="lockedHint ?? undefined"
          placeholder="https://..."
          :on-save="v => updateDetailCandidature({ url_offre: v })"
        />
      </div>
    </div>

    <!-- Envoyée et après : fiche de lecture compacte -->
    <dl v-else class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 m-0">
      <div v-for="row in readOnlyRows" :key="row.key" class="flex flex-col gap-0.5 py-[10px] border-t border-neutral-100 min-w-0">
        <dt class="text-[11.5px] text-neutral-500">{{ row.label }}</dt>
        <dd class="m-0 text-[13.5px] text-neutral-950 min-w-0 truncate">
          <template v-if="row.value">{{ row.value }}</template>
          <span v-else class="text-neutral-300">—</span>
        </dd>
      </div>
      <div class="flex flex-col gap-0.5 py-[10px] border-t border-neutral-100 min-w-0">
        <dt class="text-[11.5px] text-neutral-500">{{ t('detail.job.link') }}</dt>
        <dd class="m-0 text-[13.5px] min-w-0">
          <button
            v-if="candidature.url_offre"
            type="button"
            class="max-w-full border-0 bg-transparent p-0 font-sans text-[12.5px] text-blue-500 cursor-pointer inline-flex items-center gap-[5px] hover:underline"
            :title="candidature.url_offre"
            @click="openOfferLink"
          ><span class="truncate">{{ shortUrl }}</span> <UiIcon name="external" /></button>
          <span v-else class="text-neutral-300">—</span>
        </dd>
      </div>
    </dl>
  </SectionCard>
</template>
