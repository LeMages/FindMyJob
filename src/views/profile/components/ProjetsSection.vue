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
  <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
    <div :class="editMode ? 'flex items-center justify-between mb-[14px]' : ''">
      <div class="text-[11px] font-semibold font-mono tracking-[.06em] text-neutral-500" :class="editMode ? '' : 'mb-[14px]'">PROJETS</div>
      <button
        v-if="editMode"
        class="h-[30px] px-3 text-[12px] rounded-lg border border-neutral-300 bg-white font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        type="button"
        @click="openAddModal"
      >
        + Ajouter
      </button>
    </div>
    <template v-if="allItems.length">
      <div
        v-for="(proj, i) in allItems"
        :key="proj.id"
        class="flex gap-[14px] py-[14px] border-b border-neutral-100"
        :class="[proj.isPending ? 'opacity-[0.72]' : '', i === allItems.length - 1 ? 'border-b-0' : '']"
      >
        <div class="w-[42px] h-[42px] rounded-lg flex-none bg-neutral-100 bg-[repeating-linear-gradient(45deg,var(--color-neutral-200)_0,var(--color-neutral-200)_1px,transparent_1px,transparent_7px)] border border-neutral-200" />
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <div class="text-[14px] font-semibold">{{ proj.titre }}</div>
            <span v-if="proj.pour_qui" class="inline-block font-mono text-[10.5px] border border-neutral-300 rounded-5 px-[7px] py-0.5 text-neutral-700">{{ proj.pour_qui }}</span>
            <span v-if="proj.isPending" class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50">
              {{ proj.pendingKind === 'edited' ? 'Modifié' : 'Non enregistré' }}
            </span>
          </div>
          <div v-if="proj.stack" class="font-mono text-[11px] text-neutral-700 mt-px">{{ proj.stack }}</div>
          <div class="font-mono text-[11.5px] text-neutral-400 mt-[3px]">{{ fmtYearRange(proj.date_debut, proj.date_fin) }}</div>
          <template v-if="proj.description">
            <ul v-if="proj.description.includes('•')" class="text-[13px] text-neutral-700 mt-[7px] leading-[1.6] pl-[1.15em] list-disc mb-0 [&>li+li]:mt-[5px]">
              <li v-for="(b, bi) in parseBulletDescription(proj.description)" :key="bi">{{ b }}</li>
            </ul>
            <div v-else class="text-[13px] text-neutral-700 mt-[7px] leading-[1.55]">{{ proj.description }}</div>
          </template>
          <button
            v-if="proj.url"
            class="text-[11px] text-blue-500 no-underline bg-transparent border-none p-0 cursor-pointer inline-block mt-0.5 break-all text-left hover:underline"
            type="button"
            @click="invoke('open_url', { url: proj.url })"
          >{{ proj.url }}</button>
        </div>
        <div v-if="editMode" class="flex flex-row gap-[5px] flex-none self-center ml-[6px]">
          <button
            class="w-[30px] h-[30px] rounded-7 border border-neutral-200 bg-neutral-50 flex items-center justify-center cursor-pointer flex-none text-neutral-600 hover:bg-neutral-100 hover:text-neutral-950 hover:border-neutral-300"
            type="button"
            title="Modifier"
            @click="openEditModal(proj)"
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
            @click="deleteItem(proj)"
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
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">Aucun projet renseigné</div>
  </div>

  <!-- Popup ajout/modification de projet -->
  <div v-if="modalOpen" class="fixed inset-0 bg-[rgba(15,15,10,.48)] flex items-center justify-center z-[100]">
    <div class="bg-white rounded-14 shadow-[0_20px_64px_rgba(0,0,0,.24)] w-[580px] max-h-[80vh] overflow-y-auto px-7 pt-[26px] pb-6">
      <div class="flex items-center justify-between mb-5">
        <div class="text-[17px] font-semibold">
          {{ editTarget.kind === 'new' ? 'Ajouter un projet' : 'Modifier un projet' }}
        </div>
        <button class="bg-transparent border-none text-[22px] text-neutral-400 cursor-pointer leading-none p-0 hover:text-neutral-950" type="button" @click="closeModal">×</button>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Titre *</label>
        <input v-model="draft.titre" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Mon super projet" />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Pour qui</label>
        <input v-model="draft.pour_qui" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Perso, École, Client..." />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Stack utilisé</label>
        <input v-model="draft.stack" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="Vue 3, Rust, SQLite..." />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Lien</label>
        <input v-model="draft.url" class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400" placeholder="https://github.com/..." />
      </div>

      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Année de début</label>
          <select
            v-model="draft.debut_annee"
            class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
          >
            <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
          </select>
        </div>
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Année de fin</label>
          <div class="flex gap-2 items-center">
            <select
              v-if="!draft.en_cours"
              v-model="draft.fin_annee"
              class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 appearance-none bg-no-repeat bg-[right_12px_center] pr-8 cursor-pointer flex-1 bg-[url('data:image/svg+xml,%3Csvg%20xmlns=%27http://www.w3.org/2000/svg%27%20width=%2712%27%20height=%278%27%20viewBox=%270%200%2012%208%27%3E%3Cpath%20d=%27M1%201l5%205%205-5%27%20stroke=%27%239b9b95%27%20stroke-width=%271.5%27%20fill=%27none%27%20stroke-linecap=%27round%27/%3E%3C/svg%3E')]"
            >
              <option v-for="y in yearRange" :key="y" :value="y">{{ y }}</option>
            </select>
            <label class="flex items-center gap-[6px] text-[13px] cursor-pointer whitespace-nowrap select-none" :class="draft.en_cours ? '' : 'flex-none'">
              <input type="checkbox" v-model="draft.en_cours" />
              En cours
            </label>
          </div>
        </div>
      </div>

      <div class="mb-[14px]">
        <div class="flex items-center justify-between mb-[5px]">
          <label class="text-[12px] text-neutral-600 mb-0 block">Description</label>
          <button
            type="button"
            class="inline-flex items-center gap-1 h-6 px-[9px] text-[12px] rounded-5 border border-neutral-300 bg-neutral-50 text-neutral-700 cursor-pointer leading-none transition-colors duration-[120ms] hover:bg-neutral-100 hover:border-neutral-300 hover:text-neutral-950"
            aria-label="Ajouter une puce"
            @click="descTextarea && insertBullet(descTextarea, draft.description, v => draft.description = v)"
          ><span aria-hidden="true">•</span> Puce</button>
        </div>
        <textarea
          ref="descTextarea"
          v-model="draft.description"
          class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-20"
          rows="3"
          placeholder="Ce projet consiste à..."
        ></textarea>
      </div>

      <div class="flex justify-end gap-[10px] mt-5 pt-4 border-t border-neutral-100">
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
          type="button"
          @click="closeModal"
        >Annuler</button>
        <button
          class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap border bg-green-500 text-white border-green-500 hover:bg-green-600 hover:border-green-600"
          :class="{ 'opacity-40 cursor-not-allowed pointer-events-none': !draft.titre.trim() }"
          type="button"
          :disabled="!draft.titre.trim()"
          @click="saveModal"
        >Valider</button>
      </div>
    </div>
  </div>
</template>
