<script setup lang="ts">
/**
 * Popup « Postuler » de la fiche candidature : trois façons de postuler —
 * par mail (rédigé par l'IA, adressé au premier contact qui a un e-mail),
 * par message sur la plateforme (rédigé par l'IA), ou sans message. Pendant
 * la rédaction, la carte choisie affiche un chargement et les autres
 * s'estompent. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import AppModal from '@/components/AppModal.vue'
import OptionCard from '@/components/OptionCard.vue'

// Importation des modules internes
import {
  state,
  closeDetailPopup,
  applyWithoutMessage,
  applyByMail,
  applyByMessage,
} from '@/stores/app'

type ApplyVia = 'mail' | 'message' | 'sans_message'

const { t } = useI18n()

/** Voie de candidature en cours de traitement (désactive les trois options tant qu'elle est active). */
const applyingVia = ref<ApplyVia | null>(null)

const subtitle = computed(() => {
  const c = state.detailCandidature
  if (!c) return ''
  return c.entreprise ? `${c.poste} · ${c.entreprise}` : c.poste
})

/** Premier contact de la candidature qui a un e-mail : destinataire du mail rédigé. */
const mailRecipient = computed(() => state.detailContacts.find((c) => !!c.email) ?? null)

const mailDescription = computed(() =>
  mailRecipient.value
    ? t('detail.apply.mail_description_to', { name: mailRecipient.value.nom })
    : t('detail.apply.mail_description'),
)

/**
 * Déclenche une des trois façons de postuler pour la candidature ouverte,
 * en bloquant les autres options le temps de l'appel (rédaction IA incluse
 * pour mail/message) et en affichant l'erreur éventuelle.
 *
 * @param via - Voie choisie, utilisée pour l'état de chargement affiché.
 * @param action - Fonction du store à invoquer.
 */
async function confirmApply(via: ApplyVia, action: () => Promise<string | null>) {
  if (applyingVia.value) return
  applyingVia.value = via
  const error = await action()
  applyingVia.value = null
  if (error) state.detailError = error
}
</script>

<template>
  <AppModal
    v-if="state.detailPopup === 'apply'"
    :title="t('detail.apply.title')"
    :subtitle="subtitle"
    :width="620"
    divided
    backdrop-close
    @close="closeDetailPopup"
  >
    <div class="flex flex-col gap-[10px]">
      <OptionCard
        icon="mail"
        :title="t('detail.apply.mail_title')"
        :description="applyingVia === 'mail' ? t('detail.apply.mail_writing') : mailDescription"
        :badge="applyingVia ? null : t('detail.apply.mail_badge')"
        :primary="!applyingVia"
        :busy="applyingVia === 'mail'"
        :dimmed="applyingVia !== null && applyingVia !== 'mail'"
        :disabled="applyingVia !== null"
        @select="confirmApply('mail', applyByMail)"
      />
      <OptionCard
        icon="message"
        :title="t('detail.apply.message_title')"
        :description="applyingVia === 'message' ? t('detail.apply.message_writing') : t('detail.apply.message_description')"
        :busy="applyingVia === 'message'"
        :dimmed="applyingVia !== null && applyingVia !== 'message'"
        :disabled="applyingVia !== null"
        @select="confirmApply('message', applyByMessage)"
      />
      <OptionCard
        icon="send"
        :title="t('detail.apply.none_title')"
        :description="applyingVia === 'sans_message' ? t('detail.apply.none_writing') : t('detail.apply.none_description')"
        :busy="applyingVia === 'sans_message'"
        :dimmed="applyingVia !== null && applyingVia !== 'sans_message'"
        :disabled="applyingVia !== null"
        @select="confirmApply('sans_message', applyWithoutMessage)"
      />
    </div>
    <div class="text-[12px] text-neutral-500 mt-[14px]">{{ t('detail.apply.status_note') }}</div>
  </AppModal>
</template>
