<script setup lang="ts">
/**
 * Section « Compétences » du profil : affichage groupé par domaine, ajout,
 * modification et suppression, éditable dans sa propre carte. Les changements
 * sont tenus en local (via useEditableList) jusqu'à l'appel de `save()` par le
 * parent (bouton Valider de la carte).
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import PendingBadge from './PendingBadge.vue'
import ProfileCard from './ProfileCard.vue'
import RowEditButtons from './RowEditButtons.vue'

// Importation des composables
import { useEditableList } from '@/composables/useEditableList'

// Importation des types
import type { SkillView, CreateSkillInput } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const { t } = useI18n()

const props = defineProps<{
  items: SkillView[]
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
}>()

interface SkillDraft {
  nom: string
  domaine: string
}

type SkillDisplayItem = SkillView & { isPending?: true; pendingKind?: 'new' | 'edited' }

function defaultSkillDraft(): SkillDraft {
  return { nom: '', domaine: '' }
}

function draftFromView(s: SkillView): SkillDraft {
  return { nom: s.nom, domaine: s.domaine }
}

function displayFromDraft(d: SkillDraft, id: number, pendingKind: 'new' | 'edited'): SkillDisplayItem {
  return { id, nom: d.nom, domaine: d.domaine, isPending: true, pendingKind }
}

function draftToInput(d: SkillDraft): CreateSkillInput {
  return { nom: d.nom.trim(), domaine: d.domaine.trim() }
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
} = useEditableList<SkillDisplayItem, SkillDraft>({
  savedItems,
  defaultDraft: defaultSkillDraft,
  draftFromView,
  displayFromDraft,
  isDraftValid: (d) => d.nom.trim() !== '' && d.domaine.trim() !== '',
  insertPendingAt: 'end',
})

/** Compétences affichées, groupées par domaine puis triées par ordre d'ajout. */
const skillsByDomain = computed<Record<string, SkillDisplayItem[]>>(() => {
  const groups: Record<string, SkillDisplayItem[]> = {}
  for (const s of allItems.value) {
    if (!groups[s.domaine]) groups[s.domaine] = []
    groups[s.domaine].push(s)
  }
  return groups
})

const domainDropdownOpen = ref(false)

/** Domaines déjà utilisés parmi les compétences affichées (pour le combobox). */
const existingDomains = computed<string[]>(() => {
  const set = new Set<string>()
  for (const s of allItems.value) {
    if (s.domaine) set.add(s.domaine)
  }
  return Array.from(set).sort()
})

const filteredDomains = computed<string[]>(() => {
  const q = draft.value.domaine.trim().toLowerCase()
  if (!q) return existingDomains.value
  return existingDomains.value.filter((d) => d.toLowerCase().includes(q))
})

const exactDomainMatch = computed(() =>
  existingDomains.value.some((d) => d.toLowerCase() === draft.value.domaine.trim().toLowerCase()),
)

function pickDomain(d: string) {
  draft.value.domaine = d
  domainDropdownOpen.value = false
}

function onDomainBlur() {
  setTimeout(() => { domainDropdownOpen.value = false }, 150)
}

function openAdd() {
  domainDropdownOpen.value = false
  openAddModal()
}

function openEdit(item: SkillDisplayItem) {
  domainDropdownOpen.value = false
  openEditModal(item)
}

function close() {
  closeModal()
  domainDropdownOpen.value = false
}

/**
 * Sauvegarde les suppressions, modifications et ajouts en attente, puis
 * retourne la liste de compétences à jour (à assigner par le parent).
 *
 * @returns Compétences à jour après synchronisation avec le backend.
 */
async function save(): Promise<SkillView[]> {
  for (const id of deletedIds.value) {
    await invoke('delete_skill', { id })
  }
  const result = props.items.filter((s) => !deletedIds.value.includes(s.id))

  for (const [rawId, d] of Object.entries(edited.value)) {
    const id = Number(rawId)
    const updated = await invoke<SkillView>('update_skill', { id, input: draftToInput(d) })
    const idx = result.findIndex((s) => s.id === id)
    if (idx >= 0) result[idx] = updated
  }

  for (const d of pending.value) {
    const created = await invoke<SkillView>('add_skill', { input: draftToInput(d) })
    result.push(created)
  }

  reset()
  return result
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.skills.section')"
    :mode="mode"
    list
    :dirty="hasChanges"
    :saving="saving"
    :error="error"
    @edit="emit('edit')"
    @add="openAdd"
    @cancel="emit('cancel')"
    @save="emit('save')"
  >
    <template v-if="allItems.length">
      <!-- Lecture : pastilles groupées par domaine -->
      <template v-if="mode !== 'editing'">
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="font-mono text-[11px] text-neutral-500 mb-[6px] mt-3">{{ domaine }}</div>
          <div class="flex flex-wrap gap-[6px]">
            <span v-for="s in items" :key="s.id" class="border border-neutral-300 rounded-full px-[10px] py-1 text-[12px] text-neutral-800 bg-neutral-50">{{ s.nom }}</span>
          </div>
        </template>
      </template>

      <!-- Édition : une ligne par compétence avec ses boutons -->
      <template v-else>
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="font-mono text-[11px] text-neutral-500 mt-3 mb-1">{{ domaine }}</div>
          <div
            v-for="s in items"
            :key="s.id"
            class="flex items-center gap-2 py-[6px]"
            :class="{ 'opacity-[0.72]': s.isPending }"
          >
            <span class="flex-1 min-w-0 text-[13px] text-neutral-950">{{ s.nom }}</span>
            <PendingBadge v-if="s.pendingKind" :kind="s.pendingKind" />
            <RowEditButtons @edit="openEdit(s)" @remove="deleteItem(s)" />
          </div>
        </template>
      </template>
    </template>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.skills.empty') }}</div>
  </ProfileCard>

  <!-- Popup ajout/modification de compétence (sans défilement : la liste des domaines déborde) -->
  <AppModal
    v-if="modalOpen"
    :title="editTarget.kind === 'new' ? t('profile.skills.add_title') : t('profile.skills.edit_title')"
    :width="520"
    :scroll="false"
    @close="close"
  >
    <div class="mb-[14px]">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.skills.name') }}</label>
      <input v-model="draft.nom" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Python, Vue.js, Figma…" />
    </div>

    <div class="mb-0">
      <label class="text-[12px] text-neutral-600 mb-[5px] block">{{ t('profile.skills.domain') }}</label>
      <div class="relative">
        <input
          v-model="draft.domaine"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400"
          :placeholder="t('profile.skills.domain_placeholder')"
          autocomplete="off"
          @focus="domainDropdownOpen = true"
          @input="domainDropdownOpen = true"
          @blur="onDomainBlur"
        />
        <div
          v-if="domainDropdownOpen && (filteredDomains.length > 0 || draft.domaine.trim())"
          class="absolute top-[calc(100%+4px)] left-0 right-0 bg-surface-3 border border-neutral-300 rounded-lg shadow-[0_4px_20px_rgba(0,0,0,.12)] z-[200] overflow-hidden max-h-[200px] overflow-y-auto"
        >
          <div
            v-for="d in filteredDomains"
            :key="d"
            class="px-[13px] py-[9px] text-[13.5px] cursor-pointer text-neutral-950 hover:bg-surface-2"
            @mousedown.prevent="pickDomain(d)"
          >{{ d }}</div>
          <div
            v-if="draft.domaine.trim() && !exactDomainMatch"
            class="px-[13px] py-[9px] text-[13.5px] cursor-pointer border-t border-neutral-100 text-green-500 flex gap-1 hover:bg-green-50"
            @mousedown.prevent="domainDropdownOpen = false"
          >
            <i18n-t keypath="profile.skills.create" scope="global">
              <template #name><strong>{{ t('profile.skills.quoted', { name: draft.domaine.trim() }) }}</strong></template>
            </i18n-t>
          </div>
          <div
            v-if="!filteredDomains.length && !draft.domaine.trim()"
            class="px-[13px] py-[9px] text-[12px] text-neutral-400 font-mono cursor-default"
          >{{ t('profile.skills.no_domain') }}</div>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="close"
      >{{ t('common.cancel') }}</button>
      <button
        class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
        :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.nom.trim() || !draft.domaine.trim() }"
        type="button"
        :disabled="!draft.nom.trim() || !draft.domaine.trim()"
        @click="saveModal"
      >{{ t('common.validate') }}</button>
    </template>
  </AppModal>
</template>
