//! Conversion d'un document de référence Typst (CV ou lettre de motivation)
//! en un template réutilisable et en son jeu de données d'exemple.
//!
//! Appelé une seule fois, à l'upload de la référence (ou sur demande de
//! re-templatisation). La référence est la source de vérité de la
//! structure et du texte : rien n'en est retiré. Le template lit depuis un
//! fichier `data.json` au schéma canonique (`models::doc_data`) tout ce que
//! le profil, l'offre ou la rédaction fournissent, et garde le reste du
//! texte écrit en dur. Les commentaires de consignes de la référence
//! (`[FIXE]`, `[VARIABLE]`, `[ADAPTABLE]`) sont lus par Rust
//! (`documents::markup`), jamais par le LLM. La génération d'une
//! candidature ne fait plus appel à un LLM pour produire du Typst. La
//! validation (compilation, nombre de pages, texte) vit chez l'appelant,
//! qui renvoie ses erreurs ici en feedback via `TemplatizeFeedback`.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude_with_tokens;
use crate::llm::doc_data_schema::{cv_example_data_schema, lm_example_data_schema, object};
use crate::llm::target::ClaudeTarget;

/// Plafond de tokens de sortie : le template Typst complet et les données
/// d'exemple tiennent dans une seule réponse JSON, plus longue qu'une
/// réponse d'analyse classique.
const TEMPLATIZE_MAX_TOKENS: u32 = 16384;

/// Résultat brut d'une templatisation, avant validation par l'appelant.
///
/// # Attributes
///
/// * `template_typ` - Source Typst du template (lit `data.json`).
/// * `example_data` - Contenu de la référence au schéma canonique, avec
///   `detected_lang` (et `period_style` pour un CV).
pub struct TemplatizeOutput {
    pub template_typ: String,
    pub example_data: serde_json::Value,
}

/// Retour de validation d'une tentative précédente, transmis au LLM pour
/// qu'il corrige sa sortie plutôt que de repartir de zéro.
///
/// # Attributes
///
/// * `previous_template` - Template produit à la tentative précédente.
/// * `previous_example_data` - Données d'exemple produites à la tentative précédente.
/// * `problems` - Problèmes relevés (erreurs du compilateur, écart de pages ou de texte).
pub struct TemplatizeFeedback {
    pub previous_template: String,
    pub previous_example_data: String,
    pub problems: Vec<String>,
}

/// Règles communes aux deux types de documents, pour le template.
const TEMPLATE_RULES: &str = "Règles du template :
1. Recopie à l'identique le préambule (#set page, #set text, #set par), les polices, les couleurs, les fonctions définies avec #let, les espacements (#v, #h, below, above, pad) et les commentaires. Seule exception : la langue du texte devient `lang: data.lang`.
2. Charge les données une seule fois avec `#let data = json(\"data.json\")`, en toute première ligne du template, avant le préambule : `data` doit exister partout où il est utilisé, y compris dans `#set text(lang: data.lang)`.
3. L'original est la source de vérité de la structure et du texte : rien n'est jamais retiré. Passent dans `data` uniquement les valeurs décrites par le schéma : informations du candidat (identité, coordonnées, lieu, expériences, projets, formations, compétences, langues, centres d'intérêt), destinataire et date d'une lettre, prose rédigée pour chaque offre (titre, résumé, bullets, objet, paragraphes) et libellés (titres de sections, formules d'appel et de politesse). Tout autre texte de l'original (une mention, une phrase, une ligne qui n'a pas de champ dans le schéma) est recopié mot pour mot dans le template, au même endroit et avec la même mise en forme.
4. Toutes les listes sont rendues par des boucles `for` : jamais un élément écrit à la main.
5. Les valeurs sont insérées comme des chaînes, par exemple `text(data.summary)` ou `[#data.summary]`. N'utilise jamais `eval`, ni `datetime.today`, ni `read`.
6. Un champ vide (chaîne vide ou liste vide) fait disparaître sa ligne entière, ou sa valeur et le séparateur qui l'accompagne (« : », « | », « , »…), sans laisser de séparateur orphelin. Pour une ligne qui enchaîne plusieurs valeurs avec un séparateur, construis la liste des valeurs non vides puis joins-la avec le séparateur. Une section dont le titre ou la liste est vide disparaît entièrement, titre compris.
7. Si l'original affiche une valeur transformée (par exemple le nom en majuscules), range la valeur naturelle dans les données et applique la transformation dans le template (`upper`).
8. Une valeur qui a un champ dans le schéma n'est jamais écrite en dur, même au milieu d'un texte recopié : si une phrase recopiée contient par exemple la ville ou le nom du candidat, le template insère cette valeur depuis `data` dans la phrase. Une date calculée par l'original (datetime.today) est remplacée par data.date.
9. Le template doit compiler avec Typst 0.15, y compris quand tous les champs sont vides.";

/// Règles communes aux deux types de documents, pour les données d'exemple.
const EXAMPLE_DATA_RULES: &str = "Règles de example_data :
1. Recopie chaque valeur exactement telle qu'elle est écrite dans l'original (mêmes mots, même ponctuation), sauf la casse transformée par le template.
2. Un champ absent de l'original vaut une chaîne vide ou une liste vide.
3. `lang` et `detected_lang` valent la langue dans laquelle l'original est rédigé.
4. `identity.permis` vaut true seulement si l'original affiche le permis de conduire.";

/// Règle de lieu commune aux deux types de documents : le profil stocke le
/// lieu du candidat en deux champs, `ville` et `zip_code`, que la
/// génération place dans `identity`.
const ADDRESS_RULE: &str = "Lieu de l'expéditeur :
- Le lieu du candidat tient en deux champs : `identity.ville` et `identity.zip_code`. Une rue, un numéro, un complément d'adresse ou un pays de l'expéditeur n'ont pas de champ : ils restent écrits en dur dans le template, tels quels.
- Le template compose lui-même chaque affichage du lieu à partir de ces deux champs, dans l'ordre et avec la ponctuation de l'original (par exemple \"74300 Thyez\" ou \"Thyez, 74300\") : il construit la liste des valeurs non vides puis la joint avec le séparateur de l'original, comme la règle 6. Si l'un des deux est vide, aucun séparateur orphelin ne reste ; si les deux sont vides, la ligne disparaît.
- Si l'original affiche le lieu à plusieurs endroits sous des formes différentes (par exemple ville et code postal dans l'en-tête, ville seule dans la ligne de date d'une lettre), chaque endroit compose sa propre forme à partir des mêmes champs.
- example_data reprend la ville et le code postal exactement tels qu'affichés dans l'original (chaîne vide si absent).";

/// Consignes propres au CV : sens des champs du schéma et rendu attendu.
const CV_FIELDS: &str = "Champs du CV :
- identity : coordonnées du candidat. Le template n'affiche que les coordonnées présentes dans l'original, dans le même ordre. `labels.permis` s'affiche à la place du permis, seulement si `identity.permis` est vrai.
- identity.title : sous-titre professionnel affiché sous le nom (par exemple \"Étudiant ingénieur data\"), chaîne vide si l'original n'en a pas. summary : texte de la section profil.
- skills_groups : une ligne par groupe, la catégorie puis les compétences jointes par \", \".
- experiences, projects : une entrée par expérience ou projet, avec ses bullets. `pour_qui` est l'organisation affichée pour un projet.
- projects.stack et projects.url : les technologies et le lien affichés pour un projet. Si l'original les affiche, ils sont lus depuis ces champs, jamais écrits en dur. S'il ne les affiche pas, ils valent une chaîne vide dans example_data et le template ne les lit pas : n'ajoute jamais un affichage absent de l'original.
- formations : une entrée par formation, y compris celles que l'original affiche sur une seule ligne ; `details` reçoit les lignes de détail de la formation (spécialité, cours, échange).
- Si l'original affiche des éléments d'une même liste avec des mises en page différentes, reproduis chacune telle quelle : dans la boucle, le template choisit la mise en page d'un élément d'après ses données (par exemple selon qu'un champ est vide).
- langues : nom et niveau de chaque langue. Un séparateur entre le nom et le niveau (par exemple « : ») ne s'affiche que si le niveau n'est pas vide. interets : un centre d'intérêt par élément, affichés joints par \", \" après `labels.interets_label`.
- period : la période telle qu'affichée. `period_style` indique pour chaque section si les périodes affichent le mois (\"month\") ou seulement l'année (\"year\") ; mets \"month\" pour une section absente.
- labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original.
- labels.months_short : les 12 abréviations de mois, de janvier à décembre, dans la langue et le style de l'original (si l'original écrit \"Nov.\", écris \"Janv.\", \"Févr.\", etc.).
- labels.present : le mot qui termine une période en cours dans la langue de l'original (par exemple \"aujourd'hui\"), même s'il n'apparaît pas.";

/// Consignes propres à la lettre : sens des champs du schéma et rendu attendu.
const LM_FIELDS: &str = "Champs de la lettre :
- identity : coordonnées de l'expéditeur. Le template n'affiche que les coordonnées présentes dans l'original, dans le même ordre ; la signature est construite avec prénom et nom. `identity.title` est le sous-titre professionnel affiché sous le nom de l'expéditeur, chaîne vide si l'original n'en a pas.
- recipient : destinataire. company et location, puis extra_lines : les autres lignes du destinataire de l'original (service, adresse postale), une par élément, dans l'ordre. Chaque ligne s'affiche seulement si elle est non vide ; extra_lines est rendu par une boucle qui garde la mise en forme des lignes de l'original.
- date : la date affichée, déjà formatée. La ligne de date compose `identity.ville` suivi de \", \" (les deux omis si la ville est vide), `labels.date_prefix` s'il n'est pas vide, puis `data.date`. Elle n'affiche jamais le code postal. Si l'original calcule la date au lieu de l'écrire, mets une chaîne vide dans example_data.
- subject : le texte de l'objet, sans son préfixe. `labels.subject_prefix` contient le préfixe (par exemple \"Objet :\").
- labels.salutation : la formule d'appel. labels.closing : la formule de politesse finale, en entier.
- paragraphs : les paragraphes du corps, entre la formule d'appel et la formule de politesse finale. Une entrée de paragraphs = un paragraphe de l'original, c'est-à-dire ce qui est séparé du texte voisin par une ligne vide, #v, parbreak(), \\ ou un saut de bloc. Des lignes séparées seulement par un commentaire forment un seul paragraphe. Une entrée n'est jamais une zone balisée : si un paragraphe contient plusieurs commentaires de balise ([FIXE], [VARIABLE], [ADAPTABLE]), ses lignes restent dans une seule entrée.
- Rendu de paragraphs : une boucle où chaque entrée devient un paragraphe distinct qui commence sur une nouvelle ligne, séparé du précédent par le même séparateur et le même espacement vertical que dans l'original (par exemple `#for p in data.paragraphs { p; parbreak() }` si l'original sépare ses paragraphes par une ligne vide). Interdits pour paragraphs : `[#p]` ou `{ p }` ou `text(p)` sans séparateur entre deux entrées, `.join()` et `.join(\" \")` : ils collent les paragraphes ou les fondent en un seul bloc.";

/// Libellé lisible du type de document, pour les prompts.
fn doc_type_label(doc_type: &str) -> &'static str {
    match doc_type {
        "cv" => "CV",
        _ => "lettre de motivation",
    }
}

/// Construit le message utilisateur : document original, puis feedback de
/// la tentative précédente s'il y en a un.
fn build_user_message(reference_typ: &str, feedback: Option<&TemplatizeFeedback>) -> String {
    let mut message = format!("DOCUMENT ORIGINAL :\n```typst\n{reference_typ}\n```");
    if let Some(feedback) = feedback {
        let problems = feedback
            .problems
            .iter()
            .map(|p| format!("- {p}"))
            .collect::<Vec<_>>()
            .join("\n");
        message.push_str(&format!(
            "\n\nTa tentative précédente n'a pas passé la validation. Corrige-la en gardant ce qui fonctionne.\n\n\
             PROBLÈMES RELEVÉS :\n{problems}\n\n\
             TEMPLATE PRÉCÉDENT :\n```typst\n{}\n```\n\n\
             EXAMPLE_DATA PRÉCÉDENT :\n```json\n{}\n```",
            feedback.previous_template, feedback.previous_example_data
        ));
    }
    message
}

/// Schéma de sortie de la templatisation d'un type de document.
///
/// # Arguments
///
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
///
/// # Returns
///
/// Le schéma, ou `None` pour un type inconnu.
pub fn templatize_output_schema(doc_type: &str) -> Option<serde_json::Value> {
    match doc_type {
        "cv" => Some(output_schema(cv_example_data_schema())),
        "lettre_motivation" => Some(output_schema(lm_example_data_schema())),
        _ => None,
    }
}

/// Enveloppe la sortie de la templatisation autour du schéma des données.
fn output_schema(data_schema: serde_json::Value) -> serde_json::Value {
    object(&[
        ("template_typ", serde_json::json!({ "type": "string" })),
        ("example_data", data_schema),
    ])
}

/// Convertit un document de référence Typst en template + données d'exemple.
///
/// # Arguments
///
/// * `reference_typ` - Source Typst complète de la référence.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `feedback` - Retour de validation de la tentative précédente, `None` au premier essai.
/// * `target` - Clé API et modèle Claude à utiliser.
///
/// # Returns
///
/// `TemplatizeOutput` brut, à valider par l'appelant (compilation, pages, texte).
pub async fn templatize_reference(
    reference_typ: &str,
    doc_type: &str,
    feedback: Option<&TemplatizeFeedback>,
    target: &ClaudeTarget,
) -> Result<TemplatizeOutput, AppError> {
    if reference_typ.trim().is_empty() {
        return Err(AppError::Validation("Le document de référence est vide.".to_string()));
    }

    let (fields, data_schema) = match doc_type {
        "cv" => (CV_FIELDS, cv_example_data_schema()),
        "lettre_motivation" => (LM_FIELDS, lm_example_data_schema()),
        other => return Err(AppError::Validation(format!("Type de document inconnu : {other}"))),
    };
    let label = doc_type_label(doc_type);

    let system_prompt = format!(
        "Tu convertis un document Typst ({label}) en un template réutilisable. Tu produis deux éléments.\n\n\
         template_typ : le même document, dont les valeurs décrites par le schéma sont lues depuis un fichier \
         JSON, et tout le reste recopié tel quel. Rendu avec les données d'exemple, il doit être visuellement \
         identique à l'original : même texte, mot pour mot, même mise en page, même nombre de pages.\n\n\
         example_data : le contenu de l'original rangé dans le schéma JSON imposé. \
         Le template lit exactement ces champs, avec `data` à la racine.\n\n\
         {TEMPLATE_RULES}\n\n{EXAMPLE_DATA_RULES}\n\n{fields}\n\n{ADDRESS_RULE}"
    );
    let user_message = build_user_message(reference_typ, feedback);
    let schema = output_schema(data_schema);

    let raw = call_claude_with_tokens("templatisation", &system_prompt, &user_message, Some(schema), TEMPLATIZE_MAX_TOKENS, target).await?;
    parse_output(&raw)
}

/// Désérialise la réponse JSON du LLM en `TemplatizeOutput`.
fn parse_output(raw: &str) -> Result<TemplatizeOutput, AppError> {
    let mut parsed: serde_json::Value = serde_json::from_str(raw.trim()).map_err(|e| {
        AppError::Llm(format!("La templatisation n'a pas retourné du JSON valide : {e}\nRéponse brute : {raw}"))
    })?;
    let template_typ = parsed
        .get("template_typ")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| AppError::Llm("Réponse de templatisation sans champ template_typ.".to_string()))?;
    let example_data = parsed
        .get_mut("example_data")
        .map(serde_json::Value::take)
        .ok_or_else(|| AppError::Llm("Réponse de templatisation sans champ example_data.".to_string()))?;
    Ok(TemplatizeOutput { template_typ, example_data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sortie_sans_consignes() {
        let output = parse_output(r#"{"template_typ": "x", "example_data": {"lang": "fr"}}"#).expect("sortie valide");
        assert_eq!(output.template_typ, "x");
        assert!(output.example_data.get("guidance").is_none());
    }

    #[test]
    fn consignes_hors_des_schemas() {
        // Les consignes sont lues par Rust dans les commentaires : le LLM ne
        // les produit plus.
        for schema in [cv_example_data_schema(), lm_example_data_schema()] {
            assert!(schema["properties"].get("guidance").is_none());
        }
        for doc_type in ["cv", "lettre_motivation"] {
            let schema = templatize_output_schema(doc_type).expect("type connu");
            assert!(schema["properties"].get("guidance").is_none());
        }
    }
}
