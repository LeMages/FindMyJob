<script setup lang="ts">
/**
 * Section « Compétences » du profil : affichage groupé par domaine, ajout,
 * modification et suppression. Les changements sont tenus en local (via
 * useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import type { SkillView, CreateSkillInput } from '@/types/index'

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
  <div class="fieldset">
    <div :class="editMode ? 'fstitle-row' : ''">
      <div class="fstitle">COMPÉTENCES</div>
      <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAdd">
        + Ajouter
      </button>
    </div>

    <!-- Vue normale : chips groupées par domaine -->
    <template v-if="!editMode">
      <template v-if="allItems.length">
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="cmeta" style="margin-bottom:6px;margin-top:10px">{{ domaine }}</div>
          <div class="chips" style="margin-bottom:4px">
            <span v-for="s in items" :key="s.id" class="chip">{{ s.nom }}</span>
          </div>
        </template>
      </template>
      <div v-else class="cempty">Aucune compétence renseignée</div>
    </template>

    <!-- Vue édition : liste avec boutons édition/suppression -->
    <template v-else>
      <template v-if="allItems.length">
        <template v-for="(items, domaine) in skillsByDomain" :key="domaine">
          <div class="cmeta" style="margin-top:12px;margin-bottom:4px">{{ domaine }}</div>
          <div
            v-for="s in items"
            :key="s.id"
            class="skill-edit-row"
            :class="{ 'exp-pending': s.isPending }"
          >
            <span class="skill-nom">{{ s.nom }}</span>
            <span v-if="s.isPending" class="tag exp-pending-tag" style="flex:none">
              {{ s.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
            <div class="exp-actions">
              <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEdit(s)">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
              </button>
              <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteItem(s)">
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
      <div v-else class="cempty">Aucune compétence renseignée</div>
    </template>
  </div>

  <!-- Popup ajout/modification de compétence -->
  <div v-if="modalOpen" class="popup-backdrop">
    <div class="popup-box" style="width:440px">
      <div class="popup-hd">
        <div class="popup-title">
          {{ editTarget.kind === 'new' ? 'Ajouter une compétence' : 'Modifier une compétence' }}
        </div>
        <button class="popup-close" type="button" @click="close">×</button>
      </div>

      <div class="field">
        <label class="flabel">Nom de la compétence *</label>
        <input v-model="draft.nom" class="input" placeholder="Python, Vue.js, Figma…" />
      </div>

      <div class="field" style="margin-bottom:0">
        <label class="flabel">Domaine *</label>
        <div class="domain-combo-wrap">
          <input
            v-model="draft.domaine"
            class="input"
            placeholder="Langages, Frameworks, Outils…"
            autocomplete="off"
            @focus="domainDropdownOpen = true"
            @input="domainDropdownOpen = true"
            @blur="onDomainBlur"
          />
          <div
            v-if="domainDropdownOpen && (filteredDomains.length > 0 || draft.domaine.trim())"
            class="domain-dropdown"
          >
            <div
              v-for="d in filteredDomains"
              :key="d"
              class="domain-opt"
              @mousedown.prevent="pickDomain(d)"
            >{{ d }}</div>
            <div
              v-if="draft.domaine.trim() && !exactDomainMatch"
              class="domain-opt domain-opt-new"
              @mousedown.prevent="domainDropdownOpen = false"
            >
              Créer <strong>« {{ draft.domaine.trim() }} »</strong>
            </div>
            <div
              v-if="!filteredDomains.length && !draft.domaine.trim()"
              class="domain-opt-empty"
            >Aucun domaine existant</div>
          </div>
        </div>
      </div>

      <div class="popup-foot">
        <button class="btn" type="button" @click="close">Annuler</button>
        <button
          class="btn edit-save"
          type="button"
          :class="{ disabled: !draft.nom.trim() || !draft.domaine.trim() }"
          :disabled="!draft.nom.trim() || !draft.domaine.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skill-edit-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 0;
  border-bottom: 1px solid #f5f5f2;
}
.skill-edit-row:last-child { border-bottom: none; }

.skill-nom {
  flex: 1;
  font-size: 13px;
  color: #1d1d1b;
}

.domain-combo-wrap { position: relative; }

.domain-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid #d4d4ce;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0,0,0,.12);
  z-index: 200;
  overflow: hidden;
  max-height: 200px;
  overflow-y: auto;
}

.domain-opt {
  padding: 9px 13px;
  font-size: 13.5px;
  cursor: pointer;
  color: #1d1d1b;
}
.domain-opt:hover { background: #f5f5f2; }

.domain-opt-new {
  border-top: 1px solid #f0f0ed;
  color: #2a9e5f;
  display: flex;
  gap: 4px;
}
.domain-opt-new:hover { background: #f0faf4; }

.domain-opt-empty {
  padding: 9px 13px;
  font-size: 12px;
  color: #aeaea7;
  font-family: 'IBM Plex Mono', monospace;
  cursor: default;
}
</style>
