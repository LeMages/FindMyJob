/**
 * Insertion d'une puce (« • ») dans un textarea, au niveau du curseur ou de
 * la sélection courante — utilisé par les champs de description du profil.
 */

// Importation des modules Vue / framework
import { nextTick } from 'vue'

/**
 * Insère une puce dans un textarea à la position du curseur (ou remplace la
 * sélection), puis replace le curseur juste après la puce insérée.
 *
 * @param el - Élément textarea ciblé.
 * @param value - Valeur actuelle du textarea.
 * @param onUpdate - Callback recevant la nouvelle valeur (ex. `v-model` setter).
 */
export function insertBullet(
  el: HTMLTextAreaElement,
  value: string,
  onUpdate: (newValue: string) => void
): void {
  const start = el.selectionStart
  const end = el.selectionEnd
  const hasSelection = start !== end
  const toInsert = hasSelection
    ? '• '
    : start > 0 && value[start - 1] !== '\n'
    ? '\n• '
    : '• '
  onUpdate(value.slice(0, start) + toInsert + value.slice(end))
  const newCursor = start + toInsert.length
  nextTick(() => {
    el.setSelectionRange(newCursor, newCursor)
    el.focus()
  })
}
