# kits

Kit loading and validation. The contract is `schemas/kit.schema.json` (ADR 0002).
`load_kit` = schema → references → conversion to `mnemo_core::kit::KitDef`; the runtime uses the
typed `KitDef`, never the raw manifest.
- New manifest sections: schema first (with an ADR), then reference checks, then a fixture in
  `kits/_test` and a failing case in `kits/_test/invalid`.
