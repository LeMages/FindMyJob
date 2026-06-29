<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import AppSidebar from '@/components/AppSidebar.vue'
import TrackingView from '@/components/views/TrackingView.vue'
import DetailView from '@/components/views/DetailView.vue'
import NewApplicationView from '@/components/views/NewApplicationView.vue'
import ProfileView from '@/components/views/ProfileView.vue'
import SettingsView from '@/components/views/SettingsView.vue'
import DeleteSessionPopup from '@/components/popups/DeleteSessionPopup.vue'
import OnboardingWizard from '@/components/OnboardingWizard.vue'
import { state, setUserInfo } from '@/stores/app'
import type { UserView } from '@/types/index'

const needsOnboarding = ref<boolean | null>(null)
const startupError = ref<string | null>(null)

function applyUserInfo(u: UserView) {
  const name = [u.prenom, u.nom].filter(Boolean).join(' ')
  const initials = ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase()
  setUserInfo(name, initials || '?')
}

onMounted(async () => {
  try {
    const exists = await invoke<boolean>('user_exists')
    if (exists) {
      const user = await invoke<UserView>('get_user')
      applyUserInfo(user)
    }
    needsOnboarding.value = !exists
  } catch (e: unknown) {
    startupError.value =
      e && typeof e === 'object' && 'message' in e
        ? (e as { message: string }).message
        : typeof e === 'string'
          ? e
          : "Impossible de démarrer l'application."
  }
})

function handleOnboardingCompleted(user: UserView) {
  applyUserInfo(user)
  needsOnboarding.value = false
}
</script>

<template>
  <!-- Checking user -->
  <div v-if="needsOnboarding === null && !startupError" class="startup-screen" />

  <!-- Startup error — fail fast, do not assume absence of user -->
  <div v-else-if="startupError" class="startup-screen startup-error">
    <div class="startup-error-box">
      <div class="startup-error-title">Impossible de démarrer</div>
      <div class="startup-error-msg">{{ startupError }}</div>
    </div>
  </div>

  <!-- Onboarding wizard -->
  <OnboardingWizard
    v-else-if="needsOnboarding"
    @completed="handleOnboardingCompleted"
  />

  <!-- Main app -->
  <template v-else>
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
</template>

<style scoped>
.startup-screen {
  position: fixed;
  inset: 0;
  background: #f8f8f6;
  display: flex;
  align-items: center;
  justify-content: center;
}

.startup-error {
  display: flex;
  align-items: center;
  justify-content: center;
}

.startup-error-box {
  background: #fff;
  border: 1px solid #e6e6e1;
  border-radius: 12px;
  padding: 32px 36px;
  max-width: 400px;
  text-align: center;
}

.startup-error-title {
  font-size: 16px;
  font-weight: 600;
  color: #1d1d1b;
  margin-bottom: 8px;
}

.startup-error-msg {
  font-size: 13px;
  color: #76766f;
  font-family: 'IBM Plex Mono', monospace;
}
</style>
