<script setup lang="ts">
/**
 * Pastille d'un élément de liste du profil modifié ou ajouté mais pas encore
 * enregistré (avant « Valider »).
 */

// Importation des modules Vue / framework
import { useI18n } from 'vue-i18n'

defineProps<{
  kind: 'new' | 'edited'
}>()

const { t } = useI18n()
</script>

<template>
  <span class="inline-block font-mono text-[10.5px] rounded-5 px-[7px] py-0.5 border border-amber-300 text-amber-700 bg-amber-50 flex-none">
    {{ kind === 'edited' ? t('profile.pending.edited') : t('profile.pending.new') }}
  </span>
</template>
