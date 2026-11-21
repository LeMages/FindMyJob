/**
 * Sections éditables du profil et règle « une seule carte éditable à la
 * fois » : le mode d'affichage d'une carte dépend de la section en cours
 * d'édition.
 */

/** Section du profil éditable séparément. */
export type ProfileSection = 'info' | 'about' | 'experiences' | 'formations' | 'projets' | 'skills' | 'langues'

/** Mode d'affichage d'une carte : lecture, en édition, ou estompée pendant l'édition d'une autre. */
export type ProfileCardMode = 'idle' | 'editing' | 'dimmed'

/**
 * Mode d'affichage de la carte d'une section.
 *
 * @param section - Section de la carte.
 * @param editing - Section en cours d'édition, `null` si aucune.
 * @returns `'editing'` pour la carte éditée, `'dimmed'` pour les autres
 *   pendant une édition, `'idle'` sinon.
 */
export function cardMode(section: ProfileSection, editing: ProfileSection | null): ProfileCardMode {
  if (editing === null) return 'idle'
  return editing === section ? 'editing' : 'dimmed'
}
