# Export Design

This document describes the general design for export features and the specific
CHIRP CSV export currently implemented.

## Goals

- Provide reusable export plumbing shared across multiple output formats.
- Keep the export code isolated from HTTP and CLI layers so it can be reused.
- Keep per-format logic explicit and documented, especially around field
  mappings.

## General Design

Exports are implemented under `src/service/export/` and split into two pieces:

- **Data loading:** Shared logic that loads full `Repeater` domain models,
  including all services, via `service::repeater_system::load`.
- **Format rendering:** Per-export submodules that convert those models into a
  file format.

The shared data loading is implemented in `src/service/export/mod.rs` and
returns a list of fully populated repeaters. This keeps format implementations
small and focused on mapping.

Exporters accept `ExportOptions`, which is a shared configuration struct meant
to grow as additional formats are added.

## CHIRP CSV Export

Implementation lives at `src/service/export/chirp.rs`.

### Scope

- Only FM services are exported.
- Non-FM services are skipped.
- Location numbers start at zero.

### Reference Files

- [chirp-example-blank.csv](chirp-example-blank.csv): A brand-new CHIRP file
  saved without edits.
- [chirp-example-tones.csv](chirp-example-tones.csv): A matrix of tone
  permutations (CTCSS/DCS/Cross).

### Output Columns

The CSV header mirrors CHIRP’s stock CSV format and includes a superset of
fields:

```
Location,Name,Frequency,Duplex,Offset,Tone,rToneFreq,cToneFreq,DtcsCode,
DtcsPolarity,RxDtcsCode,CrossMode,Mode,TStep,Skip,Power,Comment,URCALL,RPT1CALL,
RPT2CALL,DVCODE
```

Fields not used by the current data model are written as empty strings.

### Field Mapping (General)

- **Name:** `<CALLSIGN> <SERVICE_LABEL>`, trimmed.
- **Frequency:** TX frequency, formatted as MHz with six decimals.
- **Frequency:** The radio's transmit frequency, i.e. the repeater's RX
  frequency (`rx_hz` in the data model), formatted as MHz with six decimals.
- **Duplex/Offset:** Derived from the repeater's TX-RX difference:
  - `Duplex` is `"+"` when repeater TX > repeater RX, `"-"` when TX < RX,
    otherwise empty.
  - `Offset` is `abs(tx_hz - rx_hz)` in MHz with six decimals.
  - Note: `tx_hz` and `rx_hz` are from the repeater's perspective. The CHIRP
    `Frequency` field is the radio operator's transmit frequency (= repeater
    RX), not the repeater's TX.
- **Mode:** `"FM"` for wide and `"NFM"` for narrow.
- **TStep:** `"5.00"` (no per-service step information yet).
- **Comment:** From service note.

### Field Mapping (Tone / CrossMode)

The tone-related columns are populated based on the observed CHIRP exports in
`docs/chirp-example-blank.csv` and `docs/chirp-example-tones.csv`.

Observed behavior (from `chirp-example-tones.csv` row names):

- CHIRP uses `Tone="Tone"` for “Tx CTCSS only”.
- CHIRP uses `Tone="TSQL"` for “Rx CTCSS only”.
- CHIRP uses `Tone="DTCS"` for DCS-only rows (`DCS 074, NN/NR/RN/RR`).
- CHIRP uses `Tone="Cross"` for all “cross” rows, including:
  - CTCSS with mismatched TX/RX frequencies (`CTCSS, 67.0 -> 88.5`).
  - DCS-only TX or RX (`DCS, tx only` / `DCS, rx only`).
  - Mixed CTCSS ↔ DCS (`CTCSS -> DCS`, `DCS -> CTCSS`).
  - DCS ↔ DCS with different codes (`DCS -> DCS, different codes`).

Exporter rules (derived from the examples):

- CHIRP requires non-empty values for `rToneFreq`, `cToneFreq`, `DtcsCode`,
  `RxDtcsCode`, `CrossMode`, and `Power` in every row, even when those fields
  are not in use. Use `88.5` for tone frequency fields, `023` for DTCS code
  fields, `"Tone->Tone"` for `CrossMode`, and `"50W"` for `Power`. Writing empty
  strings causes CHIRP to reject the row with a parse error.

- For tone-related fields not used by a rule below, write the default value
  (`88.5` or `023` as appropriate).

- **CTCSS only**
  - TX only → `Tone="Tone"`, `rToneFreq` set, `cToneFreq` empty.
  - RX only → `Tone="TSQL"`, `cToneFreq` set, `rToneFreq` empty.
  - TX+RX same frequency → `Tone="TSQL"` and set both `rToneFreq` and
    `cToneFreq` to the same value.
  - TX+RX different frequencies → `Tone="Cross"`, `CrossMode="Tone->Tone"`, and
    set both `rToneFreq` and `cToneFreq` (matches `CTCSS, 67.0 -> 88.5`).

- **DCS only**
  - TX only → `Tone="Cross"`, `CrossMode="DTCS->"`, `DtcsCode` set.
  - RX only → `Tone="Cross"`, `CrossMode="->DTCS"`, `RxDtcsCode` set.
  - TX+RX same code → `Tone="DTCS"`, set `DtcsCode`.
  - TX+RX different codes → `Tone="Cross"`, `CrossMode="DTCS->DTCS"`, set both
    `DtcsCode` and `RxDtcsCode`.
  - `DtcsPolarity` is always set (default `NN` in the examples, varied to `NR`,
    `RN`, `RR` in the DCS rows).

- **CTCSS ↔ DCS cross**
  - Set `Tone="Cross"`.
  - Populate `CrossMode` to match direction:
    - TX CTCSS + RX DCS → `CrossMode="Tone->DTCS"`.
    - TX DCS + RX CTCSS → `CrossMode="DTCS->Tone"`.
    - RX only CTCSS → `CrossMode="->Tone"`.
    - TX only CTCSS → `CrossMode="Tone->"`.
  - Populate tone fields for the present sides (`rToneFreq`/`cToneFreq` for
    CTCSS, `DtcsCode`/`RxDtcsCode` for DCS).

### Options

`ExportOptions.export_rx_tone` controls whether RX tone data is exported. When
false, only TX tone is included (or no tone if the service has none).
