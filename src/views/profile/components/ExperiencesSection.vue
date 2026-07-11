<script setup lang="ts">
/**
 * Section « Expériences » du profil : affichage, ajout, modification et
 * suppression des expériences professionnelles. Les changements sont tenus
 * en local (via useEditableList) jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { useEditableList } from '@/composables/useEditableList'
import { insertBullet } from '@/composables/insertBullet'
import { parseBulletDescription } from '@/composables/parseBulletDescription'
import { MONTHS, buildYearRange, fmtRange } from './dateRangeFormat'
import type { ExperienceView, CreateExperienceInput } from '@/types/index'

const props = defineProps<{
  items: ExperienceView[]
  editMode: boolean
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
}

type ExpDisplayItem = ExperienceView & { isPending?: true; pendingKind?: 'new' | 'edited' }

const TYPE_LABELS: Record<string, string> = {
  stage: 'Stage',
  alternance: 'Alternance',
  interim: 'Intérim',
  cdd: 'CDD',
  cdi: 'CDI',
}

/**
 * Libellé lisible d'un type de contrat.
 *
 * @param t - Type de contrat brut (ex. `"stage"`), ou `null`.
 * @returns Libellé affichable, ou chaîne vide si absent.
 */
function typeLabel(t: string | null): string {
  if (!t) return ''
  return TYPE_LABELS[t] ?? t
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

const descTextarea = ref<HTMLTextAreaElement | null>(null)

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
  <div class="fieldset">
    <div :class="editMode ? 'fstitle-row' : ''">
      <div class="fstitle">EXPÉRIENCES</div>
      <button v-if="editMode" class="btn exp-add-btn" type="button" @click="openAddModal">
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(exp, i) in allItems"
        :key="exp.id"
        class="exp"
        :class="{ 'exp-pending': exp.isPending }"
        :style="i === allItems.length - 1 ? 'border-bottom:none' : ''"
      >
        <div class="explogo ph" />
        <div class="f1">
          <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
            <div class="exprole">{{ exp.poste }}</div>
            <span v-if="exp.type_contrat" class="tag">{{ typeLabel(exp.type_contrat) }}</span>
            <span v-if="exp.isPending" class="tag exp-pending-tag">
              {{ exp.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div class="expco">
            {{ [exp.entreprise, exp.lieu].filter(Boolean).join(' — ') || '—' }}
          </div>
          <div class="expmeta">{{ fmtRange(exp.date_debut, exp.date_fin) }}</div>
          <template v-if="exp.description">
            <ul v-if="exp.description.includes('•')" class="expd-bullets">
              <li v-for="(b, bi) in parseBulletDescription(exp.description)" :key="bi">{{ b }}</li>
            </ul>
            <div v-else class="expd">{{ exp.description }}</div>
          </template>
        </div>
        <!-- Boutons édition (visibles uniquement en mode édition) -->
        <div v-if="editMode" class="exp-actions">
          <button class="exp-btn exp-btn-edit" type="button" title="Modifier" @click="openEditModal(exp)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="exp-btn exp-btn-delete" type="button" title="Supprimer" @click="deleteItem(exp)">
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
    <div v-else class="cempty">Aucune expérience renseignée</div>
  </div>

  <!-- Popup ajout/modification d'expérience -->
  <div v-if="modalOpen" class="popup-backdrop">
    <div class="popup-box">
      <div class="popup-hd">
        <div class="popup-title">
          {{ editTarget.kind === 'new' ? 'Ajouter une expérience' : 'Modifier une expérience' }}
        </div>
        <button class="popup-close" type="button" @click="closeModal">×</button>
      </div>

      <div class="field">
        <label class="flabel">Intitulé du poste *</label>
        <input v-model="draft.poste" class="input" placeholder="Développeur Full-Stack" />
      </div>

      <div class="frow">
        <div class="field">
          <label class="flabel">Entreprise</label>
          <input v-model="draft.entreprise" class="input" placeholder="Nom de l'entreprise" />
        </div>
        <div class="field">
          <label class="flabel">Lieu</label>
          <input v-model="draft.lieu" class="input" placeholder="Paris, France" />
        </div>
      </div>

      <div class="field">
        <label class="flabel">Type de contrat</label>
        <select v-model="draft.type_contrat" class="input">
          <option value="">-- Sélectionner --</option>
          <option value="stage">Stage</option>
          <option value="alternance">Alternance</option>
          <option value="interim">Intérim</option>
          <option value="cdd">CDD</option>
          <option value="cdi">CDI</option>
        </select>
      </div>

      <div class="frow">
        <div class="field">
          <label class="flabel">Début</label>
          <div style="display:flex;gap:8px">
            <select v-model="draft.debut_mois" class="input" style="flex:1">
              <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
            </select>
            <select v-model="draft.debut_annee" class="input" style="flex:1">
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
          </div>
        </div>
        <div class="field">
          <label class="flabel">Fin</label>
          <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">
            <template v-if="!draft.en_cours">
              <select v-model="draft.fin_mois" class="input" style="flex:1;min-width:80px">
                <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
              </select>
              <select v-model="draft.fin_annee" class="input" style="flex:1;min-width:80px">
                <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
              </select>
            </template>
            <label class="en-cours-label">
              <input type="checkbox" v-model="draft.en_cours" />
              En cours
            </label>
          </div>
        </div>
      </div>

      <div class="field" style="margin-bottom:0">
        <div class="desc-field-hd">
          <label class="flabel" style="margin-bottom:0">Description</label>
          <button
            type="button"
            class="bullet-btn"
            aria-label="Ajouter une puce"
            @click="descTextarea && insertBullet(descTextarea, draft.description, v => draft.description = v)"
          ><span aria-hidden="true">•</span> Puce</button>
        </div>
        <textarea
          ref="descTextarea"
          v-model="draft.description"
          class="input"
          style="height:100px"
          placeholder="Décris tes missions, technologies utilisées…"
        />
      </div>

      <div class="popup-foot">
        <button class="btn" type="button" @click="closeModal">Annuler</button>
        <button
          class="btn edit-save"
          type="button"
          :class="{ disabled: !draft.poste.trim() }"
          :disabled="!draft.poste.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
