<script setup lang="ts">
/**
 * Vue « Paramètres » : configuration de l'IA, des relances, de l'apparence,
 * de la signature et du stockage. Interface de maquette — les champs ne sont
 * pas encore persistés (aucune commande Tauri branchée).
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

const showApiKey = ref(false)
const iaMode = ref<'cloud' | 'local'>('cloud')
const theme = ref<'clair' | 'sombre' | 'systeme'>('clair')
const relanceDelay = ref(7)
</script>

<template>
  <div class="px-9 pt-[26px] pb-11">
    <h1 class="text-2xl font-semibold leading-tight">Paramètres</h1>
    <div class="text-[13px] text-neutral-500 mt-[5px] font-mono mb-[22px]">Configuration de l'application FindMyJob.</div>

    <!-- IA -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">INTELLIGENCE ARTIFICIELLE</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Clé API Ollama Cloud</label>
        <div class="flex gap-2">
          <input
            class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400 flex-1 font-mono"
            :type="showApiKey ? 'text' : 'password'"
            value="sk-ollama-xxxxxxxxxxxxxxxxxxx"
          />
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
            @click="showApiKey = !showApiKey"
          >
            {{ showApiKey ? 'Masquer' : 'Afficher' }}
          </button>
        </div>
        <div class="font-mono text-[11px] text-neutral-500 mt-[6px]">Stockée dans le trousseau de l'OS — jamais en base de données.</div>
      </div>

      <div class="flex gap-[14px]">
        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Modèle</label>
          <div class="h-[38px] border border-neutral-300 rounded-lg px-3 flex items-center justify-between text-[13.5px] bg-neutral-50 text-neutral-950 cursor-pointer">
            <span>llama3.2</span>
            <span class="font-mono text-[11px] text-neutral-500">▾</span>
          </div>
        </div>

        <div class="flex-1 mb-[14px]">
          <label class="text-[12px] text-neutral-600 mb-[5px] block">Mode IA</label>
          <div class="flex gap-2">
            <div
              class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] font-semibold cursor-pointer"
              :class="iaMode === 'cloud' ? 'border-2 border-neutral-950 bg-neutral-950 text-white' : 'border border-neutral-200 text-neutral-300'"
              @click="iaMode = 'cloud'"
            >
              Cloud
            </div>
            <div class="flex-1 h-[38px] border border-neutral-200 rounded-lg flex items-center justify-center text-[13px] text-neutral-300 cursor-not-allowed gap-[6px]">
              Local
              <span class="text-[10px] font-mono bg-neutral-100 px-[6px] py-0.5 rounded">bientôt</span>
            </div>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-between gap-5 pt-2">
        <div>
          <div class="text-[14px] font-medium">Tester la connexion</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">Vérifie que la clé et le modèle sélectionné répondent correctement.</div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        >Tester →</button>
      </div>
    </div>

    <!-- Relances -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">RELANCES</div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Délai avant relance suggérée</label>
        <div class="flex items-center gap-3">
          <input
            v-model="relanceDelay"
            class="w-[90px] h-[38px] border border-neutral-300 rounded-lg px-3 text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block focus:outline-none focus:border-neutral-400"
            type="number"
          />
          <span class="text-[13px] text-neutral-500 font-mono m-0">jours sans réponse avant d'afficher une suggestion de relance.</span>
        </div>
      </div>
    </div>

    <!-- Apparence -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">APPARENCE</div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Thème</label>
        <div class="flex gap-2">
          <div
            v-for="t in (['clair', 'sombre', 'systeme'] as const)"
            :key="t"
            class="flex-1 h-[38px] rounded-lg flex items-center justify-center text-[13px] cursor-pointer capitalize"
            :class="theme === t ? 'border-2 border-neutral-950 font-semibold' : 'border border-neutral-300'"
            @click="theme = t"
          >
            {{ t.charAt(0).toUpperCase() + t.slice(1) }}
          </div>
        </div>
      </div>
    </div>

    <!-- Signature -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">SIGNATURE DE CANDIDATURE</div>
      <div class="mb-0">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Texte de clôture des mails</label>
        <textarea class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-neutral-50 text-neutral-950 block resize-none leading-[1.5] h-20">Cordialement, Alex Martin</textarea>
      </div>
    </div>

    <!-- Données & stockage -->
    <div class="bg-white border border-neutral-200 rounded-xl px-[22px] py-5 mb-[18px]">
      <div class="text-[11px] font-semibold mb-[14px] font-mono tracking-[.06em] text-neutral-500">DONNÉES &amp; STOCKAGE</div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Base de données</label>
        <div class="flex gap-2">
          <input
            class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[12px] font-mono bg-neutral-50 text-neutral-500 block focus:outline-none focus:border-neutral-400 flex-1"
            value="~/Library/Application Support/FindMyJob/db.sqlite"
            readonly
          />
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
          >Ouvrir le dossier</button>
        </div>
      </div>

      <div class="mb-[14px]">
        <label class="text-[12px] text-neutral-600 mb-[5px] block">Dossier documents</label>
        <div class="flex gap-2">
          <input
            class="h-[38px] border border-neutral-300 rounded-lg px-3 text-[12px] font-mono bg-neutral-50 text-neutral-500 block focus:outline-none focus:border-neutral-400 flex-1"
            value="~/Library/Application Support/FindMyJob/documents/"
            readonly
          />
          <button
            class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
          >Ouvrir le dossier</button>
        </div>
      </div>

      <div class="flex items-center justify-between gap-5 py-[10px] border-b border-neutral-100">
        <div>
          <div class="text-[14px] font-medium">Sauvegarde / Export</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">Exporter toutes les données en JSON pour archivage.</div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-900 inline-flex items-center gap-[7px] cursor-pointer font-sans whitespace-nowrap hover:bg-neutral-100"
        >Exporter</button>
      </div>

      <div class="flex items-center justify-between gap-5 py-[10px]">
        <div>
          <div class="text-[14px] font-medium">Réinitialiser l'application</div>
          <div class="text-[12.5px] text-neutral-500 mt-[3px] max-w-[520px] leading-[1.45]">Supprimer toutes les données locales. Action irréversible.</div>
        </div>
        <button
          class="h-9 px-4 rounded-lg border border-neutral-300 bg-white text-[13px] font-medium text-neutral-500 inline-flex items-center cursor-pointer font-sans hover:bg-neutral-100 hover:text-neutral-800"
        >Réinitialiser</button>
      </div>
    </div>
  </div>
</template>
