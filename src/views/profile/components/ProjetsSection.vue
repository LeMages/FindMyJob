<script setup lang="ts">
/**
 * Section « Projets » du profil : affichage, ajout, modification et
 * suppression des projets personnels, éditable dans sa propre carte. Les
 * changements sont tenus en local (via useEditableList) jusqu'à l'appel de
 * `save()` par le parent (bouton Valider de la carte).
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import DetailedContextField from './DetailedContextField.vue'
import ProfileCard from './ProfileCard.vue'
import ProfileEntryRow from './ProfileEntryRow.vue'

// Importation des composables
import { useEditableList } from '@/composables/useEditableList'
import { insertBullet } from '@/composables/insertBullet'
import { buildYearRange, fmtYearRange } from './dateRangeFormat'

// Importation des types
import type { ProjetView, CreateProjetInput } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const { t } = useI18n()

const props = defineProps<{
  items: ProjetView[]
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
  'open-url': [url: string]
}>()

interface ProjetDraft {
  titre: string
  description: string
  stack: string
  url: string
  pour_qui: string
  debut_annee: number
  fin_annee: number
  en_cours: boolean
  contexte: string
}

type ProjetDisplayItem = ProjetView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const yearRange = buildYearRange()

function toDateStr(y: number): string {
  return `${y}-01-01`
}

function defaultProjetDraft(): ProjetDraft {
  return {
    titre: '',
    description: '',
    stack: '',
    url: '',
    pour_qui: '',
    debut_annee: new Date().getFullYear(),
    fin_annee: new Date().getFullYear(),
    en_cours: false,
    contexte: '',
  }
}

function draftFromView(p: ProjetView): ProjetDraft {
  const curYear = new Date().getFullYear()
  const parseYear = (d: string | null) => (d ? parseInt(d.split('-')[0] ?? String(curYear), 10) : curYear)
  return {
    titre: p.titre,
    description: p.description ?? '',
    stack: p.stack ?? '',
    url: p.url ?? '',
    pour_qui: p.pour_qui ?? '',
    debut_annee: parseYear(p.date_debut),
    fin_annee: parseYear(p.date_fin),
    en_cours: p.date_fin === null,
    contexte: p.contexte ?? '',
  }
}

function displayFromDraft(d: ProjetDraft, id: number, pendingKind: 'new' | 'edited'): ProjetDisplayItem {
  return {
    id,
    titre: d.titre,
    description: d.description || null,
    stack: d.stack || null,
    url: d.url || null,
    pour_qui: d.pour_qui || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
    contexte: d.contexte.trim() || null,
    isPending: true,
    pendingKind,
  }
}

function draftToInput(d: ProjetDraft): CreateProjetInput {
  return {
    titre: d.titre.trim(),
    description: d.description.trim() || null,
    stack: d.stack.trim() || null,
    url: d.url.trim() || null,
    pour_qui: d.pour_qui.trim() || null,
    date_debut: toDateStr(d.debut_annee),
    date_fin: d.en_cours ? null : toDateStr(d.fin_annee),
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
} = useEditableList<ProjetDisplayItem, ProjetDraft>({
  savedItems,
  defaultDraft: defaultProjetDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.titre.trim() !== '',
})

const descTextarea = ref<HTMLTextAreaElement | null>(null)

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de projets à jour (à assigner par le parent).
 *
 * @returns Projets à jour après synchronisation avec le backend.
 */
async function save(): Promise<ProjetView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_projet', { id })
  }
  let result = props.items.filter((p) => !deletedIds.value.includes(p.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<ProjetView>('update_projet', { id, input: draftToInput(d) })
    const idx = result.findIndex((p) => p.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of [...pending.value].reverse()) {
    const created = await invoke<ProjetView>('add_projet', { input: draftToInput(d) })
    result = [created, ...result]
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.projects.section')"
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
        v-for="(proj, i) in allItems"
        :key="proj.id"
        :title="proj.titre"
        :badge="proj.pour_qui"
        :mono="proj.stack"
        :period="fmtYearRange(proj.date_debut, proj.date_fin)"
        :description="proj.description"
        :url="proj.url"
        :contexte="proj.contexte"
        :pending-kind="proj.pendingKind"
        :editing="mode === 'editing'"
        :last="i === allItems.length - 1"
        @edit="openEditModal(proj)"
        @remove="deleteItem(proj)"
        @open-url="url => emit('open-url', url)"
      />
    </template>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.projects.empty') }}</div>
  </ProfileCard>

  <!-- Popup ajout/modification de projet : champs courts à gauche, textes à droite -->
  <AppModal
    v-if="modalOpen"
    :title="editTarget.kind === 'new' ? t('profile.projects.add_title') : t('profile.projects.edit_title')"
    :width="980"
    @close="closeModal"
  >
    <div class="grid grid-cols-1 min-[900px]:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] gap-x-8 items-start">
      <div>
        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.projects.title') }}</label>
          <input v-model="draft.titre" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.projects.title_placeholder')" />
        </div>

        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.projects.for_whom') }}</label>
          <input v-model="draft.pour_qui" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.projects.for_whom_placeholder')" />
        </div>

        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.projects.stack') }}</label>
          <input v-model="draft.stack" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Vue 3, Rust, SQLite..." />
        </div>

        <div class="mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.projects.link') }}</label>
          <input v-model="draft.url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://github.com/..." />
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

      <div>
        <div class="mb-[14px]">
          <div class="flex items-center justify-between mb-[5px]">
            <label class="text-[12px] text-neutral-600 mb-0 block">{{ t('profile.description') }}</label>
            <button
              type="button"
              class="inline-flex items-center gap-1 h-6 px-[9px] text-[12px] rounded-5 border border-neutral-300 bg-surface-2 text-neutral-700 cursor-pointer leading-none transition-colors duration-[120ms] hover:bg-surface-3 hover:border-neutral-300 hover:text-neutral-950"
              :aria-label="t('profile.bullet.add')"
              @click="descTextarea && insertBullet(descTextarea, draft.description, v => draft.description = v)"
            ><span aria-hidden="true">•</span> {{ t('profile.bullet.label') }}</button>
          </div>
          <textarea
            ref="descTextarea"
            v-model="draft.description"
            class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[180px]"
            :placeholder="t('profile.projects.description_placeholder')"
          ></textarea>
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
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.titre.trim() }"
        type="button"
        :disabled="!draft.titre.trim()"
        @click="saveModal"
      >{{ t('common.validate') }}</button>
    </template>
  </AppModal>
</template>
