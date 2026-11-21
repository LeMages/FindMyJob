Voici ma lettre de motivation actuelle (en pièce jointe, quel que soit son format : PDF, Canva, Word, .typ…). Convertis-la en un fichier .typ compatible avec FindMyJob, qui passe aussi les ATS.

Règles à suivre :

1. Conserve la mise en page, les couleurs, les polices et le style au plus près de l'original.
2. Le fichier respecte le contrat FindMyJob ci-dessous : valeurs d'exemple dans `example`, données lues dans `data`, métadonnée `<findmyjob>` unique, textes fixes en français et en anglais choisis par `data.lang`.
3. Mon identité, mes coordonnées, l'entreprise destinataire, la date et l'objet sont lus dans `data`, jamais écrits en dur. Les valeurs d'exemple sont celles de ma lettre.
4. Les paragraphes qui changent d'une candidature à l'autre deviennent des emplacements rédigés : un élément de `data.paragraphs` chacun, avec un emplacement (`slots`) `variable` (tiré de l'offre) ou `adaptable` (tiré de mon profil) et une consigne. Leur texte d'exemple donne le style et la longueur attendus.
5. Les phrases qui ne changent jamais (formule d'appel, présentation fixe, formule de politesse…) restent en texte fixe, en français et en anglais, placées autour des paragraphes rédigés. Un paragraphe peut mêler une phrase fixe et un emplacement rédigé.
6. Mes consignes générales (ton, longueur, ce qu'il ne faut jamais écrire) vont dans `guidance`.
7. Le bloc destinataire, s'il est à droite, est aussi large que sa plus longue ligne : `#align(right, box({ set align(left); … }))`. Une ligne vide disparaît.
8. Règles ATS : tout le texte est sélectionnable ; aucune information importante n'est seulement dans une image ou une icône ; coordonnées en texte brut ; pas de tableau pour les éléments clés.
9. Aucun tiret cadratin ni demi-cadratin (« — », « – ») : utilise « - ».
10. Si une police de l'original n'est pas disponible, choisis une police proche et indique-la-moi, pour que je l'importe dans FindMyJob (Paramètres > Polices).
11. Réponds avec un seul fichier .typ complet, compilé avec Typst 0.15 si possible.

Contrat complet (aussi consultable ici : {{DOC_URL}}) :

<<<CONTRAT
{{CONTRACT}}
CONTRAT>>>

Exemple de lettre conforme (personnage fictif), à suivre pour la structure du fichier, pas pour le contenu :

<<<EXEMPLE
{{EXAMPLE}}
EXEMPLE>>>
