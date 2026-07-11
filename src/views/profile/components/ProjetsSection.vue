<script setup lang="ts">
/**
 * Section « Projets » du profil : affichage, ajout, modification et
 * suppression des projets personnels. Les changements sont tenus en local
 * (via useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import { insertBullet } from '@/composables/insertBullet'
import { parseBulletDescription } from '@/composables/parseBulletDescription'
import { buildYearRange, fmtYearRange } from './dateRangeFormat'
import type { ProjetView, CreateProjetInput } from '@/types/index'

const props = defineProps<{
  items: ProjetView[]
  editMode: boolean
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
  <div class="fieldset">
    <div :class="editMode ? 'fstitle-row' : ''">
      <div class="fstitle">PROJETS</div>
      <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddModal">
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(proj, i) in allItems"
        :key="proj.id"
        class="exp"
        :class="{ 'exp-pending': proj.isPending }"
        :style="i === allItems.length - 1 ? 'border-bottom:none' : ''"
      >
        <div class="explogo ph" />
        <div class="f1">
          <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
            <div class="exprole">{{ proj.titre }}</div>
            <span v-if="proj.pour_qui" class="tag">{{ proj.pour_qui }}</span>
            <span v-if="proj.isPending" class="tag exp-pending-tag">
              {{ proj.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div v-if="proj.stack" class="expco proj-stack">{{ proj.stack }}</div>
          <div class="expmeta">{{ fmtYearRange(proj.date_debut, proj.date_fin) }}</div>
          <template v-if="proj.description">
            <ul v-if="proj.description.includes('•')" class="expd-bullets">
              <li v-for="(b, bi) in parseBulletDescription(proj.description)" :key="bi">{{ b }}</li>
            </ul>
            <div v-else class="expd">{{ proj.description }}</div>
          </template>
          <button v-if="proj.url" class="proj-link" type="button" @click="invoke('open_url', { url: proj.url })">{{ proj.url }}</button>
        </div>
        <div v-if="editMode" class="exp-actions">
          <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditModal(proj)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteItem(proj)">
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
    <div v-else class="cempty">Aucun projet renseigné</div>
  </div>

  <!-- Popup ajout/modification de projet -->
  <div v-if="modalOpen" class="popup-backdrop">
    <div class="popup-box">
      <div class="popup-hd">
        <div class="popup-title">
          {{ editTarget.kind === 'new' ? 'Ajouter un projet' : 'Modifier un projet' }}
        </div>
        <button class="popup-close" type="button" @click="closeModal">×</button>
      </div>

      <div class="field">
        <label class="flabel">Titre *</label>
        <input v-model="draft.titre" class="input" placeholder="Mon super projet" />
      </div>

      <div class="field">
        <label class="flabel">Pour qui</label>
        <input v-model="draft.pour_qui" class="input" placeholder="Perso, École, Client..." />
      </div>

      <div class="field">
        <label class="flabel">Stack utilisé</label>
        <input v-model="draft.stack" class="input" placeholder="Vue 3, Rust, SQLite..." />
      </div>

      <div class="field">
        <label class="flabel">Lien</label>
        <input v-model="draft.url" class="input" placeholder="https://github.com/..." />
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

      <div class="field">
        <div class="desc-field-hd">
          <label class="flabel" style="margin-bottom:0">Description</label>
          <button
            type="button"
            class="bullet-btn"
            aria-label="Ajouter une puce"
            @click="descTextarea && insertBullet(descTextarea, draft.description, v => draft.description = v)"
          ><span aria-hidden="true">•</span> Puce</button>
        </div>
        <textarea ref="descTextarea" v-model="draft.description" class="input" rows="3" placeholder="Ce projet consiste à..."></textarea>
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

<style scoped>
.proj-stack {
  font-size: 11px;
  color: #666;
  font-family: monospace;
}

.proj-link {
  font-size: 11px;
  color: #4a90d9;
  text-decoration: none;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  display: inline-block;
  margin-top: 2px;
  word-break: break-all;
  text-align: left;
}
.proj-link:hover { text-decoration: underline; }
</style>
