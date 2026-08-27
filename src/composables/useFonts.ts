/**
 * Gestion des polices utilisateur importées pour la compilation Typst :
 * chargement de la liste, import depuis un dossier source, suppression.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { FontFamilyView } from '@/types'

const fonts = ref<FontFamilyView[]>([])
const loading = ref(false)

/**
 * Charge la liste des familles de polices importées.
 *
 * @returns `null` si succès, sinon un message d'erreur.
 */
async function loadFonts(): Promise<string | null> {
  loading.value = true
  try {
    fonts.value = await invoke<FontFamilyView[]>('list_fonts')
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de charger les polices.')
  } finally {
    loading.value = false
  }
}

/**
 * Importe le contenu d'un dossier de police vers l'application, puis
 * recharge la liste.
 *
 * @param sourcePath - Chemin du dossier source sélectionné par l'utilisateur.
 * @returns `null` si succès, sinon un message d'erreur.
 */
async function importFontFolder(sourcePath: string): Promise<string | null> {
  try {
    await invoke<FontFamilyView>('import_font_folder', { sourcePath })
    return await loadFonts()
  } catch (e: unknown) {
    return extractErrorMessage(e, "Impossible d'importer ce dossier de police.")
  }
}

/**
 * Supprime une famille de polices.
 *
 * @param name - Nom de la famille à supprimer.
 * @returns `null` si succès, sinon un message d'erreur.
 */
async function deleteFont(name: string): Promise<string | null> {
  try {
    await invoke('delete_font', { name })
    fonts.value = fonts.value.filter((f) => f.name !== name)
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de supprimer cette police.')
  }
}

/** Expose la liste des polices (réactive) et les actions pour la gérer. */
export function useFonts() {
  return { fonts, loading, loadFonts, importFontFolder, deleteFont }
}
