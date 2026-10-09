//! Conversion d'un document de référence Typst (CV ou lettre de motivation)
//! en un template réutilisable et en son jeu de données d'exemple.
//!
//! Appelé une seule fois, à l'upload de la référence (ou sur demande de
//! re-templatisation). Le template produit lit toutes ses valeurs depuis un
//! fichier `data.json` au schéma canonique (`models::doc_data`) ; la
//! génération d'une candidature ne fait plus appel à un LLM pour produire du
//! Typst. La validation (compilation, nombre de pages, texte) vit chez
//! l'appelant, qui renvoie ses erreurs ici en feedback via `TemplatizeFeedback`.

// Importation des modules internes
use crate::error::AppError;
use crate::llm::client::call_claude_with_tokens;
use crate::llm::doc_data_schema::{cv_example_data_schema, lm_example_data_schema, object};

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
/// * `dropped_text` - Textes de la référence volontairement absents du
///   template (éléments sans champ dans le schéma), tels qu'affichés.
pub struct TemplatizeOutput {
    pub template_typ: String,
    pub example_data: serde_json::Value,
    pub dropped_text: Vec<String>,
}

/// Retour de validation d'une tentative précédente, transmis au LLM pour
/// qu'il corrige sa sortie plutôt que de repartir de zéro.
///
/// # Attributes
///
/// * `previous_template` - Template produit à la tentative précédente.
/// * `previous_example_data` - Données d'exemple produites à la tentative précédente.
/// * `previous_dropped_text` - Textes déclarés supprimés à la tentative précédente.
/// * `problems` - Problèmes relevés (erreurs du compilateur, écart de pages ou de texte).
pub struct TemplatizeFeedback {
    pub previous_template: String,
    pub previous_example_data: String,
    pub previous_dropped_text: Vec<String>,
    pub problems: Vec<String>,
}

/// Règles communes aux deux types de documents, pour le template.
const TEMPLATE_RULES: &str = "Règles du template :
1. Recopie à l'identique le préambule (#set page, #set text, #set par), les polices, les couleurs, les fonctions définies avec #let, les espacements (#v, #h, below, above, pad) et les commentaires. Seule exception : la langue du texte devient `lang: data.lang`.
2. Charge les données une seule fois avec `#let data = json(\"data.json\")`, en toute première ligne du template, avant le préambule : `data` doit exister partout où il est utilisé, y compris dans `#set text(lang: data.lang)`.
3. Aucune valeur de contenu ne reste en dur : identité, coordonnées, titres de sections, intitulés, entreprises, dates, textes, formules d'appel et de politesse viennent tous de `data`. Seuls les séparateurs graphiques (puces, barres verticales, points médians) peuvent rester écrits dans le template.
4. Toutes les listes sont rendues par des boucles `for` : jamais un élément écrit à la main.
5. Les valeurs sont insérées comme des chaînes, par exemple `text(data.summary)` ou `[#data.summary]`. N'utilise jamais `eval`, ni `datetime.today`, ni `read`.
6. Un champ vide (chaîne vide ou liste vide) fait disparaître sa ligne entière, sans laisser de séparateur orphelin. Pour une ligne qui enchaîne plusieurs valeurs avec un séparateur, construis la liste des valeurs non vides puis joins-la avec le séparateur. Une section dont le titre ou la liste est vide disparaît entièrement, titre compris.
7. Si l'original affiche une valeur transformée (par exemple le nom en majuscules), range la valeur naturelle dans les données et applique la transformation dans le template (`upper`).
8. Un élément de l'original qui n'a aucun champ correspondant dans le schéma est supprimé du template. Il n'est jamais recopié en dur. Un élément qui a un champ dans le schéma (une formation, une langue, etc.) figure toujours dans example_data, jamais dans dropped_text. Liste les textes supprimés dans dropped_text, un par ligne, tels qu'ils s'affichent dans l'original. Une date calculée par l'original (datetime.today) est remplacée par data.date et n'a pas à y figurer. dropped_text est une chaîne vide si rien n'est supprimé.
9. Le template doit compiler avec Typst 0.15, y compris quand tous les champs sont vides.";

/// Règles communes aux deux types de documents, pour les données d'exemple.
const EXAMPLE_DATA_RULES: &str = "Règles de example_data :
1. Recopie chaque valeur exactement telle qu'elle est écrite dans l'original (mêmes mots, même ponctuation), sauf la casse transformée par le template.
2. Un champ absent de l'original vaut une chaîne vide ou une liste vide.
3. `lang` et `detected_lang` valent la langue dans laquelle l'original est rédigé.
4. `identity.permis` vaut true seulement si l'original affiche le permis de conduire.";

/// Consignes propres au CV : sens des champs du schéma et rendu attendu.
const CV_FIELDS: &str = "Champs du CV :
- identity : coordonnées du candidat. Le template n'affiche que les coordonnées présentes dans l'original, dans le même ordre. `labels.permis` s'affiche à la place du permis, seulement si `identity.permis` est vrai.
- title : titre affiché sous le nom. summary : texte de la section profil.
- skills_groups : une ligne par groupe, la catégorie puis les compétences jointes par \", \".
- experiences, projects : une entrée par expérience ou projet, avec ses bullets. `pour_qui` est l'organisation affichée pour un projet.
- formations : une entrée par formation, y compris celles que l'original affiche sur une seule ligne ; `details` reçoit les lignes de détail de la formation (spécialité, cours, échange).
- Si l'original affiche des éléments d'une même liste avec des mises en page différentes, reproduis chacune telle quelle : dans la boucle, le template choisit la mise en page d'un élément d'après ses données (par exemple selon qu'un champ est vide).
- langues : nom et niveau de chaque langue. interets : un centre d'intérêt par élément, affichés joints par \", \" après `labels.interets_label`.
- period : la période telle qu'affichée. `period_style` indique pour chaque section si les périodes affichent le mois (\"month\") ou seulement l'année (\"year\") ; mets \"month\" pour une section absente.
- labels.section_* : titres des sections, tels qu'écrits dans le code source de l'original.
- labels.months_short : les 12 abréviations de mois, de janvier à décembre, dans la langue et le style de l'original (si l'original écrit \"Nov.\", écris \"Janv.\", \"Févr.\", etc.).
- labels.present : le mot qui termine une période en cours dans la langue de l'original (par exemple \"aujourd'hui\"), même s'il n'apparaît pas.";

/// Consignes propres à la lettre : sens des champs du schéma et rendu attendu.
const LM_FIELDS: &str = "Champs de la lettre :
- identity : coordonnées de l'expéditeur. Le template n'affiche que les coordonnées présentes dans l'original, dans le même ordre ; la signature est construite avec prénom et nom.
- recipient : destinataire. Affiche, chacune sur sa ligne et seulement si elle est non vide : company, contact_name, contact_role, location. Les autres lignes du destinataire de l'original (service, adresse postale) n'ont pas de champ et sont supprimées.
- date : la date affichée, déjà formatée. La ligne de date compose la ville de l'expéditeur, une virgule, `labels.date_prefix` s'il n'est pas vide, puis `data.date`. Si l'original calcule la date au lieu de l'écrire, mets une chaîne vide dans example_data.
- subject : le texte de l'objet, sans son préfixe. `labels.subject_prefix` contient le préfixe (par exemple \"Objet :\").
- labels.salutation : la formule d'appel. labels.closing : la formule de politesse finale, en entier.
- paragraphs : les paragraphes du corps, entre la formule d'appel et la formule de politesse finale, rendus par une boucle avec l'espacement de l'original entre deux paragraphes.";

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
             EXAMPLE_DATA PRÉCÉDENT :\n```json\n{}\n```\n\n\
             DROPPED_TEXT PRÉCÉDENT :\n{:?}",
            feedback.previous_template, feedback.previous_example_data, feedback.previous_dropped_text
        ));
    }
    message
}

/// Convertit un document de référence Typst en template + données d'exemple.
///
/// # Arguments
///
/// * `reference_typ` - Source Typst complète de la référence.
/// * `doc_type` - `"cv"` ou `"lettre_motivation"`.
/// * `feedback` - Retour de validation de la tentative précédente, `None` au premier essai.
/// * `model` - Identifiant du modèle Claude à utiliser.
///
/// # Returns
///
/// `TemplatizeOutput` brut, à valider par l'appelant (compilation, pages, texte).
pub async fn templatize_reference(
    reference_typ: &str,
    doc_type: &str,
    feedback: Option<&TemplatizeFeedback>,
    model: &str,
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
         template_typ : le même document, dont toutes les valeurs de contenu sont lues depuis un fichier JSON. \
         Rendu avec les données d'exemple, il doit être visuellement identique à l'original : même texte, \
         même mise en page, même nombre de pages.\n\n\
         example_data : le contenu de l'original rangé dans le schéma JSON imposé. \
         Le template lit exactement ces champs, avec `data` à la racine.\n\n\
         {TEMPLATE_RULES}\n\n{EXAMPLE_DATA_RULES}\n\n{fields}"
    );
    let user_message = build_user_message(reference_typ, feedback);
    let schema = object(&[
        ("template_typ", serde_json::json!({ "type": "string" })),
        ("example_data", data_schema),
        // Chaîne (un texte par ligne) plutôt que tableau : avec un tableau, la
        // grammaire compilée du schéma CV dépasse le plafond de l'API.
        ("dropped_text", serde_json::json!({ "type": "string" })),
    ]);

    let raw = call_claude_with_tokens(&system_prompt, &user_message, Some(schema), TEMPLATIZE_MAX_TOKENS, model).await?;
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
    let dropped_text = parsed
        .get("dropped_text")
        .and_then(|v| v.as_str())
        .map(|text| text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect())
        .unwrap_or_default();
    Ok(TemplatizeOutput { template_typ, example_data, dropped_text })
}
