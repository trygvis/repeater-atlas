---
id: ra-1fef
status: closed
deps: []
links: []
created: 2026-02-14T07:10:56Z
type: task
priority: 2
assignee: Trygve Laugstøl
---

# Create CHIRP CSV export feature

Make a module under service/ called "chirp" with a function called
`chirp_export`. Right now it will take no query arguments and should just export
all repeaters. The function should still be broken up into one part that does
the query and one part that generates the actual CSV. The part that does the
query can be placed in the `export` module directly as it can be shared across
implementations of other exporters.

Use `serde` and make a ChripRepeaterRow type with the fields described below.

Add a `ExportOptions` type to configure the export. This should be considered
generic code. These are its fields:

- "export rx tone": This controls if the repeater's tone should be put into the
  export or not. Many people just want to use their squelch instead.

Before starting:

- Make sure you understand the columns of the format
- Skip any unknown services on a repeater, but note them in the code

This is a snippet describing the CSV format from chirp:

```
"Location":      (int,   "number"),
"Name":          (str,   "name"),
"Frequency":     (chirp_common.parse_freq, "freq"),
"Duplex":        (str,   "duplex"),
"Offset":        (chirp_common.parse_freq, "offset"),
"Tone":          (str,   "tmode"),
"rToneFreq":     (float, "rtone"),
"cToneFreq":     (float, "ctone"),
"DtcsCode":      (int,   "dtcs"),
"DtcsPolarity":  (str,   "dtcs_polarity"),
"RxDtcsCode":    (int,   "rx_dtcs"),
"CrossMode":     (parse_cross_mode, "cross_mode"),
"Mode":          (str,   "mode"),
"TStep":         (float, "tuning_step"),
"Skip":          (str,   "skip"),
"Power":         (chirp_common.parse_power, "power"),
"Comment":       (str,   "comment"),
"URCALL":
"RPT1CALL":
"RPT2CALL":
"DVCODE":
```

## Example CSV files:

US 60 meter channels (Dial).csv:

```
Location,Name,Frequency,Duplex,Offset,Tone,rToneFreq,cToneFreq,DtcsCode,DtcsPolarity,Mode,TStep,Skip,Comment,URCALL,RPT1CALL,RPT2CALL
1,60m CH1,5.330500,,0.600000,,88.5,88.5,023,NN,USB,5.00,,,,,
2,60m CH2,5.346500,,0.600000,,88.5,88.5,023,NN,USB,5.00,,,,,
3,60m CH3,5.357000,,0.600000,,88.5,88.5,023,NN,USB,5.00,,,,,
4,60m CH4,5.371500,,0.600000,,88.5,88.5,023,NN,USB,5.00,,,,,
5,60m CH5,5.403500,,0.600000,,88.5,88.5,023,NN,USB,5.00,,,,,
```

CA FRS and GMRS Channels.csv:

```
Location,Name,Frequency,Duplex,Offset,Tone,rToneFreq,cToneFreq,DtcsCode,DtcsPolarity,Mode,TStep,Skip,Comment,URCALL,RPT1CALL,RPT2CALL
1,FRS 1,462.562500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
2,FRS 2,462.587500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
3,FRS 3,462.612500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
4,FRS 4,462.637500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
5,FRS 5,462.662500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
6,FRS 6,462.687500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
7,FRS 7,462.712500,,5.000000,,88.5,88.5,023,NN,FM,12.50,,,,,
8,FRS 8,467.562500,,5.000000,,88.5,88.5,023,NN,NFM,12.50,,,,,
9,FRS 9,467.587500,,5.000000,,88.5,88.5,023,NN,NFM,12.50,,,,,
```
