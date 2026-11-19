<script setup lang="ts">
/**
 * Section « Formation » du profil : affichage, ajout, modification et
 * suppression des formations, éditable dans sa propre carte. Les changements
 * sont tenus en local (via useEditableList) jusqu'à l'appel de `save()` par le
 * parent (bouton Valider de la carte).
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import ProfileCard from './ProfileCard.vue'
import ProfileEntryRow from './ProfileEntryRow.vue'

// Importation des composables
import { useEditableList } from '@/composables/useEditableList'
import { buildYearRange, fmtYearRange } from './dateRangeFormat'

// Importation des types
import type { FormationView, CreateFormationInput } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const { t } = useI18n()

const props = defineProps<{
  items: FormationView[]
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
}>()

interface FormDraft {
  titre: string
  ecole: string
  description: string
  debut_annee: number
  fin_annee: number
  en_cours: boolean
}

type FormDisplayItem = FormationView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const yearRange = buildYearRange()

function toDateStr(y: number): string {
  return `${y}-01-01`
}

function defaultFormDraft(): FormDraft {
  return {
    titre: '',
    ecole: '',
    description: '',
    debut_annee: new Date().getFullYear(),
    fin_annee: new Date().getFullYear(),
    en_cours: false,
  }
}

function draftFromView(f: FormationView): FormDraft {
  const curYear = new Date().getFullYear()
  const parseYear = (d: string | null) => (d ? parseInt(d.split('-')[0] ?? String(curYear), 10) : curYear)
  return {
    titre: f.titre,
    ecole: f.ecole ?? '',
    description: f.description ?? '',
    debut_annee: parseYear(f.date_debut),
    fin_annee: parseYear(f.date_fin),
    en_cours: f.date_fin === null,
  }
}

function displayFromDraft(d: FormDraft, id: number, pendingKind: 'new' | 'edited'): FormDisplayItem {
  return {
    id,
    titre: d.titre,
    ecole: d.ecole || null,
    description: d.description || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
    isPending: true,
    pendingKind,
  }
}

function draftToInput(d: FormDraft): CreateFormationInput {
  return {
    titre: d.titre.trim(),
    ecole: d.ecole.trim() || null,
    description: d.description.trim() || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
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
} = useEditableList<FormDisplayItem, FormDraft>({
  savedItems,
  defaultDraft: defaultFormDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.titre.trim() !== '',
})

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de formations à jour (à assigner par le parent).
 *
 * @returns Formations à jour après synchronisation avec le backend.
 */
async function save(): Promise<FormationView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_formation', { id })
  }
  let result = props.items.filter((f) => !deletedIds.value.includes(f.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<FormationView>('update_formation', { id, input: draftToInput(d) })
    const idx = result.findIndex((f) => f.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of [...pending.value].reverse()) {
    const created = await invoke<FormationView>('add_formation', { input: draftToInput(d) })
    result = [created, ...result]
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.formations.section')"
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
        v-for="(form, i) in allItems"
        :key="form.id"
        :title="form.titre"
        :sub="form.ecole"
        :period="fmtYearRange(form.date_debut, form.date_fin)"
        :description="form.description"
        description-style="lines"
        :pending-kind="form.pendingKind"
        :editing="mode === 'editing'"
        :last="i === allItems.length - 1"
        @edit="openEditModal(form)"
        @remove="deleteItem(form)"
      />
    </template>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.formations.empty') }}</div>
  </ProfileCard>

  <!-- Popup ajout/modification de formation : champs courts à gauche, détails à droite -->
  <AppModal
    v-if="modalOpen"
    :title="editTarget.kind === 'new' ? t('profile.formations.add_title') : t('profile.formations.edit_title')"
    :width="860"
    @close="closeModal"
  >
    <div class="grid grid-cols-1 min-[800px]:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] gap-x-8 items-start">
      <div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.formations.title') }}</label>
          <input v-model="draft.titre" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.formations.title_placeholder')" />
        </div>

        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.formations.school') }}</label>
          <input v-model="draft.ecole" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.formations.school_placeholder')" />
        </div>

        <div class="flex gap-[14px]">
          <div class="flex-1 min-w-0 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.start_year') }}</label>
            <select
              v-model="draft.debut_annee"
              class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
            >
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
          </div>
          <div class="flex-1 min-w-0 mb-[14px]">
            <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.end_year') }}</label>
            <div class="flex gap-2 items-center">
              <select
                v-if="!draft.en_cours"
                v-model="draft.fin_annee"
                class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer flex-1 bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
              >
                <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
              </select>
              <label class="flex items-center gap-[6px] text-[13px] cursor-pointer whitespace-nowrap select-none" :class="draft.en_cours ? '' : 'flex-none'">
                <input type="checkbox" v-model="draft.en_cours" />
                {{ t('profile.in_progress') }}
              </label>
            </div>
          </div>
        </div>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.formations.details') }}</label>
        <textarea v-model="draft.description" class="w-full h-[180px] border border-neutral-300 rounded-lg px-3 py-2 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 resize-y" :placeholder="t('profile.formations.details_placeholder')"></textarea>
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
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.titre.trim() }"
        type="button"
        :disabled="!draft.titre.trim()"
        @click="saveModal"
      >{{ t('common.validate') }}</button>
    </template>
  </AppModal>
</template>
