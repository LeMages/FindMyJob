<script setup lang="ts">
/**
 * Barre latérale globale, repliable. Dépliée : entrée Candidatures et ses
 * sessions en haut, Profil et Paramètres en bas, puis « Réduire ». Repliée :
 * rail d'icônes avec infobulles, le survol de Candidatures ouvre un panneau
 * des sessions. Composant partagé, monté une seule fois par `App.vue`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import NavIcon from '@/components/NavIcon.vue'
import SidebarSessionList from '@/components/SidebarSessionList.vue'

// Importation des composables
import { useSidebar } from '@/composables/useSidebar'

// Importation des modules internes
import { state, navigate, openApplications } from '@/stores/app'

const { t } = useI18n()
const { collapsed, toggleSidebar } = useSidebar()

/** Zone de navigation de l'écran courant (entrée surlignée). */
const area = computed<'applications' | 'profile' | 'settings'>(() => {
  if (state.screen === 'profile') return 'profile'
  if (state.screen === 'settings') return 'settings'
  return 'applications'
})

const itemClass = (on: boolean) => on
  ? 'bg-neutral-950 text-white [&>svg]:text-white'
  : 'text-neutral-700 hover:bg-neutral-200 hover:text-neutral-950 [&>svg]:text-neutral-500'
const railClass = (on: boolean) => on
  ? 'bg-neutral-950 text-white'
  : 'text-neutral-600 hover:bg-neutral-200 hover:text-neutral-950'
const tipClass = 'hidden group-hover/rail:block absolute left-[calc(100%+10px)] top-1/2 -translate-y-1/2 z-[60] bg-neutral-950 text-white text-[12px] px-[9px] py-[5px] rounded-md whitespace-nowrap pointer-events-none'
</script>

<template>
  <!-- Repliée : rail d'icônes -->
  <aside v-if="collapsed" class="w-16 flex-none bg-neutral-100 border-r border-neutral-200 flex flex-col h-screen relative z-30">
    <div class="flex items-center justify-center pt-[18px] pb-[14px] flex-none">
      <div class="w-7 h-7 rounded-lg bg-neutral-950 text-white flex items-center justify-center font-bold text-[13px]">F</div>
    </div>

    <nav class="flex-1 flex flex-col items-center gap-[6px] pt-1">
      <div class="relative group/rail">
        <button class="w-10 h-10 rounded-10 border-0 bg-transparent flex items-center justify-center cursor-pointer" :class="railClass(area === 'applications')" :aria-label="t('sidebar.applications')" @click="openApplications">
          <NavIcon name="applications" />
        </button>
        <!-- Panneau des sessions ; le pseudo-élément comble l'écart pour que le survol ne se perde pas -->
        <div class="hidden group-hover/rail:block absolute left-[calc(100%+10px)] top-0 z-[60] w-[230px] bg-surface-3 border border-neutral-200 rounded-xl shadow-[0_12px_32px_rgba(0,0,0,.14)] p-2 before:content-[''] before:absolute before:-left-[14px] before:top-0 before:w-[14px] before:h-12">
          <div class="text-[11px] font-semibold text-neutral-500 px-2 pt-1 pb-[6px] font-mono tracking-[.04em]">{{ t('sidebar.applications') }}</div>
          <SidebarSessionList compact :in-applications="area === 'applications'" />
        </div>
      </div>
    </nav>

    <div class="flex-none border-t border-neutral-200 p-[10px] flex flex-col items-center gap-[6px]">
      <div class="relative group/rail">
        <button class="w-10 h-10 rounded-10 border-0 bg-transparent flex items-center justify-center cursor-pointer" :class="railClass(area === 'profile')" :aria-label="t('sidebar.profile')" @click="navigate('profile')">
          <NavIcon name="profile" />
        </button>
        <div :class="tipClass">{{ t('sidebar.profile') }}</div>
      </div>
      <div class="relative group/rail">
        <button class="w-10 h-10 rounded-10 border-0 bg-transparent flex items-center justify-center cursor-pointer" :class="railClass(area === 'settings')" :aria-label="t('sidebar.settings')" @click="navigate('settings')">
          <NavIcon name="settings" />
        </button>
        <div :class="tipClass">{{ t('sidebar.settings') }}</div>
      </div>
      <div class="h-px bg-neutral-200 my-[6px] mx-1 self-stretch" />
      <div class="relative group/rail">
        <button class="w-10 h-10 rounded-10 border-0 bg-transparent flex items-center justify-center cursor-pointer" :class="railClass(false)" :aria-label="t('sidebar.expand')" @click="toggleSidebar">
          <NavIcon name="expand" />
        </button>
        <div :class="tipClass">{{ t('sidebar.expand') }}</div>
      </div>
    </div>
  </aside>

  <!-- Dépliée -->
  <aside v-else class="w-[248px] flex-none bg-neutral-100 border-r border-neutral-200 flex flex-col h-screen">
    <div class="flex items-center gap-[10px] px-[18px] pt-[18px] pb-[14px] flex-none">
      <div class="w-7 h-7 rounded-lg bg-neutral-950 text-white flex items-center justify-center font-bold text-[13px]">F</div>
      <span class="font-semibold text-[15px] text-neutral-950">FindMyJob</span>
    </div>

    <nav class="flex-1 overflow-y-auto px-[10px] py-1">
      <div class="flex items-center gap-[11px] py-[9px] px-[10px] rounded-lg text-[13.5px] font-medium cursor-pointer select-none mb-0.5" :class="itemClass(area === 'applications')" @click="openApplications">
        <NavIcon name="applications" /><span>{{ t('sidebar.applications') }}</span>
      </div>
      <div class="mt-0.5 mb-[10px] ml-[19px] pl-[10px] border-l border-neutral-200">
        <SidebarSessionList :in-applications="area === 'applications'" />
      </div>
    </nav>

    <div class="flex-none border-t border-neutral-200 p-[10px]">
      <div class="flex items-center gap-[11px] py-[9px] px-[10px] rounded-lg text-[13.5px] font-medium cursor-pointer select-none mb-0.5" :class="itemClass(area === 'profile')" @click="navigate('profile')">
        <span
          class="w-[18px] h-[18px] rounded-full text-[8.5px] font-bold flex items-center justify-center flex-none"
          :class="area === 'profile' ? 'bg-white text-neutral-950' : 'bg-neutral-300 text-neutral-800'"
        >{{ state.userInitials || '?' }}</span>
        <span>{{ t('sidebar.profile') }}</span>
      </div>
      <div class="flex items-center gap-[11px] py-[9px] px-[10px] rounded-lg text-[13.5px] font-medium cursor-pointer select-none mb-0.5" :class="itemClass(area === 'settings')" @click="navigate('settings')">
        <NavIcon name="settings" /><span>{{ t('sidebar.settings') }}</span>
      </div>
      <div class="h-px bg-neutral-200 my-[6px] mx-1" />
      <button class="flex items-center gap-[10px] w-full border-0 bg-transparent py-2 px-[10px] rounded-lg text-[12.5px] text-neutral-500 cursor-pointer font-sans hover:bg-neutral-200 hover:text-neutral-950" @click="toggleSidebar">
        <NavIcon name="collapse" /><span>{{ t('sidebar.collapse') }}</span>
      </button>
    </div>
  </aside>
</template>
