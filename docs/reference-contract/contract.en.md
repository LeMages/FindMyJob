# FindMyJob contract for reference CVs and cover letters (version 1)

FindMyJob generates your CV and cover letter from **your own reference documents**, written in [Typst](https://typst.app). A reference is usable only if it follows this contract. The app checks it at import, deterministically and without any AI call: if a rule is broken, the import is refused and every problem is listed (with the line or field concerned).

Two complete, compliant examples for a fictional student: [`cv_exemple.typ`](cv_exemple.typ) and [`lm_exemple.typ`](lm_exemple.typ). The easiest way is to start from one of them, or to use the app's "Copy the prompt for Claude" button to have your current document converted.

## 1. File shape

A single `.typ` file, UTF-8, at most 512 KiB, containing in this order:

```typst
// 1. Example values, in the exact shape of the data (section 3)
#let example = (
  lang: "fr",
  identity: (prenom: "Antoine", nom: "Dupont", /* … */),
  // …
)

// 2. The data: FindMyJob's, or the example when the file is compiled on its own
#let data = if "data" in sys.inputs { json(bytes(sys.inputs.data)) } else { example }

// 3. The contract metadata (section 2)
#metadata((
  contract: "findmyjob/cv",
  version: 1,
  languages: ("fr", "en"),
  example: example,
  guidance: "General writing instructions.",
  slots: (
    (field: "summary", mode: "adaptable", instruction: "Instruction for this slot."),
  ),
  period: (
    granularity: (experiences: "month", projects: "year", formations: "year"),
    separator: " - ",
    month_style: "short",
  ),
)) <findmyjob>

// 4. Fixed texts, in every language
#let L = (
  fr: (experiences: "Expériences", /* … */),
  en: (experiences: "Experience", /* … */),
).at(data.lang)

// 5. The layout, free, reading only `data` and `L`
```

The file must compile on its own (it then shows its example values). FindMyJob then compiles it with the real data, passed in `sys.inputs.data`.

**Forbidden** (found by syntax analysis, before any compilation):

- `#import` and `#include` (including `@preview/…` packages);
- `read(…)`, `eval(…)`, `plugin(…)`;
- `image("file")`, `json("file")`, `csv(…)`, `yaml(…)`, `toml(…)`, `xml(…)`, `cbor(…)` with a file path;
- `datetime.today` (the letter's date is given in `data.date`).

**Allowed**: an icon or logo pasted into the file, for example `image(bytes("<svg …>…</svg>"))`.

> **Typst pitfall**: a one-element array is written with a trailing comma, `("Python",)`. Without it, `("Python")` is a plain string and the example is refused (`example_invalid`).

## 2. The `<findmyjob>` metadata

Exactly one `#metadata((…)) <findmyjob>` per file, with:

| Field | Content |
|---|---|
| `contract` | `"findmyjob/cv"` for a CV, `"findmyjob/lettre"` for a letter. Must match the imported type. |
| `version` | `1`, the current contract version. |
| `languages` | The languages the file can be shown in. Must contain every language of the app: `("fr", "en")`. |
| `example` | The example values (`example`), in the exact shape of the data (section 3). |
| `guidance` | Optional. General writing instructions (tone, length, what must never be done). |
| `slots` | Optional. The written slots and their instruction (section 4). |
| `period` | CV only: the period format (section 5). Ignored for a letter. |

The example values are the model: length and style of written texts, number of projects and skill groups, number of letter paragraphs, number of pages (the generated document never has more). Categories shown by the example (experiences, projects, education, skills, languages, driving licence, contact details) are flagged when your profile does not fill them.

## 3. The data (`data`)

Every value is a string (`""` when empty, never `none`), except `permis` (boolean) and lists.

### Common to the CV and the letter

| Field | Content |
|---|---|
| `lang` | Document language: `"fr"` or `"en"` (the offer's). Picks the fixed texts: `L = (fr: …, en: …).at(data.lang)`. |
| `identity.prenom`, `identity.nom` | First and last name. |
| `identity.title` | Professional title under the name, written for the offer. |
| `identity.email`, `identity.telephone` | Contact details. |
| `identity.ville`, `identity.zip_code` | City and postal code of the job-search session. |
| `identity.linkedin`, `identity.github`, `identity.portfolio` | Links, as text. |
| `identity.permis` | `true` if you hold a driving licence. |

### CV (`findmyjob/cv`)

| Field | Content |
|---|---|
| `summary` | Profile summary, written for the offer. |
| `skills_groups[]` | Skill groups: `category` (group name) and `skills[]` (skills). |
| `experiences[]` | `poste`, `entreprise`, `lieu`, `period`, `intro`, `bullets[]`. |
| `projects[]` | `titre`, `pour_qui`, `period`, `stack`, `url`, `intro`, `bullets[]`. |
| `formations[]` | `titre`, `ecole`, `period`, `intro`, `details[]`. |
| `langues[]` | `nom` and `level`. |
| `list_marker` | List symbol chosen in the session (`•` or `-`). |

`intro` is the text of a description before its list, without symbol. Each item of `bullets` or `details` may contain a line break (`\n`): show the string as is. `period` is already formatted (section 5).

### Letter (`findmyjob/lettre`)

| Field | Content |
|---|---|
| `recipient.company` | The offer's company. |
| `recipient.extra_lines[]` | Intermediate address lines (department, street…). |
| `recipient.location` | Last address line (city). |
| `date` | Today's date, already written in the language: "13 octobre 2026", "13 October 2026". The place and wording ("Lyon, le") are composed in the file. |
| `subject` | The letter's subject, written for the offer (it contains the exact job title). |
| `paragraphs[]` | The written paragraphs, in order. Their number is the example's. |

The letter's fixed text (salutation, sentences that never change, closing, signature) is written in the file, in every language, around the written paragraphs. One paragraph may mix fixed and written text: `#L.intro #data.paragraphs.at(0)`.

## 4. Written slots (`slots`)

The fields written by the AI are `identity.title` and `summary` for a CV, `subject` and `paragraphs[i]` for a letter (`i` from 0, lower than the example's number of paragraphs). Each slot declares:

| Field | Content |
|---|---|
| `field` | The field concerned (`"summary"`, `"paragraphs[1]"`…). A field can have only one slot. |
| `mode` | `"variable"`: the example text is replaced with information from the offer, following the instruction. `"adaptable"`: the text is written from the profile for the offer, inventing nothing. |
| `instruction` | The instruction for this slot (optional). |

A written field without a slot is written freely from the profile and the offer. The writer also receives the full rendering of your example, to follow its style and avoid repeating the neighbouring fixed parts.

## 5. Periods (CV)

FindMyJob formats `period` itself, from the profile's dates, following `period` in the metadata:

| Field | Content |
|---|---|
| `granularity` | For `experiences`, `projects` and `formations`: `"month"` (month and year) or `"year"` (year only). |
| `separator` | Text between the two dates, for example `" - "` or `" to "`. A "–" or "—" dash is replaced with "-". |
| `month_style` | Optional, `"short"` by default: `"short"` ("Nov. 2025", "Nov 2025"), `"long"` ("novembre 2025", "November 2025") or `"numeric"` ("11.2025"). |
| `numeric_separator` | Optional, `"/"` by default: separator between month and year in `"numeric"` style. |

An ongoing period ends with "présent" / "present"; a period whose two ends are written the same shows only one ("2025").

## 6. Mandatory conventions

1. **No hard-coded data.** Everything that comes from the profile, the session, the offer or the writing is read from `data`. FindMyJob checks it by replacing each example value with a marker: an identity or profile value still visible is refused.
2. **What is empty disappears.** A line, section or separator whose data is empty is not shown. The simplest way: `values.filter(v => v != "").join(" | ")`.
3. **Section titles as `heading`.** Write each section title with `heading(…)` and style it with a `#show heading: …` rule. A title whose section is empty must disappear with it.
4. **List symbol.** In a CV, `data.list_marker` precedes every item of `bullets` and `details` (for example `list(marker: data.list_marker, ..e.bullets)`), and only them: no list symbol is hard-coded. Header separators ("•" between contact details) are not list symbols.
5. **Fixed texts in every language.** Titles, labels, mentions and fixed sentences exist in French and English, picked by `data.lang`. A category FindMyJob does not know (interests, references…) stays a fixed text, in every language.
6. **Number of pages.** In every language, the document rendered with the example must not exceed the example's number of pages.
7. **Letter recipient.** When on the right, it is as wide as its longest line, flush against the right margin, text aligned left: `#align(right, box({ set align(left); … }))`.
8. **No em or en dash** ("—", "–") in the rendering: use "-".

## 7. Fonts

A font must be available at compile time: fonts shipped with Typst (Libertinus Serif, New Computer Modern, DejaVu Sans Mono), fonts installed on the computer, or fonts imported in **Settings > Fonts** (the safest: they follow the app). A missing family is an error, never a silent substitution.

## 8. Errors and warnings

An **error** blocks the import; a **warning** is shown but the import succeeds. All the problems of a file are listed together.

| Code | Type | Explanation |
|---|---|---|
| `not_typ` | Error | The file is not a Typst file (`.typ`). |
| `too_large` | Error | The file exceeds 512 KiB. |
| `not_utf8` | Error | The file is not UTF-8 text. |
| `forbidden_construct` | Error | Forbidden construct (section 1), with its line. |
| `compile_error` | Error | The file does not compile with its example values; Typst's message and the line are given. |
| `missing_font` | Error | A requested font cannot be found (section 7). |
| `metadata_missing` | Error | No `<findmyjob>` metadata. |
| `metadata_duplicated` | Error | Several `<findmyjob>` metadata. |
| `metadata_invalid` | Error | Unreadable metadata: missing, unexpected or wrongly typed field. |
| `wrong_contract` | Error | `contract` does not match the imported type (CV or letter). |
| `outdated_contract_version` | Error | `version` is not the current contract version. |
| `language_missing` | Error | A language of the app is missing from `languages`. |
| `example_invalid` | Error | An example value is missing, unexpected or wrongly typed; the field path is given (`example.experiences[1].period`). |
| `slot_unknown_field` | Error | Slot on a field that is not written or is missing from the example. |
| `slot_invalid_mode` | Error | Slot mode other than `variable` or `adaptable`. |
| `slot_duplicated` | Error | Two slots on the same field. |
| `language_compile_error` | Error | The file does not compile in one of the languages (often a fixed text missing from `L`). |
| `page_count_mismatch` | Error | In one of the languages, the document has more pages than with the example. |
| `empty_data_compile_error` | Error | The file does not compile with empty data (empty lists, empty strings). |
| `hardcoded_value` | Error | An identity or profile example value is hard-coded instead of read from `data`. |
| `list_marker_wrong` | Error | `data.list_marker` does not precede exactly every list item, or a symbol is hard-coded. |
| `orphan_heading` | Error | A section title stays visible while its section is empty. |
| `orphan_separator` | Warning | A separator stays visible when a secondary field is empty; the field is given. |
| `recipient_not_right_aligned` | Warning | The recipient block is not flush against the right margin when it is short. |
| `dash_marks` | Warning | The rendering contains "—" or "–" dashes. |

## 9. Older references

References imported before this contract (`[CONSIGNES]`, `[FIXE]`… markup in comments) are no longer accepted: generation asks you to re-import them. Convert them with the "Copy the prompt for Claude" button.
