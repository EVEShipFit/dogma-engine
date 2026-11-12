# esf/1

A modern plain-text format for EVE Online ship fits.

```
2x 150mm Light AutoCannon II :Barrage S !heat   // spare
│  │                         │          │       │
│  │                         │          │       └─ comment
│  │                         │          └───────── state
│  │                         └──────────────────── charge
│  └────────────────────────────────────────────── type name
└───────────────────────────────────────────────── count

5MN Microwarpdrive II +Unstable {speedFactor 524} @cargo
│                     │         │                 │
│                     │         │                 └─ location
│                     │         └─────────────────── overrides
│                     └───────────────────────────── mutaplasmid
└─────────────────────────────────────────────────── type name
```

Everything after the type name is optional, and appears in the order shown:
charge, mutaplasmid, overrides, state, location, comment.

## Contents

1. [Premise](#1-premise)
2. [Examples](#2-examples)
3. [Tokens](#3-tokens)
4. [Reading a document](#4-reading-a-document)
5. [Placement](#5-placement)
6. [Semantics](#6-semantics)
7. [Grammar](#7-grammar)
8. [Canonical form](#8-canonical-form)
9. [EFT interop](#9-eft-interop)
10. [Binary form](#10-binary-form)

## 1. Premise

A fit is a list of items. Anything the SDE already knows is not written down. A
launcher goes in a high slot because its power attribute says so; a subsystem
is a subsystem because its category says so.

Syntax is only needed where the SDE cannot answer: how many, what is loaded,
what state an item is in, where it sits when that is not the obvious place, and
where its attributes differ from the type's. Tactical modes are also written
out (§6.1).

This means:

- Racks and slot numbers are not written, except to pin a slot or keep one
  empty.
- Blank lines have no meaning. Grouping is only for the reader.
- Line order has no meaning, with two exceptions. The header and hull lines
  come first. Items that take a position are placed in the order they appear;
  these are items in the same rack, and fighter squadrons.

esf/1 describes the fit, and nothing around it. Skills, security status,
wormhole and other environment effects, and projected fits are not part of it.

This document defines how esf/1 must be read and written.

## 2. Examples

An ordinary fit. Nothing marks the racks; the grouping is for the reader only.

```
%esf/1
Rifter "Shield Buffer"

3x 200mm AutoCannon II :Republic Fleet EMP S
Small Energy Nosferatu II

1MN Afterburner II
Warp Scrambler II
Medium Shield Extender II

Damage Control II
Gyrostabilizer II
Overdrive Injector System II

2x Small Projectile Collision Accelerator I
Small Polycarbon Engine Housing I

1000x Republic Fleet EMP S @cargo
```

A fit exercising every module construct at once. `- @high` holds the third
high slot open, so the guns take the first two and the nosferatu the fourth.
`@mid3` holds the second mid slot open (as nothing unpinned is left to fill it).

```
%esf/1
Hecate "Sharpshooter Kite" /Sharpshooter

150mm Light AutoCannon II :Barrage S
150mm Light AutoCannon II :Republic Fleet Fusion S
- @high
Small Energy Nosferatu II

5MN Y-T8 Compact Microwarpdrive +Unstable {cpu 18.4, speedFactor 524}
Warp Disruptor II @mid3

Damage Control II !off
Gyrostabilizer II !heat
Damage Control II @cargo          // spare

Zainou 'Gnome' Shield Management SM-703
Standard Blue Pill Booster
```

Drones and fighters. A deployed drone line repeats, so `5x Warrior II` is five
drones in space; a `@bay` line is one stack; a fighter line is one squadron, so
`6x Firbolg II` twice is two squadrons of six, never one of twelve.

```
%esf/1
Thanatos "Ratting"

Drone Damage Amplifier II
Drone Damage Amplifier II

5x Warrior II
5x Hobgoblin II @bay

6x Firbolg II
6x Firbolg II
3x Dromi II
3x Cyclops II @bay
```

Two fits in one document: a Raven carrying a fitted Heron in its frigate escape
bay. Because the Raven refers to it, the Heron exists only in that bay.

```
%esf/1
Raven "Mission Runner"

4x Cruise Missile Launcher II :Scourge Cruise Missile
Large Shield Extender II

Heron "Scout" @frigate

%esf/1
Heron "Scout"

Core Probe Launcher I :Core Scanner Probe I
Relic Analyzer I
```

A container in the cargo hold. It is its own fit, with the container as its
hull, and referenced like the Heron. What it holds is its cargo.

```
%esf/1
Rifter "Roamer"

3x 200mm AutoCannon II :Republic Fleet EMP S
1MN Afterburner II

Small Secure Container "Spares"

%esf/1
Small Secure Container "Spares"

1000x Republic Fleet EMP S
Damage Control II
```

## 3. Tokens

| token | name | meaning |
| --- | --- | --- |
| `Nx` | count | Written before the type name. How many there are. |
| `:` | charge | Ammunition or script loaded into this item, optionally with a count. |
| `+` | mutaplasmid | Declares the item abyssal. |
| `{ }` | overrides | Attribute values replacing the type's own. |
| `!` | state | `off`, `on`, `heat`. |
| `@` | location | `cargo`, `bay` for drones and fighters, another hold (§5.1), or a rack. An item names a rack only to pin a slot. |
| `"` | quoting | A fit name after a type name; otherwise a literal type name. |
| `/` | mode | The tactical mode, by name. Only on the hull line. |
| `-` | empty slot | Stands in for a type name. Takes a location, and an index pins it. On the hull line, it means no ship. |
| `//` | comment | Runs to the end of the line. |

## 4. Reading a document

### 4.1 Lex - no SDE required

Split each line on spaces, except inside a `{ }` block or a quoted string,
each of which is scanned to its closing delimiter as one token. Inside a
quoted string, `""` is a literal `"`, not the closing delimiter.

The sigils are the characters that mark a token as something other than part of
the type name: `:` `+` `{` `!` `@` `"` `/`. A token that starts with a sigil is
a charge, mutaplasmid, overrides, state, location, quoted string, mode or
comment (§3). A token beginning with `//` is a comment; one beginning with a
single `/` is a mode.

A first token matching the count pattern is always the count. The type name
runs from the token after it up to the next token that starts with a sigil, or
the end of the line.
A bare `-` there is the empty-slot marker, never a name. Past the first token,
`-` is part of the name: `Legion Defensive - Covert Reconfiguration`.

A line whose first token begins with `%` is a header, and starts a block
(§6.11). `%esf/1` starts a fit, and a document holds one or more fits (§6.12).
The hull line is the first line after the header, skipping blank and
comment-only lines. It is required. Every other line is an item, a reference
or an empty slot. No lookup is needed to tell them apart.

A fit can break the game's rules and still be a valid document. This includes
going over powergrid, CPU, calibration, hardpoints, bandwidth, tube count or
squadron size, more modules than the rack holds, more charges than the item
holds, and a hold the hull does not have. It also includes any other fitting
rule, such as ship restrictions, max-per-ship limits, rig size and subsystem
sets.

### 4.2 Resolve - SDE required

Look up each name. Names are English SDE names; other languages are not
matched. Where a name is shared, a published type wins over an unpublished
one, then the lowest type ID wins. Category, group and dogma attributes
determine what the item is and where it belongs (§5). Rack indices are assigned
by counting occurrences. The hull line resolves to a Ship, Structure or
container, or is `-`.

## 5. Placement

This is where an item goes when no `@` token says otherwise.

| resolves as | default placement |
| --- | --- |
| Ship, Structure, container | The hull line. Elsewhere, cargo. |
| Subsystem | Subsystem rack, next free index. |
| Rig | Rig rack, next free index. |
| Service module | Service rack, next free index. |
| Module | The rack named by its power attribute, next free index. |
| Drone | Deployed. |
| Fighter | One squadron, in the next free tube. |
| Charge | Attached by `:`, otherwise cargo. |
| Implant, Booster | Plugged in. |
| Ship Modifier | The hull line's `/mode`, and nowhere else. |
| anything else | Cargo. |

An `@` token overrides the default. `Damage Control II @cargo` is a spare in
the hold rather than a fitted module, `@bay` applies to drones and fighters,
and `@fuel` puts an item in the fuel bay.

An item names its rack only to pin a slot (§6.3). An empty slot always names
its rack.

### 5.1 Holds

Besides `cargo` and `bay`, a location names one of these holds. Nothing is
placed in them by default.

| location | hold | SDE attribute |
| --- | --- | --- |
| `ammo` | Ammo Hold | `specialAmmoHoldCapacity` |
| `booster` | Booster Hold | `specialBoosterHoldCapacity` |
| `command` | Command Center Hold | `specialCommandCenterHoldCapacity` |
| `corpse` | Corpse Hold | `specialCorpseHoldCapacity` |
| `depot` | Mobile Depot Hold | `specialMobileDepotHoldCapacity` |
| `expedition` | Expedition Hold | `specialExpeditionHoldCapacity` |
| `fleet` | Fleet Hangar | `fleetHangarCapacity` |
| `frigate` | Frigate Escape Bay | `frigateEscapeBayCapacity` |
| `fuel` | Fuel Bay | `specialFuelBayCapacity` |
| `gas` | Gas Hold | `specialGasHoldCapacity` |
| `ice` | Ice Hold | `specialIceHoldCapacity` |
| `infrastructure` | Infrastructure Hold | `specialColonyResourcesHoldCapacity` |
| `maintenance` | Ship Maintenance Bay | `shipMaintenanceBayCapacity` |
| `mineral` | Mineral Hold | `specialMineralHoldCapacity` |
| `mining` | Mining Hold | `generalMiningHoldCapacity` |
| `moon` | Moon Material Output Bay | `outputMoonMaterialBayCapacity` |
| `planetary` | Planetary Commodities Hold | `specialPlanetaryCommoditiesHoldCapacity` |
| `quafe` | Quafe Hold | `specialQuafeHoldCapacity` |
| `subsystem` | Subsystem Hold | `specialSubsystemHoldCapacity` |

## 6. Semantics

### 6.1 Tactical modes

Some hulls, such as Tactical Destroyers, are always in one of their tactical
modes. The SDE has each mode as its own unpublished type, in the Ship Modifiers
group, one set per hull.

A tactical mode is written on the hull line as `/name`, and nowhere else. A fit
has at most one.

The name is looked up among the modes of the hull on the same line, ignoring
case (§6.10). A mode's name starts with the hull's name, and you may leave that
out. You may also drop whole words from the end, as long as only one mode still
matches. So `/sharpshooter`, `/Sharpshooter Mode` and
`/Hecate Sharpshooter Mode` all mean the same mode. Canonical form leaves out
the hull's name and writes the fewest words that still match one mode:
`/Sharpshooter`.

### 6.2 Lines and counts

What a count means depends on where the item ends up (§5).

On a fitted or deployed item, `Nx` is repetition: `3x 200mm AutoCannon II` is
three lines naming that gun.

An item that is stored - in cargo, a bay or a hold - is a stack, and `Nx` is
the stack's size. A fighter line is a squadron, and `Nx` is the number of
fighters in it. Neither expands into repeated lines, and two such lines are two
stacks or two squadrons, never one.

Absent means one, except on a fighter in a tube, where it means a full squadron.
A count, also one after `:`, is at least 1 and at most 4294967295, the
largest the binary form holds (§10). That holds too for the count that lines
collapse into in canonical form (§8).

An implant or booster that is plugged in takes no count, and a fit plugs in
each type at most once.

A count and a pinned slot are mutually exclusive.

### 6.3 Positions

Slots are numbered from 1 to 4294967295: `@low1` is the first low slot, `@low3`
the third.

Items in the same rack with no `@` take the next free index in order of
appearance. `@low3` pins. Pinned lines are placed first, then unpinned lines
fill what remains, in order.

Fighter squadrons take their tube in line order, and are never pinned.

### 6.4 States

`!off` means the item is offline. `!on` means it is online but not running.
`!heat` means it is overloaded, which also means running.

A line with no state token takes a default. A fitted or deployed item defaults
to running when its type has a dogma effect in the active or target category,
and to online otherwise. The `online` effect counts as being in the online
category, whatever category the SDE gives it.

An implant or booster defaults to online, and takes `!off` only.

A stored item has no state, and a state token on it is invalid.

### 6.5 Charges

`:` names the charge loaded into the item on its line, and must resolve to a
type in the Charge category. A count after the `:`
is how many are loaded: `Bomb Launcher I :1x Void Bomb`. Without one, the item
is fully loaded: its capacity divided exactly by the charge's volume, rounded
down.

A count at the start of the line counts modules, not charges:
`3x Light Missile Launcher II :7x Scourge Light Missile` is three launchers
holding seven missiles each. Spare ammunition is a separate cargo line.

### 6.6 Attribute overrides

`{ }` lists attributes as name value pairs separated by commas, using SDE
attribute names. Where a name is shared, a published attribute wins over an
unpublished one, then the lowest attribute ID wins. Values are absolute:
`speedFactor 524`, and must fit in a 64-bit float. Each replaces the
type's base attribute value, and everything the engine computes on top of it
still applies. An attribute that is not listed keeps its base value. An
attribute is listed at most once.

`+name` before the braces declares the item abyssal and names the mutaplasmid
applied to the base type on the same line. The name is looked up among the
mutaplasmids that apply to that base, ignoring case (§6.10). You may drop whole
words from the end, as long as only one mutaplasmid still matches. So
`+Unstable` and `+Unstable 5MN Microwarpdrive Mutaplasmid` mean the same.

Without `+`, the braces are a plain override and the item remains its own
type.

### 6.7 Empty slots

`- @high` keeps one high slot empty; `3x - @high` keeps three empty;
`- @high4` keeps the fourth high slot empty.

### 6.8 Comments

A comment starts with `//` and runs to the end of the line. It is not part of
the fit, and canonical form drops it.

### 6.9 Quoted names

A type name may be written in double quotes: `"Weird/Name II"`. Quoting is
required for a name whose first word begins with `-` or `%`, or in which any
word starts with a sigil (§4.1). Quoting is allowed on any name. Inside quotes, a
double quote is written twice: `"GDN-9 ""Nightstalker"" Combat Goggles"`.

A quoted string after the type name is a fit name, on the hull line and in a
reference (§6.12). Canonical form quotes only where required.

### 6.10 Whitespace and encoding

A document is UTF-8. Lines end in LF or CRLF. Spaces at the start and end of a
line are ignored. Several spaces between tokens count as one.

Names match case-insensitively, by locale-independent Unicode simple case
folding.

### 6.11 Blocks

A document is a list of blocks. A header starts a block, which runs to the next
header or the end of the document. A header is `%`, a lowercase name, `/` and
a version: `%esf/1`. A document starts with a header.

`%esf/N` starts a fit. This document defines version 1. A reader rejects any
`N` it does not implement.

A reader skips a block whose name it does not know. This lets other formats,
such as a fleet, share a document with fits.

### 6.12 Multiple fits

A document holds one or more fits, each starting with its own header.
Two documents joined together are one valid document.

A container is a type in the Cargo Container, Secure Cargo Container, Audit
Log Secure Container or Freight Container group.

A Ship, Structure or container line followed by a quoted fit name is a
reference: it places that fit where the line ends up (§5).
`Heron "Scout" @frigate` puts the Heron fit named `Scout` in the frigate escape
bay. It matches the fit in the same document with that hull and that fit name,
ignoring case (§6.10). It is invalid unless exactly one fit matches. A reference takes a count and a
location, and nothing else.

A fit that is referenced exists only where it is referenced, once per
reference and count. A fit that is not referenced stands on its own. No fit
may contain itself, directly or through another fit.

A Ship or Structure line without a fit name is an unfitted hull, and a
container line without one is an empty container.

A hull line of `-` is a fit without a ship: a plain list of items, such as a
contract. Every line in it is stored in cargo, and any other location or an
empty slot is invalid. It may have a fit name, but no mode.

A hull line naming a container is read the same way, and its cargo is what the
container holds.

## 7. Grammar

The grammar is in [grammar.ebnf](grammar.ebnf).

It uses the EBNF notation from XML 1.0, §6. Alternatives are tried in the order written.
Terminals are characters, not bytes; §6.10 gives the encoding.

A document that does not end in an `eol` is read as if it did.

## 8. Canonical form

Canonical form is used for hashing, diffing, URLs, and comparing fits between
tools. Two documents describe
the same fit if and only if their canonical forms are byte-identical, and
`canonical(canonical(x)) == canonical(x)`.

Canonical form is minimal: it writes only what cannot be derived from the line
itself. In practice it is close to what a person writes by hand.

- Fits keep their order in the document, with one blank line between them.
  Other blocks are dropped. Each fit is written as follows.
- `%esf/1`, then the hull line - type name, quoted fit name, and `/mode` as
  §6.1 writes it where the hull has one - then a blank line, unless the fit
  has no other lines.
- Groups in this order, one blank line between them: subsystems, high, mid,
  low, rig, service, drones, fighters, cargo, the holds of §5.1 in the order
  listed, implants, boosters. An empty group is omitted. Drones in the bay
  belong to the drones group, and fighters in the bay to the fighters group.
- Within a rack, and among fighters in tubes, line order is the position.
  Fighters in the bay follow them, and drones in the bay follow the other
  drones. Every other line is sorted, among the lines of its part, by type
  name ignoring case (§6.10), then by the whole canonical line compared byte
  by byte.
- A rack is written in slot order, occupied and empty lines alike, so a line's
  place in that run is its position. No line carries an index.
- An empty-slot line after the last occupied slot in its rack is dropped.
- After sorting, adjacent identical lines collapse into one count: fitted
  items, deployed items, and empty slots. Stacks and squadrons never
  collapse.
- A modifier is written only where it differs from the item's default: a state
  that is not the one §6.4 gives it, a location that is not its default
  placement, a charge count where the item is not fully loaded, a fighter
  count where the squadron is not full.
- Type names are the English SDE name at the SDE's casing, quoted only where
  §6.9 requires it. A reference writes the fit name as the referenced hull
  line does.
- Overrides are sorted by attribute name ignoring case (§6.10). An abyssal
  item names its mutaplasmid with the fewest words that still match one
  mutaplasmid (§6.6), and writes every rollable attribute, plus any other
  attribute given. A plain override writes only the attributes given.
- Modifiers are written charge, mutaplasmid, overrides, state, location.
- A number has the fewest significant digits that read back as the same
  64-bit float, the closest to it where several do, and no exponent. Negative
  zero is written as `0`.
- Comments are dropped. Tokens are separated by one space, lines end in LF,
  with no trailing spaces, and the document ends with one newline.

Canonicalisation keeps the fit, and drops its presentation: grouping,
comments and shorthand.

## 9. EFT interop

EFT is a convention, not a format: there is no spec, and each tool reads and
writes its own variant. esf/1 aims to hold any fit an EFT text can describe.
Going the other way, what carries over depends on the variant a tool writes.

## 10. Binary form

The binary form is a compact way to store a fit, or to put it in a URL. It is
the canonical form, with SDE IDs in place of names, encoded as Protocol
Buffers. The schema is in [esf.proto](esf.proto), and a binary document is one
`Document` message.

Each block type (§6.11) has its own field in `Document`, and each such field is
a list of messages. Because of that, the first byte of a binary document is
never `%` or a space, while a text document always starts with one of them. A
reader can tell the two apart by that first byte.

### 10.1 Mapping

Each `%esf/1` block becomes a `Fit` in `Document.fits`. Each line after its
hull line becomes an `Entry`, in the same order.

| esf | binary |
| --- | --- |
| `%esf/1` | `Fit.version`, `1` |
| hull type name | `Fit.hull`, its type ID; absent for `-` |
| fit name | `Fit.name`, unquoted; empty for none |
| `/mode` | `Fit.tactical_mode`, its type ID (§6.1) |
| type name | `Entry.type`, its type ID; absent for `-` |
| `Nx` | `Entry.count` |
| reference fit name | `Entry.fit_name`, unquoted |
| `:` | `Entry.charge`, its type ID, and `Entry.charge_count` |
| `+` | `Entry.mutaplasmid`, its type ID |
| `{ }` | `Entry.override_attributes`, attribute IDs, and `Entry.override_values`, in the same order |
| `!` | `Entry.state` |
| `@` | `Entry.location` |

A field marked `optional` is only present when the text has it, and `0` is
then a real value. Any other field is left out when it is zero or empty. An
override value is stored as the 64-bit float closest to the decimal in the
text. Negative zero is stored as `0`.

Comments and blank lines are not stored. Neither are pinned slots: canonical
form writes a rack in slot order, so the order of the entries already gives
each slot.

### 10.2 Reading

A binary document is read as the text document it maps to (§4). It is invalid
if that text is invalid. It is also invalid if:

- it is empty;
- an ID is not the one its name resolves to in text (§4.2, §6.1, §6.6);
- a name contains CR or LF;
- `Entry.charge_count` is set without `Entry.charge`;
- the two override fields have different lengths, or a value is not finite;
- a field or enum value is not in the schema, except a field of `Document`.

A reader skips a field of `Document` it does not know, just as it skips an
unknown block in text. It rejects any `Fit.version` it does not implement
(§6.11). Two binary documents joined together are one valid document, as in
§6.12.

### 10.3 Writing

A writer encodes the canonical form, keeping its order of fits, entries and
overrides. Fields are written in field number order, and only when present. A
field that is not a list is written at most once. Each list of messages is
written as one record per message, and each list of numbers as one packed
record. Varints take their shortest form.

### 10.4 In text

In URLs and other text, the binary form is written as base64url without padding
(RFC 4648 §5).
