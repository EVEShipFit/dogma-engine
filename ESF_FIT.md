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

## 1. Premise

A fit is a list of items. Anything the SDE already knows is left unwritten. A
launcher is a high slot because its power attribute says so; a subsystem is a
subsystem because its category says so.

Syntax appears where the SDE cannot answer: how many, what is loaded, what
state an item is in, where it sits when that is not the obvious place, and
where its attributes differ from the type's. Tactical modes are written
explicitly (§6.1).

Three properties follow:

- Racks and slot indices are unwritten in ordinary use.
- Blank lines are insignificant. Grouping is for the reader.
- Line order is insignificant, except for the version and hull lines, and for
  items that take a position - those in the same rack, and fighter squadrons -
  which are placed in the order they appear.

This document defines how esf/1 must be read and written.

## 2. Examples

An ordinary fit. Nothing marks the racks; the grouping is for the reader only.

```
esf/1
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
esf/1
Hecate "Sharpshooter Kite" /Sharpshooter

150mm Light AutoCannon II :Barrage S
150mm Light AutoCannon II :Republic Fleet Fusion S
- @high
Small Energy Nosferatu II

5MN Y-T8 Compact Microwarpdrive +Unstable {speedFactor 524, mass 45.2}
Warp Disruptor II @mid3

Damage Control II !off
Gyrostabilizer II !heat
Damage Control II @cargo          // spare

Zainou 'Gnome' Shield Management SM-703
Standard Blue Pill Booster
```

Drones and fighters. A deployed drone line repeats, so `5x Warrior II` is five
drones in space; a `@bay` line is one stack; a fighter line is one squadron, so
`9x Firbolg II` twice is two squadrons of nine, never one of eighteen.

```
esf/1
Thanatos "Ratting"

Drone Damage Amplifier II
Drone Damage Amplifier II

5x Warrior II
5x Hobgoblin II @bay

9x Firbolg II
9x Firbolg II
6x Dromi II
3x Cyclops II @bay
```

## 3. Tokens

| token | name | meaning |
| --- | --- | --- |
| `Nx` | count | Prefix. How many. |
| `:` | charge | Ammunition or script loaded into this item. |
| `+` | mutaplasmid | Declares the item abyssal. |
| `{ }` | overrides | Attribute values replacing the type's own. |
| `!` | state | `off`, `on`, `heat`. |
| `@` | location | `cargo`, `bay` for drones and fighters, or a rack. An item names a rack only to pin a slot. |
| `"` | quoting | A fit name on the hull line; elsewhere a literal type name. |
| `/` | mode | Hull line only. Tactical mode, by name. |
| `-` | empty slot | Stands in for a type name. Takes a location, and an index pins it. |
| `//` | comment | To end of line. |

## 4. Reading a document

### 4.1 Lex - no SDE required

Split each line on spaces, except inside a `{ }` block or a quoted string,
each of which is scanned to its closing delimiter as one token. A token is
sigil-initial if its first character is one of `:` `+` `{` `!` `@` `"` `/` `-`.
A token beginning with `//` is a comment; one beginning with a single `/` is a
mode.

A first token matching the count pattern is always the count. The type name
runs from the token after it to the next sigil-initial token or end of line.
A bare `-` there is the empty-slot marker, never a name.

The hull line is the first line after the version line, skipping
blank and comment-only lines. It is required. Every other line is an item or
an empty slot. No lookup is needed to tell them apart.

A fit that could be wrong - powergrid, CPU, calibration, hardpoints,
bandwidth, tube count, squadron size, more modules than the rack holds - is
still a valid document.

### 4.2 Resolve - SDE required

Look up each name. Category, group and dogma attributes determine what
the item is and where it belongs (§5). Rack indices are assigned by counting
occurrences. The hull line resolves to a Ship; no other line may.

## 5. Placement

Where an item goes when no `@` token says otherwise.

| resolves as | default placement |
| --- | --- |
| Ship | The hull line, and nowhere else. |
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
the hold rather than a fitted module, and `@bay` applies to drones and
fighters.

An item never names its rack on its own. The SDE already does, so `@low` says
nothing, and the only rack an item writes is a pinned slot (§6.3). An empty
slot has no type to derive a rack from, so it always names one.

A Ship Modifier never gets a line of its own: a tactical mode is a `/` token on
the hull line (§6.1), and a line resolving to one is invalid.

## 6. Semantics

### 6.1 Modes

A tactical mode is written on the hull line as `/name`, and nowhere else. At
most one per document.

The value is a name, resolved against the modes belonging to the hull on the
same line. Any run of consecutive whole words unambiguous among them is
accepted, folded as §6.10 says, so `/sharpshooter`, `/Sharpshooter Mode` and
`/Hecate Sharpshooter Mode` are equivalent. Canonical form writes the shortest
such run, the earliest one on a tie, which for a tactical destroyer is the
mode's own word: `/Sharpshooter`.

### 6.2 Lines and counts

What a count means follows from where the line lands, never from whether an
`@` token was written. A line that falls in cargo by default counts the same as
one that says `@cargo`.

An item that is fitted or deployed is a singleton: one thing, in one place.
`Nx` on such a line is repetition, so `3x 200mm AutoCannon II` is exactly three
lines naming that gun, filling three consecutive slots.

An item that is stored - in cargo or in a bay - is a stack, and `Nx` is the
stack's size. A fighter line is a squadron, and `Nx` is the number of fighters
in it. Neither expands into repeated lines, and two such lines are two stacks
or two squadrons, never one.

Absent means one, except on a fighter line, where it means a full squadron.

A count and a pinned slot are mutually exclusive.

### 6.3 Positions

Slots are numbered from 1: `@low1` is the first low slot, `@low3` the third.

Items in the same rack with no `@` take the next free index in order of
appearance. `@low3` pins. Pinned lines are placed first, then unpinned lines
fill what remains, in order.

An index applies to racks only, and only a pin carries one on an item line.
Fighter squadrons take their tube in line order, and are never pinned.

### 6.4 States

`!off` offline. `!on` online but not running. `!heat` overloaded, which
implies running.

A line with no state token takes a default. A fitted or deployed item defaults
to running when its type has a dogma effect in the active or target category,
and to online otherwise.

A stored item has no state at all: it sits in a hold, it is not fitted. A state
token on a line that lands in cargo or in a bay is invalid, `!heat` included.

### 6.5 Charges

`:` names the charge loaded into the item on its line. No quantity is
recorded. A count on that line counts modules, not charges, and spare
ammunition is a separate cargo line.

### 6.6 Attribute overrides

`{ }` lists attributes as name value pairs separated by commas, using SDE
attribute names. Values are absolute: `speedFactor 524`. Each replaces the
type's base attribute value, and everything the engine computes on top of it
still applies. An attribute that is not listed keeps its base value.

`+name` before the braces declares the item abyssal and names the mutaplasmid
applied to the base type on the same line. It is matched by any run of
consecutive whole words unambiguous among those applicable to that base, folded
as §6.10 says. A mutaplasmid's name repeats the module class already on the
line, so the shortest such run is usually the quality alone: `+Unstable`.

Without `+`, the braces are a plain override and the item remains its own
type.

### 6.7 Empty slots

`- @high` reserves one slot; `3x - @high` reserves three; `- @high4` reserves
that one. `Nx` is repetition here, as on a fitted line, and is exclusive with
an index.

### 6.8 Comments

`//` to end of line. Not part of the fit, and dropped by canonicalisation.

### 6.9 Quoted names

A type name may be written in double quotes: `"Weird/Name II"`. Quoting is
required for a name in which any word begins with a sigil character, or whose
first word matches the count pattern, and is permitted anywhere. A quoted name
contains no double quote.

On the hull line the first quoted string after the hull's own type is the fit
name. Canonical form quotes only where required.

### 6.10 Whitespace and encoding

UTF-8. LF or CRLF. Leading and trailing spaces on a line are insignificant.
Runs of spaces between tokens are one separator.

Names match case-insensitively, by Unicode simple case folding. The fold is
locale-independent, so the same two names match on every machine whatever the
reader's locale: `I` and `i` always match, and never fold to the Turkish
dotless forms.

### 6.11 Version

`esf/N` is the first line of every document, and identifies the format
version. It is required.

This document defines version 1. A reader rejects any `N` it does not
implement, rather than guessing or falling back to version 1. A later version
may change anything here, the grammar included, so nothing below the first line
can be read until the version is known to be supported.

## 7. Grammar

EBNF per XML 1.0 §6 (Notation). Alternatives are tried in the order written.
Terminals are characters, not bytes; §6.10 gives the encoding.

```ebnf
document     ::= version blankLine* hullLine ( entryLine | blankLine )*
version      ::= sp? "esf/" [0-9]+ sp? eol

hullLine     ::= sp? hull  sp? comment? eol
entryLine    ::= sp? entry sp? comment? eol
blankLine    ::= sp?           comment? eol

hull         ::= typeName ( sp fitName )? ( sp mode )?
fitName      ::= quoted
mode         ::= "/" typeName

entry        ::= empty | item
empty        ::= ( count sp )? "-" sp "@" rack
               | "-" sp pinned
item         ::= ( count sp )? typeName modifiers ( sp stored )?
               | typeName modifiers sp pinned
count        ::= [0-9]+ "x"
modifiers    ::= ( sp charge )? ( sp mutation )? ( sp overrides )? ( sp state )?

charge       ::= ":" typeName
mutation     ::= "+" typeName
overrides    ::= "{" sp? override ( sp? "," sp? override )* sp? "}"
override     ::= attrName sp value
attrName     ::= [A-Za-z] [A-Za-z0-9]*
value        ::= "-"? [0-9]+ ( "." [0-9]+ )?
state        ::= "!" ( "off" | "on" | "heat" )
stored       ::= "@" ( "cargo" | "bay" )
pinned       ::= "@" rack [0-9]+
rack         ::= "high" | "mid" | "low" | "rig" | "sub" | "svc"

typeName     ::= quoted | word ( sp word )*

quoted       ::= '"' qchar+ '"'
qchar        ::= space | ( uchar - '"' )

word         ::= wordStart uchar*
wordStart    ::= uchar - sigilChar
sigilChar    ::= ":" | "+" | "{" | "!" | "@" | "/" | '"' | "-"

comment      ::= "//" ( space | uchar )*
eol          ::= ( crlf | lf )+
sp           ::= space+

space        ::= #x20
cr           ::= #xD
lf           ::= #xA
crlf         ::= cr lf
char         ::= [#x0-#xD7FF] | [#xE000-#x10FFFF]  /* any Unicode scalar value */
uchar        ::= char - ( space | cr | lf )
```

A document that does not end in an `eol` is read as if it did.

## 8. Canonical form

For hashing, diffing, URLs and cross-tool comparison. Two documents describe
the same fit if and only if their canonical forms are byte-identical, and
`canonical(canonical(x)) == canonical(x)`.

Canonical form is minimal: it writes only what cannot be derived from the line
itself. In practice it is close to what a person writes by hand.

- `esf/1`, then the hull line - type name, quoted fit name, and `/mode` by its
  shortest unambiguous run of words where the hull has one - then a blank
  line.
- Groups in this order, one blank line between them: subsystems, high, mid,
  low, rig, service, drones, fighters, cargo, implants, boosters. An empty
  group is omitted.
- Within a rack, and among fighters, line order is the position. Every other
  group is sorted by type name folded as §6.10 says, then by the whole
  canonical line compared byte by byte.
- A rack is written in slot order, occupied and empty lines alike, so a line's
  place in that run is its position. No line carries an index.
- An empty-slot line after the last occupied slot in its rack is dropped.
- Adjacent identical lines collapse into one count: fitted items, deployed
  items, and empty slots. Stacks and squadrons never collapse.
- A modifier is written only where it differs from the item's default: a state
  that is not the one §6.4 gives it, a location that is not its default
  placement.
- Type names are the English SDE name at the SDE's casing, quoted only where
  §6.9 requires it.
- Overrides are sorted by attribute name. An abyssal item names its
  mutaplasmid by the shortest unambiguous run of words and writes every
  rollable attribute; a plain override writes only the attributes given.
- Modifiers are written charge, mutaplasmid, overrides, state, location.
- A number is the shortest decimal that reads back as the same value.
- Comments dropped. One space between tokens, LF endings, no trailing
  spaces, one trailing newline.

Canonicalisation is lossless for the fit and lossy for its presentation:
grouping, comments and shorthand belong to the author, the canonical form to
the machine.

## 9. EFT interop

Both directions are normative, so that every tool converts identically.

### 9.1 EFT in

`[Ship, Name]` becomes the hull line. `, Charge` becomes `:Charge`.
`/OFFLINE` becomes `!off`. `Item xN` becomes `Nx Item`. Drone entries become
deployed lines, matching §5. Blank-line grouping is discarded and racks are
re-derived.

`[Empty X slot]` becomes an empty-slot line, by this mapping:

| EFT | esf |
| --- | --- |
| `[Empty High slot]` | `- @high` |
| `[Empty Med slot]` | `- @mid` |
| `[Empty Low slot]` | `- @low` |
| `[Empty Rig slot]` | `- @rig` |
| `[Empty Subsystem slot]` | `- @sub` |
| `[Empty Service slot]` | `- @svc` |

### 9.2 EFT out

EFT carries less than esf. Some constructs are dropped outright, others are
flattened onto what EFT does have.

| construct | EFT output |
| --- | --- |
| `/mode` | dropped |
| `!heat`, `!on` | dropped |
| `+` and `{ }` | base type emitted, overrides dropped |
| `@high3` pins | line order, with `[Empty High slot]` for the gaps |
| `@cargo` spare | emitted in the cargo block |
| deployed vs `@bay` | both emitted in the drone block |
| squadron counts | summed into a fighter total |
| implants, boosters | dropped |
| comments | dropped |
