# Reference markup format

A reference (CV or cover letter, `.typ` file) can carry writing instructions in Typst line comments. They are read by the app (`src-tauri/src/documents/markup/`), deterministically, when the reference is imported: no AI interprets them. Comments are never rendered, so the reference still compiles and looks the same.

User-facing summary (shown in the app, in French) is in `src/components/ReferenceMarkupHelp.vue`; this file is the full specification.

## Tags

| Tag | Meaning at generation |
|---|---|
| `// [CONSIGNES]` | General instructions for the whole document. |
| `// [FIXE] instruction` | The zone is copied word for word (translated faithfully if the document is generated in another language). Only profile information inside it (city, name…) is updated. |
| `// [VARIABLE] instruction` | The zone is rewritten from the job offer. |
| `// [ADAPTABLE] instruction` | The zone is rewritten from the profile, chosen and ordered for the offer, without inventing anything. |

- A tag opens a `//` line comment: `// [FIXE] …`. Case is ignored (`[fixe]` works). A tag written elsewhere in a comment (`// --- Body --- [FIXE]`) or in a `/* */` comment is an error.
- The text after the tag, and the `//` lines that directly follow it (no blank line, no code in between), form its instruction. The instruction is optional.

## General instructions

```typst
// [CONSIGNES]
// - The letter fits on one page.
// - The recruiter contact is never named.
```

The block is the `[CONSIGNES]` line plus every `//` line that follows, up to the first line that is not a comment (blank line or code). One block per reference.

## Zones

A zone covers the visible text that follows its tag, up to the next tag or the end of the paragraph (blank line). When the tag precedes a literal or a `[...]` block (`text(...)[...]`, `#text(weight: "bold")[...]`), the zone stops at the end of that literal or block.

```typst
// [VARIABLE] Exact job title from the offer.
#text(weight: "bold")[Objet : Candidature pour le poste de Stagiaire Data Engineer]

// [FIXE] The first two sentences.
Actuellement étudiant en dernière année… mon école.
// [VARIABLE] Job title and company, from the offer.
C'est dans ce cadre que je vous adresse ma candidature…
```

- Several tags can split one paragraph: the two zones above belong to the same paragraph.
- A zone is attached automatically to the written field that contains its text: subject and body paragraphs of a letter, title (line under the name) and summary of a CV. A label on the same line (`Objet :`) is ignored.
- Written text before the first tag of a paragraph, or a paragraph with no tag, is an implicit `[VARIABLE]` zone.
- `[FIXE]` on a text that is not written prose (greeting, closing, a hard-coded line) has no effect: that text is already reproduced as is.

## Errors

Reported at import with their line, before any AI call when possible:

- a tag that does not open its comment, or a tag inside `/* */`;
- a second `[CONSIGNES]` block;
- a tag followed by no visible text before the next blank line;
- a `[VARIABLE]` or `[ADAPTABLE]` zone whose text is not written prose (recipient address, contact details…): only `[FIXE]` can stand there;
- two zones covering the same text.

## Hard-coded text

Visible text that is neither profile data, nor a label, nor written prose (e.g. « Références disponibles sur demande », « P.J. : CV ») needs no tag: the app stores it as a reference text and translates it when the document is generated in another language. Official names (job title, company, degree, school, project title) are never translated.
