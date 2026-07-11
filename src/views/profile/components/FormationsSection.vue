<script setup lang="ts">
/**
 * Section « Formation » du profil : affichage, ajout, modification et
 * suppression des formations. Les changements sont tenus en local (via
 * useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import { buildYearRange, fmtYearRange } from './dateRangeFormat'
import type { FormationView, CreateFormationInput } from '@/types/index'

const props = defineProps<{
  items: FormationView[]
  editMode: boolean
}>()

interface FormDraft {
  titre: string
  ecole: string
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
  <div class="fieldset">
    <div :class="editMode ? 'fstitle-row' : ''">
      <div class="fstitle">FORMATION</div>
      <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddModal">
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(form, i) in allItems"
        :key="form.id"
        class="exp"
        :class="{ 'exp-pending': form.isPending }"
        :style="i === allItems.length - 1 ? 'border-bottom:none' : ''"
      >
        <div class="explogo ph" />
        <div class="f1">
          <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
            <div class="exprole">{{ form.titre }}</div>
            <span v-if="form.isPending" class="tag exp-pending-tag">
              {{ form.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div v-if="form.ecole" class="expco">{{ form.ecole }}</div>
          <div class="expmeta">{{ fmtYearRange(form.date_debut, form.date_fin) }}</div>
        </div>
        <div v-if="editMode" class="exp-actions">
          <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditModal(form)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteItem(form)">
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
    <div v-else class="cempty">Aucune formation renseignée</div>
  </div>

  <!-- Popup ajout/modification de formation -->
  <div v-if="modalOpen" class="popup-backdrop">
    <div class="popup-box">
      <div class="popup-hd">
        <div class="popup-title">
          {{ editTarget.kind === 'new' ? 'Ajouter une formation' : 'Modifier une formation' }}
        </div>
        <button class="popup-close" type="button" @click="closeModal">×</button>
      </div>

      <div class="field">
        <label class="flabel">Titre / Diplôme *</label>
        <input v-model="draft.titre" class="input" placeholder="Master Informatique" />
      </div>

      <div class="field">
        <label class="flabel">École / Établissement</label>
        <input v-model="draft.ecole" class="input" placeholder="EFREI Paris" />
      </div>

      <div class="frow">
        <div class="field">
          <label class="flabel">Année de début</label>
          <select v-model="draft.debut_annee" class="input">
            <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
          </select>
        </div>
        <div class="field">
          <label class="flabel">Année de fin</label>
          <div style="display:flex;gap:8px;align-items:center">
            <select v-if="!draft.en_cours" v-model="draft.fin_annee" class="input" style="flex:1">
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
            <label class="en-cours-label" :style="draft.en_cours ? '' : 'flex:none'">
              <input type="checkbox" v-model="draft.en_cours" />
              En cours
            </label>
          </div>
        </div>
      </div>

      <div class="popup-foot">
        <button class="btn" type="button" @click="closeModal">Annuler</button>
        <button
          class="btn edit-save"
          type="button"
          :class="{ disabled: !draft.titre.trim() }"
          :disabled="!draft.titre.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
