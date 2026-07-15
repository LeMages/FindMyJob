<script setup lang="ts">
/**
 * Formulaire de paramètres de la session courante (onglet Paramètres de la
 * page Candidatures). Purement visuel : données mockées en dur, aucune
 * commande Tauri, aucune persistance.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

type SessionStatus = 'active' | 'pause' | 'terminee'

const titre = ref('Stage M2 — Data/ML Engineering')
const contexte = ref(
  "Stage obligatoire de 6 mois pour valider mon diplôme d'ingénieur à EFREI Paris. Recherche dans la région Romandie (Genève/Lausanne).",
)
const statut = ref<SessionStatus>('active')
const zoneCible = ref('Genève / Lausanne — Romandie')

const STATUS_OPTIONS: { value: SessionStatus; label: string }[] = [
  { value: 'active', label: 'Active' },
  { value: 'pause', label: 'En pause' },
  { value: 'terminee', label: 'Terminée' },
]

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
        <input
          v-model="titre"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400"
          type="text"
        />
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Contexte</label>
        <textarea
          v-model="contexte"
          class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-24"
        />
      </div>

      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Statut de la session</label>
        <div class="flex gap-2">
          <div
            v-for="opt in STATUS_OPTIONS"
            :key="opt.value"
            class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] font-semibold cursor-pointer"
            :class="statut === opt.value ? 'border-2 border-neutral-950 bg-neutral-950 text-white' : 'border border-neutral-300 text-neutral-500'"
            @click="statut = opt.value"
          >
            {{ opt.label }}
          </div>
        </div>
      </div>
    </div>

    <!-- Zone géographique -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">ZONE GÉOGRAPHIQUE</div>
      <div>
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Zone cible</label>
        <input
          v-model="zoneCible"
          class="w-full h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400"
          type="text"
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

    <!-- Enregistrer -->
    <button
      class="h-9 px-4 rounded-lg text-[13px] font-medium inline-flex items-center gap-[7px] font-sans whitespace-nowrap border border-neutral-950 bg-neutral-950 text-white opacity-40 cursor-not-allowed"
      disabled
    >
      Enregistrer
    </button>
  </div>
</template>
