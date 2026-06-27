<script setup lang="ts">
import AppSidebar from '@/components/AppSidebar.vue'
import TrackingView from '@/components/views/TrackingView.vue'
import DetailView from '@/components/views/DetailView.vue'
import NewApplicationView from '@/components/views/NewApplicationView.vue'
import ProfileView from '@/components/views/ProfileView.vue'
import SettingsView from '@/components/views/SettingsView.vue'
import DeleteSessionPopup from '@/components/popups/DeleteSessionPopup.vue'
import { state } from '@/stores/app'
</script>

<template>
  <div class="app">
    <AppSidebar />
    <main class="main">
      <TrackingView v-if="state.screen === 'tracking'" />
      <DetailView v-else-if="state.screen === 'detail'" />
      <NewApplicationView v-else-if="state.screen === 'new'" />
      <ProfileView v-else-if="state.screen === 'profile'" />
      <SettingsView v-else-if="state.screen === 'settings'" />
    </main>
  </div>
  <DeleteSessionPopup />
</template>
