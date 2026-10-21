/**
 * Gestion des polices utilisateur importées pour la compilation Typst :
 * chargement de la liste (familles et variantes lues dans les fichiers),
 * import depuis un dossier source, suppression.
 */

// Importation des modules Vue / framework
import { ref } from 'vue'

// Importation des modules externes
import { invoke } from '@tauri-apps/api/core'

// Importation des modules internes
import { extractErrorMessage } from '@/composables/useErrorMessage'
import type { FontFolderView } from '@/types'

const fonts = ref<FontFolderView[]>([])
const loading = ref(false)

/** Résultat d'un import : le dossier importé, ou un message d'erreur. */
export type FontImportResult = { imported: FontFolderView; error: null } | { imported: null; error: string }

/**
 * Charge la liste des dossiers de polices importés, avec leurs familles.
 *
 * @returns `null` si succès, sinon un message d'erreur.
 */
async function loadFonts(): Promise<string | null> {
  loading.value = true
  try {
    fonts.value = await invoke<FontFolderView[]>('list_fonts')
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
 * @returns Le dossier importé (familles, homonymes), ou un message d'erreur.
 */
async function importFontFolder(sourcePath: string): Promise<FontImportResult> {
  let imported: FontFolderView
  try {
    imported = await invoke<FontFolderView>('import_font_folder', { sourcePath })
  } catch (e: unknown) {
    return { imported: null, error: extractErrorMessage(e, "Impossible d'importer ce dossier de police.") }
  }
  const loadError = await loadFonts()
  return loadError ? { imported: null, error: loadError } : { imported, error: null }
}

/**
 * Supprime un dossier de polices importé.
 *
 * @param folder - Nom du dossier à supprimer.
 * @returns `null` si succès, sinon un message d'erreur.
 */
async function deleteFont(folder: string): Promise<string | null> {
  try {
    await invoke('delete_font', { folder })
    fonts.value = fonts.value.filter((f) => f.folder !== folder)
    return null
  } catch (e: unknown) {
    return extractErrorMessage(e, 'Impossible de supprimer cette police.')
  }
}

/** Expose la liste des polices (réactive) et les actions pour la gérer. */
export function useFonts() {
  return { fonts, loading, loadFonts, importFontFolder, deleteFont }
}
