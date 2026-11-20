<script setup lang="ts">
/**
 * Indicateur des trois étapes de la création d'une candidature (Offre,
 * Détails, Génération) : étapes faites cochées en vert, étape courante en
 * noir. Passe à la ligne plutôt que de déborder sur un écran étroit.
 * Propre à `NewApplicationView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import UiIcon from '@/components/UiIcon.vue'

const props = defineProps<{
  /** Index de l'étape courante (0 = Offre, 1 = Détails, 2 = Génération). */
  active: number
}>()

const { t } = useI18n()

const steps = computed(() =>
  (['offer', 'details', 'generation'] as const).map((key, i) => ({
    key,
    label: t(`new_application.stepper.${key}`),
    state: i < props.active ? 'done' : i === props.active ? 'on' : 'todo',
  })),
)
</script>

<template>
  <ol class="flex flex-wrap items-center gap-[10px] list-none m-0 p-0">
    <template v-for="(step, i) in steps" :key="step.key">
      <li
        class="flex items-center gap-2 text-[12.5px] whitespace-nowrap"
        :class="{
          'text-neutral-950 font-medium': step.state === 'on',
          'text-neutral-600': step.state === 'done',
          'text-neutral-400': step.state === 'todo',
        }"
        :aria-current="step.state === 'on' ? 'step' : undefined"
      >
        <span
          class="w-[22px] h-[22px] rounded-full border-[1.5px] flex items-center justify-center text-[11px] font-semibold"
          :class="{
            'bg-neutral-950 border-neutral-950 text-white': step.state === 'on',
            'bg-green-500 border-green-500 text-white': step.state === 'done',
            'border-neutral-300': step.state === 'todo',
          }"
        >
          <UiIcon v-if="step.state === 'done'" name="check" />
          <template v-else>{{ i + 1 }}</template>
        </span>
        <span>{{ step.label }}</span>
      </li>
      <span v-if="i < steps.length - 1" class="w-7 h-px bg-neutral-300" aria-hidden="true" />
    </template>
  </ol>
</template>
