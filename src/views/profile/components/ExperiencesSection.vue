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
  <div class="bg-surface-1 border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div :class="editMode ? 'flex items-center justify-between mb-[14px]' : ''">
      <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500" :class="editMode ? '' : 'mb-[14px]'">EXPÉRIENCES</div>
      <button
        v-if="editMode"
        class="h-[30px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-surface-2 font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
        type="button"
        @click="openAddModal"
      >
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(exp, i) in allItems"
        :key="exp.id"
        class="flex gap-[14px] py-[14px] border-b border-neutral-100"
        :class="[exp.isPending ? 'opacity-[0.72]' : '', i === allItems.length - 1 ? 'border-b-0' : '']"
      >
        <div class="w-[42px] h-[42px] rounded-lg flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <div class="text-[14px] font-semibold">{{ exp.poste }}</div>
            <span v-if="exp.type_contrat" class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ typeLabel(exp.type_contrat) }}</span>
            <span v-if="exp.isPending" class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50">
              {{ exp.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div class="text-[13px] text-neutral-500 mt-px">
            {{ [exp.entreprise, exp.lieu].filter(Boolean).join(' — ') || '—' }}
          </div>
          <div class="font-mono text-[11.5px] text-neutral-400 mt-[3px]">{{ fmtRange(exp.date_debut, exp.date_fin) }}</div>
          <template v-if="exp.description">
            <ul v-if="exp.description.includes('•')" class="text-[13px] text-neutral-700 mt-[7px] leading-[1.6] pl-[1.15em] list-disc mb-0 [&>li+li]:mt-[5px]">
              <li v-for="(b, bi) in parseBulletDescription(exp.description)" :key="bi">{{ b }}</li>
            </ul>
            <div v-else class="text-[13px] text-neutral-700 mt-[7px] leading-[1.55]">{{ exp.description }}</div>
          </template>
        </div>
        <!-- Boutons édition (visibles uniquement en mode édition) -->
        <div v-if="editMode" class="flex flex-row gap-[5px] flex-none self-center ml-[6px]">
          <button
            class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-surface-3 hover:text-neutral-950 hover:border-neutral-300"
            type="button"
            title="Modifier"
            @click="openEditModal(exp)"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button
            class="w-[30px] h-[30px] rounded-7 flex items-center justify-center cursor-pointer flex-none text-red-600 border border-red-200 bg-red-50 hover:bg-red-100 hover:border-red-300"
            type="button"
            title="Supprimer"
            @click="deleteItem(exp)"
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
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucune expérience renseignée</div>
  </div>

  <!-- Popup ajout/modification d'expérience -->
  <div v-if="modalOpen" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]">
    <div class="bg-surface-3 rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">
          {{ editTarget.kind === 'new' ? 'Ajouter une expérience' : 'Modifier une expérience' }}
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="closeModal">×</button>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Intitulé du poste *</label>
        <input v-model="draft.poste" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Développeur Full-Stack" />
      </div>

      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Entreprise</label>
          <input v-model="draft.entreprise" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Nom de l'entreprise" />
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Lieu</label>
          <input v-model="draft.lieu" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Paris, France" />
        </div>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Type de contrat</label>
        <select
          v-model="draft.type_contrat"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
        >
          <option value="">-- Sélectionner --</option>
          <option value="stage">Stage</option>
          <option value="alternance">Alternance</option>
          <option value="interim">Intérim</option>
          <option value="cdd">CDD</option>
          <option value="cdi">CDI</option>
        </select>
      </div>

      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Début</label>
          <div class="flex gap-2">
            <select v-model="draft.debut_mois" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1">
              <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
            </select>
            <select v-model="draft.debut_annee" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1">
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
          </div>
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Fin</label>
          <div class="flex gap-2 items-center flex-wrap">
            <template v-if="!draft.en_cours">
              <select v-model="draft.fin_mois" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-[80px]">
                <option v-for="(m, idx) in MONTHS" :key="idx" :value="idx + 1">{{ m }}</option>
              </select>
              <select v-model="draft.fin_annee" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-surface-2 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 min-w-[80px]">
                <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
              </select>
            </template>
            <label class="flex items-center gap-[6px] text-[13px] cursor-pointer whitespace-nowrap select-none">
              <input type="checkbox" v-model="draft.en_cours" />
              En cours
            </label>
          </div>
        </div>
      </div>

      <div class="mb-0">
        <div class="flex items-center justify-between mb-[5px]">
          <label class="text-[12px] text-neutral-600 mb-0 block">Description</label>
          <button
            type="button"
            class="inline-flex items-center gap-1 h-6 px-[9px] text-[12px] rounded-5 border border-neutral-300 bg-surface-2 text-neutral-700 cursor-pointer leading-none transition-colors duration-[120ms] hover:bg-surface-3 hover:border-neutral-300 hover:text-neutral-950"
            aria-label="Ajouter une puce"
            @click="descTextarea && insertBullet(descTextarea, draft.description, v => draft.description = v)"
          ><span aria-hidden="true">•</span> Puce</button>
        </div>
        <textarea
          ref="descTextarea"
          v-model="draft.description"
          class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[100px]"
          placeholder="Décris tes missions, technologies utilisées…"
        />
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-surface-2 text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-surface-3"
          type="button"
          @click="closeModal"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.poste.trim() }"
          type="button"
          :disabled="!draft.poste.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
