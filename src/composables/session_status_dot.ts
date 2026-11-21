/**
 * Couleur de la pastille de statut d'une session (barre latérale, bandeau
 * « lecture seule ») : vert = active, ambre = en pause, rouge = terminée.
 * Les classes reposent sur les variables du thème (`styles.css`), redéfinies
 * en thème sombre : aucune variante `dark:` n'est nécessaire.
 */

// Importation des modules internes
import type { SessionStatut } from '@/types'

/** Classe Tailwind de fond de la pastille, par statut de session. */
export const SESSION_STATUS_DOT_CLASS: Record<SessionStatut, string> = {
  active: 'bg-green-400',
  pause: 'bg-amber-300',
  termine: 'bg-red-400',
}
