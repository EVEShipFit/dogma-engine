# EVEShip.fit's Dogma Engine for .NET

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes of the ship, its items and the character.

The engine is written in Rust; this package wraps it for .NET 8 and up, so every lookup happens inside Rust.
The package carries the native library for Linux and macOS on x64 and arm64, and Windows on x64.

## Install

```bash
dotnet add package EveShipFit.DogmaEngine
```

## Usage

`sde.dat` and `names.dat` come from [`@eveshipfit/sde`](https://www.npmjs.com/package/@eveshipfit/sde).

```csharp
using EveShipFit.Dogma;

var buildNumber = DogmaEngine.LoadSde("sde.dat");

var fit = new Fit
{
    Ship = new Ship(587),
    Items = [new FitItem(2873, new Slot.High(0), State.Active, Charge: new Charge(185))],
    Character = new Character { Skills = new Dictionary<int, int> { [3300] = 5 } },
};

var calculation = DogmaEngine.Calculate(fit);
// Or if you want to know the source of the effects:
var withSources = DogmaEngine.Calculate(fit, new Options(Sources: true));
// Or if you have a beacon in space (like wormhole effects):
var withBeacon = DogmaEngine.Calculate(fit with { Incoming = DogmaEngine.Beacon(beaconTypeId) });
// Or if you have an EFT, the text format EVE copies a fit to the clipboard in (and the other way around):
var imported = DogmaEngine.Calculate(DogmaEngine.LoadEft("[Rifter, My Rifter]\n200mm AutoCannon I"));
var eft = DogmaEngine.SaveEft(fit);
// Or if you have a fitting a character saved in game, as ESI returns it (and the other way around):
var fromEsi = DogmaEngine.LoadEsiFitting(fitting);
var toEsi = DogmaEngine.SaveEsiFitting(fit);
// Or if you have DNA, the fits EVE links to in chat, with or without its `fitting:` prefix:
var fromDna = DogmaEngine.LoadDna(dna);
// EFT, ESI and DNA have no states; with the skills set, this sets the ones EVE gives an imported fit:
var settled = DogmaEngine.PostLoad(fromEsi with { Character = new Character { Skills = skills } });
// Or if you have a killmail, as ESI returns it, for the fit of the ship that died:
var fromKillmail = DogmaEngine.LoadKillmail(killmail);
// Or if you have an EVEShip.fit link, `<version>:<payload>`, with the payload gunzipped and unbase64'd:
var fromLink = DogmaEngine.LoadLink(version, payload);
// Or if you have an esf/1 document, as text or as a link (and the other way around):
var fromEsf = DogmaEngine.LoadEsf("%esf/1\nRifter\n200mm AutoCannon I\n");
var fromEsfLink = DogmaEngine.LoadEsfLink(link);
var esf = DogmaEngine.SaveEsf(fit);
var esfLink = DogmaEngine.SaveEsfLink(fit);

// What EVE would not let you fly, in `Violations` of the calculation:
var validated = DogmaEngine.Calculate(fit, new Options(Validate: true));
```

`LoadSde` may only be called once per process.

An EFT import matches the English names in `sde.dat`.
To also match the other languages EVE supports, load `names.dat` as well:

```csharp
DogmaEngine.LoadNames("names.dat");
```

A bad input throws `ArgumentException`; calling before `LoadSde` throws `InvalidOperationException`.
Every call is thread-safe, so a thread pool calculates fits in parallel.

The types are records, but as in any C# record, a list or dictionary in one compares by reference, not by its contents.

What every field means is explained under [Input and output](https://github.com/EVEShipFit/dogma-engine#input-and-output).

## Building

You need Rust and the [.NET SDK](https://dotnet.microsoft.com/download).
The build compiles the Rust library for your platform first:

```bash
dotnet test crates/esf-dotnet/tests
```

The tests read the data-files from `node_modules`; run `npm ci` in the root first, or point `ESF_SDE` and `ESF_NAMES` at them.
