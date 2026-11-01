# EVEShip.fit's Dogma Engine for Kotlin

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes of the ship, its items and the character.

The engine is written in Rust; this package wraps it for the JVM, so every lookup happens inside Rust.
The JAR carries the native library for Linux and macOS on x86_64 and aarch64, and Windows on x86_64.

## Usage

`sde.dat` and `names.dat` come from [`@eveshipfit/sde`](https://www.npmjs.com/package/@eveshipfit/sde).

```kotlin
import fit.eveship.dogma.*
import java.nio.file.Path

val buildNumber = DogmaEngine.loadSde(Path.of("sde.dat"))

val fit = Fit(
    ship = Ship(587),
    items = listOf(
        FitItem(typeId = 2873, slot = Slot.High(0), state = State.ACTIVE, charge = Charge(185)),
    ),
    character = Character(skills = mapOf(3300 to 5)),
)

val calculation = DogmaEngine.calculate(fit)
// Or if you want to know the source of the effects:
val withSources = DogmaEngine.calculate(fit, Options(sources = true))
// Or if you have a beacon in space (like wormhole effects):
val withBeacon = DogmaEngine.calculate(fit.copy(incoming = DogmaEngine.beacon(beaconTypeId)))
// Or if you have an EFT, the text format EVE copies a fit to the clipboard in (and the other way around):
val imported = DogmaEngine.calculate(DogmaEngine.loadEft("[Rifter, My Rifter]\n200mm AutoCannon I"))
val eft = DogmaEngine.saveEft(fit)
// Or if you have a fitting a character saved in game, as ESI returns it (and the other way around):
val fromEsi = DogmaEngine.loadEsiFitting(fitting)
val toEsi = DogmaEngine.saveEsiFitting(fit)
// Or if you have DNA, the fits EVE links to in chat, with or without its `fitting:` prefix:
val fromDna = DogmaEngine.loadDna(dna)
// EFT, ESI and DNA have no states; with the skills set, this sets the ones EVE gives an imported fit:
val settled = DogmaEngine.postLoad(fromEsi.copy(character = Character(skills = skills)))
// Or if you have a killmail, as ESI returns it, for the fit of the ship that died:
val fromKillmail = DogmaEngine.loadKillmail(killmail)
// Or if you have an EVEShip.fit link, `<version>:<payload>`, with the payload gunzipped and unbase64'd:
val fromLink = DogmaEngine.loadLink(version, payload)
// Or if you have an esf/1 document, as text or as a link (and the other way around):
val fromEsf = DogmaEngine.loadEsf("%esf/1\nRifter\n200mm AutoCannon I\n")
val fromEsfLink = DogmaEngine.loadEsfLink(link)
val esf = DogmaEngine.saveEsf(fit)
val link = DogmaEngine.saveEsfLink(fit)

// What EVE would not let you fly, in `violations` of the calculation:
val validated = DogmaEngine.calculate(fit, Options(validate = true))
```

`loadSde` may only be called once per process.

An EFT import matches the English names in `sde.dat`.
To also match the other languages EVE supports, load `names.dat` as well:

```kotlin
DogmaEngine.loadNames(Path.of("names.dat"))
```

A bad input throws `IllegalArgumentException`; calling before `loadSde` throws `IllegalStateException`.
Every call is thread-safe, so a thread pool calculates fits in parallel.

What every field means is explained under [Input and output](https://github.com/EVEShipFit/dogma-engine#input-and-output).

## Building

You need Rust and [Gradle](https://gradle.org/).
Gradle builds the Rust library for your platform first:

```bash
gradle -p crates/esf-kotlin test jar
```

The tests read the data-files from `node_modules`; run `npm ci` in the root first, or point `ESF_SDE` and `ESF_NAMES` at them.
