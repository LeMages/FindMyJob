/**
 * Découpage d'une description du profil (expérience, projet, formation) en
 * intro et éléments de liste, pour l'affichage dans l'app.
 *
 * Miroir exact de `src-tauri/src/docgen/description_list.rs` (source de la
 * règle, utilisée pour le CV) : les deux parseurs sont testés sur la même
 * table de cas (`src-tauri/tests/fixtures/description_list_cases.json`).
 * Règle : une ligne qui commence par `-` est un élément, sauf si le `-` est
 * suivi d'un chiffre (`-30 %`) ; les lignes avant le premier élément forment
 * l'intro ; une ligne sans tiret après un élément en est la suite.
 */

/** Description découpée : lignes d'intro, puis éléments (première ligne et suites). */
export interface DescriptionList {
  intro: string[]
  items: string[][]
}

/**
 * Contenu d'un élément si la ligne (déjà trimée) en ouvre un, `null` si
 * c'est du texte. Un contenu vide (`-` seul) donne `''`.
 */
function itemContent(line: string): string | null {
  if (!line.startsWith('-')) return null
  const rest = line.slice(1)
  // Un `-` collé à un chiffre est un signe (« -30 % »), pas un élément.
  if (/^[0-9]/.test(rest)) return null
  return rest.trim()
}

/**
 * Découpe une description du profil en intro et éléments.
 *
 * @param text - Description saisie (absente = vide).
 * @returns Intro et éléments ; lignes vides ignorées, lignes trimées.
 */
export function parseDescriptionList(text: string | null | undefined): DescriptionList {
  const list: DescriptionList = { intro: [], items: [] }
  if (!text) return list
  // `split('\n')` puis `trim` (qui retire un `\r` final) : même découpage
  // que le parseur Rust, quelle que soit la fin de ligne.
  const lines = text.split('\n').map(line => line.trim()).filter(line => line.length > 0)
  for (const line of lines) {
    const content = itemContent(line)
    if (content === '') continue
    if (content !== null) {
      list.items.push([content])
      continue
    }
    const last = list.items[list.items.length - 1]
    if (last) last.push(line)
    else list.intro.push(line)
  }
  return list
}
