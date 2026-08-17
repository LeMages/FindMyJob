<script setup lang="ts">
/**
 * Barre latérale globale : recherche, liste des sessions et accès au profil
 * et aux paramètres. Composant partagé, monté une seule fois par `App.vue`.
 */

// Importation des modules internes
import {
  state,
  allSessions,
  selectSession,
  addSession,
  openDeleteSessionPopup,
  navigate,
} from '@/stores/app'
</script>

<template>
  <aside class="w-[264px] flex-none bg-neutral-100 border-r border-neutral-200 flex flex-col h-screen overflow-hidden">
    <!-- Brand -->
    <div class="flex items-center gap-[9px] px-[18px] pt-[18px] pb-3 font-semibold text-base flex-none">
      <div class="w-[26px] h-[26px] rounded-7 bg-neutral-950 text-white flex items-center justify-center font-bold text-[13px]">F</div>
      FindMyJob
    </div>

    <!-- Search -->
    <div class="mx-[14px] mb-3 h-[34px] bg-surface-2 border border-neutral-200 rounded-lg flex items-center gap-2 px-[10px] text-[13px] text-neutral-400 flex-none">
      <svg width="15" height="15" viewBox="0 0 15 15" fill="none">
        <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" stroke-width="1.4" />
        <line x1="9.8" y1="9.8" x2="13.5" y2="13.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      Rechercher une candidature
    </div>

    <!-- Sessions -->
    <div class="px-[18px] pt-[10px] pb-[6px] font-mono text-[10.5px] tracking-[.07em] text-neutral-400 flex-none">MES SESSIONS</div>
    <nav class="flex-1 overflow-y-auto px-[10px]">
      <div
        v-for="s in allSessions"
        :key="s.id"
        class="group flex items-center gap-[10px] py-[9px] px-[10px] rounded-lg cursor-pointer text-[13.5px] mb-0.5 select-none"
        :class="s.id === state.sessionId ? 'bg-neutral-950 text-white' : 'text-neutral-800 hover:bg-neutral-200'"
        @click="selectSession(s.id)"
      >
        <div class="w-[7px] h-[7px] rounded-sm flex-none" :class="s.hasAccepted ? 'bg-green-400' : 'bg-neutral-300'" />
        <span class="flex-1 min-w-0">{{ s.nom }}</span>
        <span
          class="ml-auto font-mono text-[11px] rounded-md px-[7px] py-px flex-none"
          :class="s.id === state.sessionId ? 'bg-white/[0.12] text-white/70' : 'bg-neutral-200 text-neutral-600'"
        >{{ s.count }}</span>
        <button
          class="bg-transparent border-none cursor-pointer px-[6px] py-0.5 text-[15px] leading-none rounded-md flex-none opacity-0 transition-opacity duration-150 group-hover:opacity-100"
          :class="s.id === state.sessionId ? 'text-white/40 hover:bg-white/10 hover:text-white' : 'text-neutral-300 hover:text-neutral-800 hover:bg-neutral-100'"
          @click.stop="openDeleteSessionPopup(s.id, s.nom)"
          title="Supprimer"
        >×</button>
      </div>

      <div
        class="mx-1 mt-2 mb-3 py-[9px] px-[10px] border border-dashed border-neutral-300 rounded-lg text-[13px] text-neutral-600 cursor-pointer text-center select-none hover:bg-neutral-200 hover:text-neutral-800"
        @click="addSession"
      >+ Nouvelle session</div>
    </nav>

    <!-- User bar -->
    <div class="flex-none border-t border-neutral-200 p-3 flex items-center gap-2">
      <button class="flex-1 flex items-center gap-[10px] bg-transparent border-none cursor-pointer p-[6px] rounded-lg text-left font-sans hover:bg-neutral-200" @click="navigate('profile')">
        <div class="w-[34px] h-[34px] rounded-full flex-none flex items-center justify-center font-semibold text-[13px] bg-neutral-200 text-neutral-700">{{ state.userInitials || '…' }}</div>
        <div>
          <div class="text-[13px] font-semibold leading-tight">{{ state.userDisplayName || '…' }}</div>
          <div class="text-[11px] text-neutral-500">Voir le profil</div>
        </div>
      </button>
      <button class="w-[38px] h-[38px] flex-none rounded-lg border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer hover:bg-surface-3" title="Notifications">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" class="text-neutral-700">
          <path d="M9 2a5 5 0 0 0-5 5v3l-1.5 2.5h13L14 10V7a5 5 0 0 0-5-5Z" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round" />
          <path d="M7.5 14.5a1.5 1.5 0 0 0 3 0" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
      <button class="w-[38px] h-[38px] flex-none rounded-lg border border-neutral-200 bg-surface-2 flex items-center justify-center cursor-pointer hover:bg-surface-3" @click="navigate('settings')" title="Paramètres">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none" class="text-neutral-700">
          <line x1="3" y1="5" x2="15" y2="5" stroke="currentColor" stroke-width="1.4" />
          <line x1="3" y1="9" x2="15" y2="9" stroke="currentColor" stroke-width="1.4" />
          <line x1="3" y1="13" x2="15" y2="13" stroke="currentColor" stroke-width="1.4" />
          <circle cx="6" cy="5" r="2.1" fill="white" stroke="currentColor" stroke-width="1.4" />
          <circle cx="11.5" cy="9" r="2.1" fill="white" stroke="currentColor" stroke-width="1.4" />
          <circle cx="7" cy="13" r="2.1" fill="white" stroke="currentColor" stroke-width="1.4" />
        </svg>
      </button>
    </div>
  </aside>
</template>
