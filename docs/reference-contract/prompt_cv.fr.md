Voici mon CV actuel (en pièce jointe, quel que soit son format : PDF, Canva, Word, .typ…). Convertis-le en un fichier .typ compatible avec FindMyJob, qui passe aussi les ATS.

Règles à suivre :

1. Conserve la mise en page, les couleurs, les polices et le style au plus près de l'original.
2. Le fichier respecte le contrat FindMyJob ci-dessous : valeurs d'exemple dans `example`, données lues dans `data`, métadonnée `<findmyjob>` unique, textes fixes en français et en anglais choisis par `data.lang`.
3. Toutes les informations qui viennent de mon profil (identité, coordonnées, expériences, projets, formations, compétences, langues) sont lues dans `data`, jamais écrites en dur. Les valeurs d'exemple sont celles de mon CV.
4. Les sections que FindMyJob ne connaît pas (centres d'intérêt, références, bénévolat…) restent en texte fixe, en français et en anglais.
5. Les titres de section sont des `heading`, mis en forme avec une règle `#show heading`. Une ligne, une section ou un séparateur dont la donnée est vide disparaît, titre compris.
6. `data.list_marker` précède chaque élément de description (`bullets`, `details`).
7. Règles ATS : tout le texte est sélectionnable ; aucune information importante n'est seulement dans une image ou une icône ; titres de sections standards ; dates cohérentes ; coordonnées en texte brut ; pas de tableau pour les éléments clés.
8. Aucun tiret cadratin ni demi-cadratin (« — », « – ») : utilise « - ».
9. Si une police de l'original n'est pas disponible, choisis une police proche et indique-la-moi, pour que je l'importe dans FindMyJob (Paramètres > Polices).
10. Réponds avec un seul fichier .typ complet, compilé avec Typst 0.15 si possible.

Contrat complet (aussi consultable ici : {{DOC_URL}}) :

<<<CONTRAT
{{CONTRACT}}
CONTRAT>>>

Exemple de CV conforme (personnage fictif), à suivre pour la structure du fichier, pas pour le contenu :

<<<EXEMPLE
{{EXAMPLE}}
EXEMPLE>>>
