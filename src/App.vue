<script setup lang="ts">
/**
 * Composant racine : détermine l'écran de démarrage (chargement, erreur,
 * onboarding ou application principale) et route entre les vues selon
 * `state.screen`.
 */

// Importation des modules Vue / framework
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import AppSidebar from '@/components/AppSidebar.vue'
import TrackingView from '@/views/tracking/TrackingView.vue'
import DetailView from '@/views/detail/DetailView.vue'
import NewApplicationView from '@/views/new_application/NewApplicationView.vue'
import ProfileView from '@/views/profile/ProfileView.vue'
import SettingsView from '@/views/settings/SettingsView.vue'
import DeleteSessionPopup from '@/components/DeleteSessionPopup.vue'
import CreateSessionPopup from '@/components/CreateSessionPopup.vue'
import AppToast from '@/components/AppToast.vue'
import OnboardingWizard from '@/views/onboarding/OnboardingWizard.vue'
import { state, setUserInfo, loadSessions } from '@/stores/app'
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { UserView } from '@/types/index'

const needsOnboarding = ref<boolean | null>(null)
const startupError = ref<string | null>(null)

/**
 * Synchronise le nom d'affichage et les initiales de l'utilisateur dans le store global.
 *
 * @param u - Utilisateur dont dériver le nom et les initiales.
 */
function applyUserInfo(u: UserView) {
  const name = [u.prenom, u.nom].filter(Boolean).join(' ')
  const initials = ((u.prenom?.[0] ?? '') + (u.nom?.[0] ?? '')).toUpperCase()
  setUserInfo(name, initials || '?', u.nom, u.prenom)
}

onMounted(async () => {
  try {
    const exists = await invoke<boolean>('user_exists')
    if (exists) {
      const user = await invoke<UserView>('get_user')
      applyUserInfo(user)
      await loadSessions()
    }
    needsOnboarding.value = !exists
  } catch (e: unknown) {
    startupError.value = extractErrorMessage(e, "Impossible de démarrer l'application.")
  }
})

/**
 * Termine l'onboarding : applique l'identité utilisateur, charge les
 * sessions puis bascule vers l'application principale.
 *
 * @param user - Utilisateur fraîchement créé.
 */
async function handleOnboardingCompleted(user: UserView) {
  applyUserInfo(user)
  await loadSessions()
  needsOnboarding.value = false
}
</script>

<template>
  <!-- Checking user -->
  <div v-if="needsOnboarding === null && !startupError" class="fixed inset-0 bg-neutral-50 flex items-center justify-center" />

  <!-- Startup error — fail fast, do not assume absence of user -->
  <div v-else-if="startupError" class="fixed inset-0 bg-neutral-50 flex items-center justify-center">
    <div class="bg-surface-1 border border-neutral-200 rounded-xl py-8 px-9 max-w-[400px] text-center">
      <div class="text-[16px] font-semibold text-neutral-950 mb-2">Impossible de démarrer</div>
      <div class="text-[13px] text-neutral-600 font-mono">{{ startupError }}</div>
    </div>
  </div>

  <!-- Onboarding wizard -->
  <OnboardingWizard
    v-else-if="needsOnboarding"
    @completed="handleOnboardingCompleted"
  />

  <!-- Main app -->
  <template v-else>
    <div class="flex h-screen overflow-hidden">
      <AppSidebar />
      <main class="flex-1 overflow-y-auto bg-neutral-50 min-w-0">
        <TrackingView v-if="state.screen === 'tracking'" />
        <DetailView v-else-if="state.screen === 'detail'" />
        <NewApplicationView v-else-if="state.screen === 'new'" />
        <ProfileView v-else-if="state.screen === 'profile'" />
        <SettingsView v-else-if="state.screen === 'settings'" />
      </main>
    </div>
    <DeleteSessionPopup />
    <CreateSessionPopup />
    <AppToast />
  </template>
</template>
