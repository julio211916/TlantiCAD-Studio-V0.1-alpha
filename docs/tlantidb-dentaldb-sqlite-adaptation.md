# TlantiDB SQLite Adaptation From DentalDB-Style Case Graph

## Readout

The DentalDB dump is centered on a compact clinical order graph:

- `laboratories`
- `clients` as lab-scoped practices
- `patients` and `technicians`
- `Treatment` as the case/order
- `ToothWork` as one row per tooth indication
- `ToothWorkParameters`, numeric/textual parameter rows, dependencies and work-parameter blobs

That shape is useful because it separates the case header from per-tooth prescriptions. The part we should keep is the graph. The part we should not copy is the exact table vocabulary, composite integer IDs, German/default seed rows, and opaque `WorkParams*` names.

## TlantiCAD Shape

The adaptation landed in `Tauri/sql/002_tlantidb_clinical_order_schema.sql`.

It keeps our existing `cases`, `case_assets`, `odontogram_entries` and `case_modules`, then adds:

- `practices`: our equivalent of lab client/practice records.
- `patients`: optional patient identity attached to a practice.
- `technicians`: lab operators/designers.
- `case_order_context`: case-level order metadata, due date, locks, workload, import and parameter signatures.
- `work_type_catalog`: normalized work definitions aligned with the existing frontend dental work catalog.
- `material_catalog`: local material definitions and shade lists.
- `case_tooth_work`: the durable one-row-per-tooth prescription.
- `tooth_work_parameters`: typed per-tooth parameters for cement gap, thickness, bite-splint settings, screw access, etc.
- `tooth_work_parameter_dependencies`: rules between parameters without cloning DentalDB numeric/textual split tables.
- `work_parameter_blobs` and `work_parameter_versions`: hashed parameter snapshots for imports, generated states and future interop.
- `treatment_custom_fields`: extensible order fields.

## Difference From DentalDB

This is similar in behavior, not identical in schema:

- UUID/text primary keys instead of lab/practice/patient composite integer IDs.
- Existing `cases` remains the canonical case table.
- `case_tooth_work` replaces `ToothWork`.
- One typed `tooth_work_parameters` table replaces `NumericToothWorkParameter` and `TextualToothWorkParameter`.
- `work_parameter_versions` replaces the `WorkParamsInfo*` inheritance-style tables.
- JSON metadata is allowed at boundaries where offline sync and module-specific details are expected.

## Runtime Hook

`Tauri/src/case_repository.rs` now applies both:

- `001_core_schema.sql`
- `002_tlantidb_clinical_order_schema.sql`

So any normal TlantiDB case repository open creates the adapted schema automatically.
