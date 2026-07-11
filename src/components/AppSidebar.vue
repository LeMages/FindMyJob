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
  <aside class="side">
    <!-- Brand -->
    <div class="brand">
      <div class="brandmark">F</div>
      FindMyJob
    </div>

    <!-- Search -->
    <div class="search">
      <svg width="15" height="15" viewBox="0 0 15 15" fill="none">
        <circle cx="6.5" cy="6.5" r="4.5" stroke="#9b9b95" stroke-width="1.4" />
        <line x1="9.8" y1="9.8" x2="13.5" y2="13.5" stroke="#9b9b95" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      Rechercher une candidature
    </div>

    <!-- Sessions -->
    <div class="slabel">MES SESSIONS</div>
    <nav class="snav">
      <div
        v-for="s in allSessions"
        :key="s.id"
        class="sitem"
        :class="{ on: s.id === state.sessionId }"
        @click="selectSession(s.id)"
      >
        <div class="sdot" :class="{ grn: s.hasAccepted }" />
        <span class="f1">{{ s.nom }}</span>
        <span class="scount">{{ s.count }}</span>
        <button
          class="sdel"
          @click.stop="openDeleteSessionPopup(s.id, s.nom)"
          title="Supprimer"
        >×</button>
      </div>

      <div class="newses" @click="addSession">+ Nouvelle session</div>
    </nav>

    <!-- User bar -->
    <div class="userbar">
      <button class="userbtn" @click="navigate('profile')">
        <div class="avatar">{{ state.userInitials || '…' }}</div>
        <div>
          <div class="uname">{{ state.userDisplayName || '…' }}</div>
          <div class="usub">Voir le profil</div>
        </div>
      </button>
      <button class="iconbtn" title="Notifications">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <path d="M9 2a5 5 0 0 0-5 5v3l-1.5 2.5h13L14 10V7a5 5 0 0 0-5-5Z" stroke="#56564f" stroke-width="1.4" stroke-linejoin="round" />
          <path d="M7.5 14.5a1.5 1.5 0 0 0 3 0" stroke="#56564f" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
      <button class="iconbtn" @click="navigate('settings')" title="Paramètres">
        <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
          <line x1="3" y1="5" x2="15" y2="5" stroke="#56564f" stroke-width="1.4" />
          <line x1="3" y1="9" x2="15" y2="9" stroke="#56564f" stroke-width="1.4" />
          <line x1="3" y1="13" x2="15" y2="13" stroke="#56564f" stroke-width="1.4" />
          <circle cx="6" cy="5" r="2.1" fill="#fff" stroke="#56564f" stroke-width="1.4" />
          <circle cx="11.5" cy="9" r="2.1" fill="#fff" stroke="#56564f" stroke-width="1.4" />
          <circle cx="7" cy="13" r="2.1" fill="#fff" stroke="#56564f" stroke-width="1.4" />
        </svg>
      </button>
    </div>
  </aside>
</template>
