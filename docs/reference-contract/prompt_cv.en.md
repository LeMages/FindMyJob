Here is my current CV (attached, whatever its format: PDF, Canva, Word, .typ…). Convert it into a .typ file compatible with FindMyJob, which also passes ATS.

Rules to follow:

1. Keep the layout, colours, fonts and style as close to the original as possible.
2. The file follows the FindMyJob contract below: example values in `example`, data read from `data`, a single `<findmyjob>` metadata, fixed texts in French and English picked by `data.lang`.
3. All information that comes from my profile (identity, contact details, experiences, projects, education, skills, languages) is read from `data`, never hard-coded. The example values are those of my CV.
4. Sections FindMyJob does not know (interests, references, volunteering…) stay as fixed text, in French and English.
5. Section titles are `heading`s, styled with a `#show heading` rule. A line, section or separator whose data is empty disappears, title included.
6. `data.list_marker` precedes every description item (`bullets`, `details`).
7. ATS rules: all text is selectable; no important information only in an image or icon; standard section titles; consistent dates; contact details as plain text; no table for key items.
8. No em or en dash ("—", "–"): use "-".
9. If a font of the original is not available, pick a close one and tell me, so I can import it in FindMyJob (Settings > Fonts).
10. Answer with a single complete .typ file, compiled with Typst 0.15 if possible.

Full contract (also available here: {{DOC_URL}}):

<<<CONTRACT
{{CONTRACT}}
CONTRACT>>>

Compliant example CV (fictional person), to follow for the file structure, not for the content:

<<<EXAMPLE
{{EXAMPLE}}
EXAMPLE>>>
