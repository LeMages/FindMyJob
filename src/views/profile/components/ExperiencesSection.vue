<script setup lang="ts">
/**
 * Section « Expériences » du profil : affichage, ajout, modification et
 * suppression des expériences professionnelles, éditable dans sa propre
 * carte. Les changements sont tenus en local (via useEditableList) jusqu'à
 * l'appel de `save()` par le parent (bouton Valider de la carte).
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import DetailedContextField from './DetailedContextField.vue'
import ProfileCard from './ProfileCard.vue'
import ProfileEntryRow from './ProfileEntryRow.vue'

// Importation des composables
import { useEditableList } from '@/composables/useEditableList'
import { typeContratOptions } from '@/composables/candidature_display'
import { months, buildYearRange, fmtRange } from './dateRangeFormat'

// Importation des types
import type { ExperienceView, CreateExperienceInput } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const props = defineProps<{
  items: ExperienceView[]
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
}>()

interface ExpDraft {
  poste: string
  entreprise: string
  lieu: string
  type_contrat: string
  debut_mois: number
  debut_annee: number
  fin_mois: number
  fin_annee: number
  en_cours: boolean
  description: string
  contexte: string
}

type ExpDisplayItem = ExperienceView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const { t } = useI18n()

/**
 * Libellé lisible (traduit) d'un type de contrat.
 *
 * @param value - Type de contrat brut (ex. `"stage"`), ou `null`.
 * @returns Libellé affichable, ou chaîne vide si absent.
 */
function typeLabel(value: string | null): string {
  if (!value) return ''
  return typeContratOptions().find((opt) => opt.value === value)?.label ?? value
}

const yearRange = buildYearRange()

function defaultExpDraft(): ExpDraft {
  const now = new Date()
  return {
    poste: '',
    entreprise: '',
    lieu: '',
    type_contrat: '',
    debut_mois: now.getMonth() + 1,
    debut_annee: now.getFullYear(),
    fin_mois: now.getMonth() + 1,
    fin_annee: now.getFullYear(),
    en_cours: false,
    description: '',
    contexte: '',
  }
}

function pad(n: number): string {
  return String(n).padStart(2, '0')
}

function draftFromView(e: ExperienceView): ExpDraft {
  const now = new Date()
  const parseDate = (d: string | null) => {
    if (!d) return { mois: now.getMonth() + 1, annee: now.getFullYear() }
    const parts = d.split('-')
    return {
      mois: parseInt(parts[1] ?? '1', 10),
      annee: parseInt(parts[0] ?? String(now.getFullYear()), 10),
    }
  }
  const debut = parseDate(e.date_debut)
  const fin = parseDate(e.date_fin)
  return {
    poste: e.poste,
    entreprise: e.entreprise ?? '',
    lieu: e.lieu ?? '',
    type_contrat: e.type_contrat ?? '',
    debut_mois: debut.mois,
    debut_annee: debut.annee,
    fin_mois: fin.mois,
    fin_annee: fin.annee,
    en_cours: e.date_fin === null,
    description: e.description ?? '',
    contexte: e.contexte ?? '',
  }
}

function displayFromDraft(d: ExpDraft, id: number, pendingKind: 'new' | 'edited'): ExpDisplayItem {
  return {
    id,
    poste: d.poste,
    entreprise: d.entreprise || null,
    lieu: d.lieu || null,
    description: d.description || null,
    date_debut: `${d.debut_annee}-${pad(d.debut_mois)}-01`,
    date_fin: d.en_cours ? null : `${d.fin_annee}-${pad(d.fin_mois)}-01`,
    type_contrat: d.type_contrat || null,
    contexte: d.contexte.trim() || null,
    isPending: true,
    pendingKind,
  }
}

function draftToInput(d: ExpDraft): CreateExperienceInput {
  return {
    poste: d.poste.trim(),
    entreprise: d.entreprise.trim() || null,
    lieu: d.lieu.trim() || null,
    description: d.description.trim() || null,
    date_debut: `${d.debut_annee}-${pad(d.debut_mois)}-01`,
    date_fin: d.en_cours ? null : `${d.fin_annee}-${pad(d.fin_mois)}-01`,
    type_contrat: d.type_contrat || null,
    contexte: d.contexte.trim() || null,
  }
}

const savedItems = computed(() => props.items)

const {
  pending,
  deletedIds,
  edited,
  modalOpen,
  draft,
  editTarget,
  allItems,
  hasChanges,
  openAddModal,
  openEditModal,
  closeModal,
  saveModal,
  deleteItem,
  reset,
} = useEditableList<ExpDisplayItem, ExpDraft>({
  savedItems,
  defaultDraft: defaultExpDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.poste.trim() !== '',
})

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste d'expériences à jour (à assigner par le parent).
 *
 * @returns Expériences à jour après synchronisation avec le backend.
 */
async function save(): Promise<ExperienceView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_experience', { id })
  }
  let result = props.items.filter((e) => !deletedIds.value.includes(e.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<ExperienceView>('update_experience', { id, input: draftToInput(d) })
    const idx = result.findIndex((e) => e.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of [...pending.value].reverse()) {
    const created = await invoke<ExperienceView>('add_experience', { input: draftToInput(d) })
    result = [created, ...result]
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.experiences.section')"
    :mode="mode"
    list
    :dirty="hasChanges"
    :saving="saving"
    :error="error"
    @edit="emit('edit')"
    @add="openAddModal"
    @cancel="emit('cancel')"
    @save="emit('save')"
  >
    <template v-if="allItems.length">
      <ProfileEntryRow
        v-for="(exp, i) in allItems"
        :key="exp.id"
        :title="exp.poste"
        :badge="typeLabel(exp.type_contrat)"
        :sub="[exp.entreprise, exp.lieu].filter(Boolean).join(' — ')"
        :period="fmtRange(exp.date_debut, exp.date_fin)"
        :description="exp.description"
        :contexte="exp.contexte"
        :pending-kind="exp.pendingKind"
        :editing="mode === 'editing'"
        :last="i === allItems.length - 1"
        @edit="openEditModal(exp)"
        @remove="deleteItem(exp)"
      />
    </template>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.experiences.empty') }}</div>
  </ProfileCard>

  <!-- Popup ajout/modification d'expérience : champs courts à gauche, textes à droite -->
  <AppModal
    v-if="modalOpen"
    :title="editTarget.kind === 'new' ? t('profile.experiences.add_title') : t('profile.experiences.edit_title')"
    :width="980"
    @close="closeModal"
  >
    <div class="grid grid-cols-1 min-[900px]:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] gap-x-8 items-start">
      <div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.experiences.job_title') }}</label>
          <input v-model="draft.poste" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.experiences.job_placeholder')" />
        </div>

        <div class="flex gap-[14px]">
          <div class="flex-1 min-w-0 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.job.company') }}</label>
            <input v-model="draft.entreprise" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('detail.job.company_placeholder')" />
          </div>
          <div class="flex-1 min-w-0 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.job.location') }}</label>
            <input v-model="draft.lieu" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.experiences.location_placeholder')" />
          </div>
        </div>

        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('detail.job.contract') }}</label>
          <select
            v-model="draft.type_contrat"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
          >
            <option value="">{{ t('common.select_placeholder') }}</option>
            <option v-for="opt in typeContratOptions()" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>

        <div class="flex gap-[14px] flex-wrap">
          <div class="flex-1 min-w-[160px] mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.experiences.start') }}</label>
            <div class="flex gap-2">
              <select v-model="draft.debut_mois" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1">
                <option v-for="(m, idx) in months()" :key="idx" :value="idx + 1">{{ m }}</option>
              </select>
              <select v-model="draft.debut_annee" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1">
                <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
              </select>
            </div>
          </div>
          <div class="flex-1 min-w-[160px] mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.experiences.end') }}</label>
            <div class="flex gap-2 items-center flex-wrap">
              <template v-if="!draft.en_cours">
                <select v-model="draft.fin_mois" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-[80px]">
                  <option v-for="(m, idx) in months()" :key="idx" :value="idx + 1">{{ m }}</option>
                </select>
                <select v-model="draft.fin_annee" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-[80px]">
                  <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
                </select>
              </template>
              <label class="flex items-center gap-[6px] text-[13px] cursor-pointer whitespace-nowrap select-none">
                <input type="checkbox" v-model="draft.en_cours" />
                {{ t('profile.in_progress') }}
              </label>
            </div>
          </div>
        </div>
      </div>

      <div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.description') }}</label>
          <textarea
            v-model="draft.description"
            class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[180px]"
            :placeholder="t('profile.experiences.description_placeholder')"
          />
          <div class="mt-[5px] text-[11.5px] text-neutral-500">{{ t('profile.description_list_help') }}</div>
        </div>
        <DetailedContextField v-model="draft.contexte" />
      </div>
    </div>

    <template #footer>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="closeModal"
      >{{ t('common.cancel') }}</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.poste.trim() }"
        type="button"
        :disabled="!draft.poste.trim()"
        @click="saveModal"
      >{{ t('common.validate') }}</button>
    </template>
  </AppModal>
</template>

