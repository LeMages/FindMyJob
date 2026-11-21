<script setup lang="ts">
/**
 * Section « Langues » du profil : affichage, ajout, modification et
 * suppression des langues, éditable dans sa propre carte. Les changements
 * sont tenus en local (via useEditableList) jusqu'à l'appel de `save()` par le
 * parent (bouton Valider de la carte).
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import PendingBadge from './PendingBadge.vue'
import ProfileCard from './ProfileCard.vue'
import RowEditButtons from './RowEditButtons.vue'

// Importation des composables
import { useEditableList } from '@/composables/useEditableList'

// Importation des types
import type { LangView, CreateLangInput } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const props = defineProps<{
  items: LangView[]
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
}>()

interface LangDraft {
  nom: string
  level: string
}

type LangDisplayItem = LangView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const { t } = useI18n()

// Niveaux stockés en base sous leur libellé français (`langues.level`) : la
// valeur ne change jamais, seul l'affichage est traduit via un identifiant
// stable (`profile.languages.level.<key>`).
const LANG_LEVELS: { value: string; key: string }[] = [
  { value: 'Langue maternelle', key: 'native' },
  { value: 'Bilingue', key: 'bilingual' },
  { value: 'Courant (C2)', key: 'c2' },
  { value: 'Avancé (C1)', key: 'c1' },
  { value: 'Intermédiaire avancé (B2)', key: 'b2' },
  { value: 'Intermédiaire (B1)', key: 'b1' },
  { value: 'Élémentaire (A2)', key: 'a2' },
  { value: 'Débutant (A1)', key: 'a1' },
]

/** Options du sélecteur de niveau, libellés traduits (re-calculés au changement de langue). */
const levelOptions = computed(() =>
  LANG_LEVELS.map(({ value, key }) => ({ value, label: t(`profile.languages.level.${key}`) })),
)

/**
 * Libellé affiché d'un niveau stocké : traduit s'il fait partie de la liste,
 * tel quel sinon (saisie ancienne ou libre).
 */
function levelLabel(level: string | null): string {
  if (!level) return ''
  const known = LANG_LEVELS.find((l) => l.value === level)
  return known ? t(`profile.languages.level.${known.key}`) : level
}

function defaultLangDraft(): LangDraft {
  return { nom: '', level: '' }
}

function draftFromView(l: LangView): LangDraft {
  return { nom: l.nom ?? '', level: l.level ?? '' }
}

function displayFromDraft(d: LangDraft, id: number, pendingKind: 'new' | 'edited'): LangDisplayItem {
  return { id, nom: d.nom || null, level: d.level || null, isPending: true, pendingKind }
}

function draftToInput(d: LangDraft): CreateLangInput {
  return { nom: d.nom.trim(), level: d.level.trim() || null }
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
} = useEditableList<LangDisplayItem, LangDraft>({
  savedItems,
  defaultDraft: defaultLangDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.nom.trim() !== '',
  insertPendingAt: 'end',
})

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de langues à jour (à assigner par le parent).
 *
 * @returns Langues à jour après synchronisation avec le backend.
 */
async function save(): Promise<LangView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_lang', { id })
  }
  const result = props.items.filter((l) => !deletedIds.value.includes(l.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<LangView>('update_lang', { id, input: draftToInput(d) })
    const idx = result.findIndex((l) => l.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of pending.value) {
    const created = await invoke<LangView>('add_lang', { input: draftToInput(d) })
    result.push(created)
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.languages.section')"
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
    <div class="mt-2">
      <template v-if="allItems.length">
        <div
          v-for="lang in allItems"
          :key="lang.id"
          class="flex items-center gap-3 py-2 border-b border-neutral-100 last:border-b-0"
          :class="{ 'opacity-[0.72]': lang.isPending }"
        >
          <span class="text-[13.5px] font-medium flex-1 min-w-0">{{ lang.nom ?? '—' }}</span>
          <span class="font-mono text-[11px] text-neutral-500 text-right">{{ levelLabel(lang.level) }}</span>
          <PendingBadge v-if="lang.pendingKind" :kind="lang.pendingKind" />
          <RowEditButtons v-if="mode === 'editing'" @edit="openEditModal(lang)" @remove="deleteItem(lang)" />
        </div>
      </template>
      <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.languages.empty') }}</div>
    </div>
  </ProfileCard>

  <!-- Popup ajout/modification de langue -->
  <AppModal
    v-if="modalOpen"
    :title="editTarget.kind === 'new' ? t('profile.languages.add_title') : t('profile.languages.edit_title')"
    :width="500"
    @close="closeModal"
  >
    <div class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.languages.name') }}</label>
      <input v-model="draft.nom" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" :placeholder="t('profile.languages.name_placeholder')" />
    </div>

    <div class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.languages.level_label') }}</label>
      <select
        v-model="draft.level"
        class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
      >
        <option value="">{{ t('common.select_placeholder') }}</option>
        <option v-for="lvl in levelOptions" :key="lvl.value" :value="lvl.value">{{ lvl.label }}</option>
      </select>
    </div>

    <template #footer>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="closeModal"
      >{{ t('common.cancel') }}</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.nom.trim() }"
        type="button"
        :disabled="!draft.nom.trim()"
        @click="saveModal"
      >{{ t('common.validate') }}</button>
    </template>
  </AppModal>
</template>
