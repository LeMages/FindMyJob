<script setup lang="ts">
/**
 * Section « Compétences » du profil : affichage groupé par domaine, ajout,
 * modification et suppression. Les changements sont tenus en local (via
 * useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import type { SkillView, CreateSkillInput } from '@/types/index'

const { t } = useI18n()

const props = defineProps<{
  items: SkillView[]
  editMode: boolean
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
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div :class="editMode ? 'flex items-center justify-between mb-[14px]' : ''">
      <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500" :class="editMode ? '' : 'mb-[14px]'">{{ t('profile.skills.section') }}</div>
      <button
        v-if="editMode"
        class="h-[30px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-surface-2 font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="openAdd"
      >
        {{ t('common.add') }}
      </button>
    </div>

    <!-- Vue normale : chips groupées par domaine -->
    <template v-if="!editMode">
      <template v-if="allItems.length">
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="font-mono text-[11px] text-neutral-500 mb-[6px] mt-[10px]">{{ domaine }}</div>
          <div class="flex flex-wrap gap-2 mb-1">
            <span v-for="s in items" :key="s.id" class="border border-neutral-300 rounded-full px-3 py-[5px] text-[12.5px] text-neutral-800 bg-neutral-50">{{ s.nom }}</span>
          </div>
        </template>
      </template>
      <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.skills.empty') }}</div>
    </template>

    <!-- Vue édition : liste avec boutons édition/suppression -->
    <template v-else>
      <template v-if="allItems.length">
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="font-mono text-[11px] text-neutral-500 mt-3 mb-1">{{ domaine }}</div>
          <div
            v-for="s in items"
            :key="s.id"
            class="flex items-center gap-2 py-[7px] border-b border-neutral-50 last:border-b-0"
            :class="{ 'opacity-[0.72]': s.isPending }"
          >
            <span class="flex-1 text-[13px] text-neutral-950">{{ s.nom }}</span>
            <span v-if="s.isPending" class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50 flex-none">
              {{ s.pendingKind === 'edited' ? t('profile.pending.edited') : t('profile.pending.new') }}
            </span>
            <div class="flex flex-row gap-[5px] flex-none self-center ml-[6px]">
              <button
                class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-surface-3 hover:text-neutral-950 hover:border-neutral-300"
                type="button"
                :title="t('common.edit')"
                @click="openEdit(s)"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
              </button>
              <button
                class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
                type="button"
                :title="t('common.delete')"
                @click="deleteItem(s)"
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
        </template>
      </template>
      <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('profile.skills.empty') }}</div>
    </template>
  </div>

  <!-- Popup ajout/modification de compétence -->
  <div v-if="modalOpen" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[440px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">
          {{ editTarget.kind === 'new' ? t('profile.skills.add_title') : t('profile.skills.edit_title') }}
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="close">×</button>
      </div>

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

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
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
      </div>
    </div>
  </div>
</template>
