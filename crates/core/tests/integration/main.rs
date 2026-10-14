//! Binaire unique des tests d'intégration de `findmyjob-core` : chaque
//! fichier de ce dossier est un module, `common/` regroupe les fixtures
//! partagées. Tout nouveau test d'intégration est un module déclaré ici.

mod common;

mod docgen_budgets;
mod docgen_dates;
mod docgen_lint;
mod docgen_prose_guidance;
mod docgen_selection;
mod docgen_sentinel;
mod docgen_text_diff;
mod scraping_contract_type;
mod scraping_fetch_rules;
mod scraping_finalize;
mod scraping_json_ld;
mod scraping_page_text;
mod text_conversion;
