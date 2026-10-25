# Translating Clovercut

Interface translations live in `src/crates/concat/locales/<language-code>.json`.
Copy `en.json`, preserve its keys and placeholders such as `{0}`, and translate
the values. The `_` entry names the language in its own words.

Register a new language in `BUILT_IN` in `src/crates/concat/src/i18n.rs` and run:

```sh
python3 scripts/locales.py --check --strict
```

Missing translations fall back to English. The check validates the string
inventory, compiled language registrations, nonempty strings and exact numbered
placeholders. CI also rejects untranslated English. Identical wording is permitted
for product names, units and the exact language-specific values reviewed in
`scripts/locale-identical.json`; do not add untranslated sentences to that file.
A changed English value needs a new review. Run checker regressions with
`python3 -m unittest discover -s scripts -p test_locales.py`.

Documentation stays in English and does not require a separate README for every
interface language.

To try a translation without rebuilding, place it in
`~/.config/io.github.tulcoturo.clovercut/locales/` and restart the app.
A file for a shipped language overrides only the lines it contains.

Developers should use `I18n.t("area.key")` in Slint and `t`/`tf` in Rust,
write the English value into `en.json`, then run `python3 scripts/locales.py`.
Names inside effect and preset manifests remain source metadata. Standard colour
grading controls are translated at the inspector display boundary using the
`grading.*` keys; their serialized parameter and reset-group identifiers stay
unchanged.
