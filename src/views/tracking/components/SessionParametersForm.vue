<script setup lang="ts">
/**
 * Formulaire de paramètres de la session courante (onglet Paramètres de la
 * page Candidatures). Titre, contexte, statut et zone cible sont reliés au
 * backend (`sessions`), édités au double-clic (même mécanique que la page
 * détail d'une candidature) ; le CV/LM restent mockés en dur en attendant
 * leur commande Tauri.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules internes
import { currentSession, updateSession } from '@/stores/app'
import type { SessionStatut } from '@/types'
import EditableField from '@/components/EditableField.vue'

const STATUS_OPTIONS: { value: SessionStatut; label: string }[] = [
  { value: 'active', label: 'Active' },
  { value: 'pause', label: 'En pause' },
  { value: 'termine', label: 'Terminée' },
]

/** Sauvegarde le titre de la session. */
async function saveNom(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { nom: v })
}

/** Sauvegarde le contexte de la session. */
async function saveContexte(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { contexte: v })
}

/** Sauvegarde le statut de la session. */
async function saveStatut(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { statut: v as SessionStatut })
}

/** Sauvegarde la zone cible de la session. */
async function saveZone(v: string): Promise<string | null | undefined> {
  const session = currentSession.value
  if (!session) return
  return updateSession(session.id, { zone: v })
}

const cvFilename = ref('CV_Loic_Cupif_2026.pdf')
const lettreFilename = ref('LM_Template_Stage.pdf')
</script>

<template>
  <div class="px-[30px] pt-[18px] pb-[30px]">
    <!-- Informations générales -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">INFORMATIONS GÉNÉRALES</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Titre de la session</label>
        <EditableField
          :model-value="currentSession?.nom ?? null"
          :editable="true"
          placeholder="Titre de la session"
          :on-save="saveNom"
        />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Contexte</label>
        <EditableField
          type="textarea"
          :model-value="currentSession?.contexte ?? null"
          :editable="true"
          placeholder="Contexte de la session"
          :on-save="saveContexte"
        />
      </div>

      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Statut de la session</label>
        <EditableField
          type="select"
          :options="STATUS_OPTIONS"
          :model-value="currentSession?.statut ?? null"
          :editable="true"
          :on-save="saveStatut"
        />
      </div>
    </div>

    <!-- Zone géographique -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">ZONE GÉOGRAPHIQUE</div>
      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Zone cible</label>
        <EditableField
          :model-value="currentSession?.zone ?? null"
          :editable="true"
          placeholder="Ex. Genève / Lausanne — Romandie"
          :on-save="saveZone"
        />
      </div>
    </div>

    <!-- Documents -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DOCUMENTS</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[7px] block">CV</label>
        <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px]">
          <div class="w-7 h-9 rounded-md flex-none flex items-center justify-center bg-neutral-100 border border-neutral-200 text-neutral-500">
            <svg width="14" height="17" viewBox="0 0 14 17" fill="none">
              <path d="M1.5 1.5h7l4 4v9.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1v-12.5a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
              <path d="M8.5 1.5v4h4" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[13.5px] font-semibold leading-[1.25]">{{ cvFilename }}</div>
            <div class="font-mono text-[11px] text-neutral-500">Joint</div>
          </div>
        </div>
      </div>

      <div>
        <label class="text-[12px] text-neutral-600 mb-[7px] block">Lettre de motivation</label>
        <div class="flex items-center gap-[10px] border border-neutral-200 rounded-lg p-[10px]">
          <div class="w-7 h-9 rounded-md flex-none flex items-center justify-center bg-neutral-100 border border-neutral-200 text-neutral-500">
            <svg width="14" height="17" viewBox="0 0 14 17" fill="none">
              <path d="M1.5 1.5h7l4 4v9.5a1 1 0 0 1-1 1h-10a1 1 0 0 1-1-1v-12.5a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
              <path d="M8.5 1.5v4h4" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[13.5px] font-semibold leading-[1.25]">{{ lettreFilename }}</div>
            <div class="font-mono text-[11px] text-neutral-500">Jointe</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
