Here is my current cover letter (attached, whatever its format: PDF, Canva, Word, .typ…). Convert it into a .typ file compatible with FindMyJob, which also passes ATS.

Rules to follow:

1. Keep the layout, colours, fonts and style as close to the original as possible.
2. The file follows the FindMyJob contract below: example values in `example`, data read from `data`, a single `<findmyjob>` metadata, fixed texts in French and English picked by `data.lang`.
3. My identity, contact details, the recipient company, the date and the subject are read from `data`, never hard-coded. The example values are those of my letter.
4. Paragraphs that change from one application to the next become written slots: one item of `data.paragraphs` each, with a `variable` slot (from the offer) or an `adaptable` slot (from my profile) and an instruction. Their example text gives the expected style and length.
5. Sentences that never change (salutation, fixed introduction, closing…) stay as fixed text, in French and English, placed around the written paragraphs. A paragraph may mix a fixed sentence and a written slot.
6. My general instructions (tone, length, what must never be written) go in `guidance`.
7. The recipient block, when on the right, is as wide as its longest line: `#align(right, box({ set align(left); … }))`. An empty line disappears.
8. ATS rules: all text is selectable; no important information only in an image or icon; contact details as plain text; no table for key items.
9. No em or en dash ("—", "–"): use "-".
10. If a font of the original is not available, pick a close one and tell me, so I can import it in FindMyJob (Settings > Fonts).
11. Answer with a single complete .typ file, compiled with Typst 0.15 if possible.

Full contract (also available here: {{DOC_URL}}):

<<<CONTRACT
{{CONTRACT}}
CONTRACT>>>

Compliant example cover letter (fictional person), to follow for the file structure, not for the content:

<<<EXAMPLE
{{EXAMPLE}}
EXAMPLE>>>
