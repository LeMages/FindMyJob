<script setup lang="ts">
/**
 * Carte « Documents » de la fiche candidature : le CV et la lettre générés,
 * avec taille, version du modèle, aperçu et téléchargement en icônes (ou
 * « Fichier introuvable »), puis les avertissements de la génération
 * regroupés dans un encadré ambre. Propre à `DetailView`.
 */

// Importation des modules Vue / framework
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

// Importation des composants
import IconButton from '@/components/IconButton.vue'
import SectionCard from '@/components/SectionCard.vue'
import UiIcon from '@/components/UiIcon.vue'

// Importation des modules internes
import { state } from '@/stores/app'
import { formatFileSize } from '@/composables/format_file_size'
import { useDocumentDownload } from '@/composables/useDocumentDownload'
import type { DownloadDocType } from '@/composables/document_file_name'

const emit = defineEmits<{
  /** Aperçu demandé : chemin, titre de la modale, type du document. */
  preview: [path: string, label: string, docType: DownloadDocType]
}>()

const { t } = useI18n()
const { downloading, download } = useDocumentDownload()

/** Une ligne par document généré, dans l'ordre CV puis lettre. */
const documents = computed(() => {
  const c = state.detailCandidature
  return [
    {
      docType: 'cv' as const,
      title: t('detail.documents.cv'),
      previewTitle: t('detail.documents.cv_preview_title'),
      path: c?.cv_path ?? null,
      info: state.detailCvInfo,
      version: c?.cv_reference_version ?? null,
    },
    {
      docType: 'lm' as const,
      title: t('detail.documents.cover_letter'),
      previewTitle: t('detail.documents.lm_preview_title'),
      path: c?.lettre_motivation_path ?? null,
      info: state.detailLettreInfo,
      version: c?.lm_reference_version ?? null,
    },
  ].map((doc) => ({ ...doc, exists: !!doc.path && !!doc.info?.exists }))
})

const warnings = computed(() => state.detailCandidature?.generation_warnings ?? [])
</script>

<template>
  <SectionCard :title="t('detail.documents.title')" :hint="t('detail.documents.hint')">
    <div
      v-for="(doc, i) in documents"
      :key="doc.docType"
      class="flex items-center gap-3 py-[10px]"
      :class="i > 0 ? 'border-t border-neutral-100' : ''"
    >
      <div class="w-9 h-9 rounded-9 bg-neutral-100 text-neutral-700 flex items-center justify-center flex-none">
        <UiIcon name="document" :size="18" />
      </div>
      <div class="flex-1 min-w-0">
        <div class="text-[13.5px] font-medium text-neutral-950">{{ doc.title }}</div>
        <div class="text-[11.5px] text-neutral-500 font-mono">
          <template v-if="doc.exists">{{ t('detail.documents.file_meta', { size: formatFileSize(doc.info?.size ?? 0) }) }}</template>
          <span v-else class="text-red-600">{{ t('detail.documents.file_missing') }}</span>
          <template v-if="doc.version != null"> · {{ t('detail.documents.template_version', { version: doc.version }) }}</template>
        </div>
      </div>
      <template v-if="doc.exists && doc.path">
        <IconButton :title="t('detail.documents.preview')" @click="emit('preview', doc.path, doc.previewTitle, doc.docType)">
          <UiIcon name="eye" />
        </IconButton>
        <IconButton :title="t('detail.documents.download')" :disabled="downloading !== null" @click="download(doc.docType, doc.path)">
          <UiIcon name="download" />
        </IconButton>
      </template>
    </div>

    <!-- Avertissements de la génération : texte libre du backend, affiché tel quel -->
    <div v-if="warnings.length > 0" class="mt-3 rounded-10 bg-amber-50 border border-amber-200 px-3 py-[10px]">
      <div class="flex items-center gap-[6px] text-[12.5px] font-semibold text-amber-700">
        <UiIcon name="alert" :size="14" /> {{ t('detail.documents.warnings_count', warnings.length) }}
      </div>
      <div v-for="warning in warnings" :key="warning" class="text-[12px] text-amber-700 mt-[6px] leading-[1.45]">{{ warning }}</div>
    </div>
  </SectionCard>
</template>
