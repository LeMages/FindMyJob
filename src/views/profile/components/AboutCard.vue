<script setup lang="ts">
/**
 * Carte « À propos » du profil : description libre, éditable dans sa carte.
 * Le brouillon est tenu en local jusqu'à l'appel de `save()` par le parent.
 */

// Importation des modules Vue / framework
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

// Importation des composants
import ProfileCard from './ProfileCard.vue'

// Importation des types
import type { UserView } from '@/types/index'
import type { ProfileCardMode } from './profile_sections'

const props = defineProps<{
  user: UserView
  mode: ProfileCardMode
  saving: boolean
  error: string | null
}>()

const emit = defineEmits<{
  edit: []
  cancel: []
  save: []
}>()

const { t } = useI18n()

const draft = ref(props.user.description ?? '')

/** Repart de la description enregistrée (Annuler, ou entrée en édition). */
function reset() {
  draft.value = props.user.description ?? ''
}

watch(() => props.mode, (mode) => {
  if (mode === 'editing') reset()
})

const hasChanges = computed(() => draft.value !== (props.user.description ?? ''))

/**
 * Enregistre la description si elle a changé.
 *
 * @returns Utilisateur à jour (à assigner par le parent).
 */
async function save(): Promise<UserView> {
  if (!hasChanges.value) return props.user
  return invoke<UserView>('update_description', { description: draft.value.trim() || null })
}

defineExpose({ hasChanges, reset, save })
</script>

<template>
  <ProfileCard
    :title="t('profile.about.section')"
    :mode="mode"
    :dirty="hasChanges"
    :saving="saving"
    :error="error"
    @edit="emit('edit')"
    @cancel="emit('cancel')"
    @save="emit('save')"
  >
    <textarea
      v-if="mode === 'editing'"
      v-model="draft"
      class="w-full border border-neutral-300 rounded-lg px-3 py-[10px] text-[13.5px] font-sans bg-surface-2 text-neutral-950 block resize-none leading-[1.5] h-[140px] mt-3"
      :placeholder="t('profile.placeholders.about')"
    />
    <div v-else-if="user.description" class="text-[13.5px] text-neutral-700 mt-[10px] leading-[1.6]">{{ user.description }}</div>
    <div v-else class="text-[12px] text-neutral-400 text-center py-6 font-mono">{{ t('common.not_specified') }}</div>
  </ProfileCard>
</template>
