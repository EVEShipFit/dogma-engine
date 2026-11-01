using System.Text.Json;
using System.Text.Json.Serialization;

namespace EveShipFit.Dogma;

/// <summary>The state of an item, lowest first.</summary>
public enum State { Offline, Online, Active, Overload }

/// <summary>The security of a solar system.</summary>
public enum Security { HighSec, LowSec, NullSec, Wormhole }

/// <summary>Where an item is; <c>Index</c> starts at 0, but for implants and boosters is the slot as EVE numbers it.</summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(High), "high")]
[JsonDerivedType(typeof(Medium), "medium")]
[JsonDerivedType(typeof(Low), "low")]
[JsonDerivedType(typeof(Rig), "rig")]
[JsonDerivedType(typeof(Subsystem), "subsystem")]
[JsonDerivedType(typeof(Service), "service")]
[JsonDerivedType(typeof(FighterTube), "fighter_tube")]
[JsonDerivedType(typeof(Implant), "implant")]
[JsonDerivedType(typeof(Booster), "booster")]
[JsonDerivedType(typeof(FighterBay), "fighter_bay")]
[JsonDerivedType(typeof(DroneBay), "drone_bay")]
[JsonDerivedType(typeof(Cargo), "cargo")]
public abstract record Slot
{
    private Slot() { }

    public sealed record High(int Index) : Slot;
    public sealed record Medium(int Index) : Slot;
    public sealed record Low(int Index) : Slot;
    public sealed record Rig(int Index) : Slot;
    public sealed record Subsystem(int Index) : Slot;
    public sealed record Service(int Index) : Slot;
    public sealed record FighterTube(int Index) : Slot;
    public sealed record Implant(int Index) : Slot;
    public sealed record Booster(int Index) : Slot;
    public sealed record FighterBay : Slot;
    public sealed record DroneBay : Slot;
    public sealed record Cargo : Slot;
}

/// <summary>A charge loaded in a module.</summary>
public sealed record Charge(int TypeId);

/// <summary>How a module or drone was mutated.</summary>
public sealed record Mutation(int Base, IReadOnlyDictionary<int, double> Attributes);

/// <summary>How far a module has spooled; 0.0 is unspooled, 2.125 is +212.5%.</summary>
public sealed record Spool(double MultiplierBonus);

/// <summary>A module, drone, fighter squadron, implant, booster or item in cargo.</summary>
public sealed record FitItem(
    int TypeId,
    Slot Slot,
    State State,
    int Quantity = 1,
    Charge? Charge = null,
    Mutation? Mutation = null,
    IReadOnlyList<int>? FighterAbilities = null,
    Spool? Spool = null)
{
    public IReadOnlyList<int> BoosterSideEffects { get; init; } = [];
}

/// <summary>The ship of a fit.</summary>
public sealed record Ship(int TypeId, int? Mode = null);

/// <summary>The character flying the ship.</summary>
public sealed record Character
{
    public IReadOnlyDictionary<int, int> Skills { get; init; } = new Dictionary<int, int>();
    public double SecurityStatus { get; init; }
}

/// <summary>Incoming damage, relative to each other.</summary>
public sealed record DamageProfile(double Em = 0, double Explosive = 0, double Kinetic = 0, double Thermal = 0);

/// <summary>What a Reactive Armor Hardener shifts its resistances towards.</summary>
[JsonConverter(typeof(ReactiveArmorConverter))]
public abstract record ReactiveArmor
{
    private ReactiveArmor() { }

    public sealed record DoNotAdapt : ReactiveArmor;
    public sealed record DamageProfile : ReactiveArmor;
    public sealed record Profile(Dogma.DamageProfile Damage) : ReactiveArmor;
}

internal sealed class ReactiveArmorConverter : JsonConverter<ReactiveArmor>
{
    public override ReactiveArmor Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType == JsonTokenType.StartObject)
        {
            var profile = JsonSerializer.Deserialize<Dictionary<string, DamageProfile>>(ref reader, options)!;
            return new ReactiveArmor.Profile(profile["profile"]);
        }

        return reader.GetString() switch
        {
            "do_not_adapt" => new ReactiveArmor.DoNotAdapt(),
            "damage_profile" => new ReactiveArmor.DamageProfile(),
            var name => throw new JsonException($"unknown reactive armor: {name}"),
        };
    }

    public override void Write(Utf8JsonWriter writer, ReactiveArmor value, JsonSerializerOptions options)
    {
        switch (value)
        {
            case ReactiveArmor.DoNotAdapt:
                writer.WriteStringValue("do_not_adapt");
                break;
            case ReactiveArmor.DamageProfile:
                writer.WriteStringValue("damage_profile");
                break;
            case ReactiveArmor.Profile profile:
                writer.WriteStartObject();
                writer.WritePropertyName("profile");
                JsonSerializer.Serialize(writer, profile.Damage, options);
                writer.WriteEndObject();
                break;
        }
    }
}

/// <summary>Where the ship is.</summary>
public sealed record FitEnvironment
{
    public DamageProfile DamageProfile { get; init; } = new(0.25, 0.25, 0.25, 0.25);
    public Security Security { get; init; } = Security.HighSec;
    public ReactiveArmor ReactiveArmor { get; init; } = new ReactiveArmor.DoNotAdapt();
}

/// <summary>A buff, as <c>dbuffCollections</c> in the SDE numbers them.</summary>
public sealed record ProjectedBuff(int Id, double Value);

/// <summary>An effect aimed at a fit, like a stasis webifier.</summary>
public sealed record ProjectedEffect(int TypeId, int EffectId, IReadOnlyDictionary<int, double> Attributes, int Quantity = 1);

/// <summary>What one fit hands to another.</summary>
public sealed record Projection
{
    public IReadOnlyList<ProjectedBuff> Buffs { get; init; } = [];
    public IReadOnlyList<ProjectedEffect> Effects { get; init; } = [];
}

/// <summary>A ship, what is fitted to it, and the character flying it.</summary>
public sealed record Fit
{
    public required Ship Ship { get; init; }
    public IReadOnlyList<FitItem> Items { get; init; } = [];
    public string? Name { get; init; }
    public Character Character { get; init; } = new();
    public FitEnvironment Environment { get; init; } = new();
    public Projection Incoming { get; init; } = new();
}

/// <summary>What <c>Calculate</c> reports on top of the values.</summary>
public sealed record Options(bool Sources = false, bool Validate = false);

/// <summary>An item of an ESI fitting; <c>Flag</c> is where it is, like <c>HiSlot0</c>, <c>DroneBay</c> or <c>Cargo</c>.</summary>
public sealed record EsiFittingItem(string Flag, int Quantity, int TypeId);

/// <summary>A fitting as ESI saves it for a character.</summary>
public sealed record EsiFitting(
    string Name,
    int ShipTypeId,
    IReadOnlyList<EsiFittingItem> Items,
    string Description = "",
    int? FittingId = null);

/// <summary>An item of a killmail; <c>Flag</c> is where it was, like 27 for the first high slot.</summary>
public sealed record EsiKillmailItem(int Flag, int ItemTypeId, int QuantityDestroyed = 0, int QuantityDropped = 0);

/// <summary>The ship that died.</summary>
public sealed record EsiKillmailVictim(int ShipTypeId)
{
    public IReadOnlyList<EsiKillmailItem> Items { get; init; } = [];
}

/// <summary>A killmail as ESI returns it; only what a fit is made from.</summary>
public sealed record EsiKillmail(int KillmailId, EsiKillmailVictim Victim);

/// <summary>How a modifier changes an attribute, in the order they are applied.</summary>
public enum EffectOperator { PreAssign, PreMul, PreDiv, ModAdd, ModSub, PostMul, PostDiv, PostPercent, PostAssign }

/// <summary>Where a modifier came from; <c>Index</c> points into <c>Fit.Items</c>, or <c>Fit.Incoming.Effects</c> for <c>Projected</c>.</summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(Ship), "ship")]
[JsonDerivedType(typeof(Mode), "mode")]
[JsonDerivedType(typeof(Character), "character")]
[JsonDerivedType(typeof(Item), "item")]
[JsonDerivedType(typeof(Charge), "charge")]
[JsonDerivedType(typeof(Skill), "skill")]
[JsonDerivedType(typeof(Projected), "projected")]
[JsonDerivedType(typeof(Buff), "buff")]
public abstract record SourceRef
{
    private SourceRef() { }

    public sealed record Ship : SourceRef;
    public sealed record Mode : SourceRef;
    public sealed record Character : SourceRef;
    public sealed record Item(int Index) : SourceRef;
    public sealed record Charge(int Index) : SourceRef;
    public sealed record Skill(int TypeId) : SourceRef;
    public sealed record Projected(int Index) : SourceRef;
    public sealed record Buff(int Id) : SourceRef;
}

/// <summary>Reads a JSON <c>null</c> as NaN, which is how the engine writes a non-finite value.</summary>
internal sealed class NonFiniteDouble : JsonConverter<double>
{
    public override bool HandleNull => true;

    public override double Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options) =>
        reader.TokenType == JsonTokenType.Null ? double.NaN : reader.GetDouble();

    public override void Write(Utf8JsonWriter writer, double value, JsonSerializerOptions options) =>
        writer.WriteNumberValue(value);
}

/// <summary>One modifier on an attribute, and where it came from.</summary>
public sealed record Source(
    SourceRef From,
    int? EffectId,
    int? SourceAttributeId,
    EffectOperator Operator,
    [property: JsonConverter(typeof(NonFiniteDouble))] double Value,
    int Quantity,
    double? Penalty,
    bool Applied);

/// <summary>One attribute, before and after the effects on it.</summary>
public sealed record AttributeValue(
    [property: JsonConverter(typeof(NonFiniteDouble))] double Base,
    [property: JsonConverter(typeof(NonFiniteDouble))] double Value)
{
    public IReadOnlyList<Source> Sources { get; init; } = [];
}

/// <summary>The calculated attributes of the ship, its mode, the character, or one item.</summary>
public sealed record ItemResult(
    IReadOnlyDictionary<int, AttributeValue> Attributes,
    State State,
    State MaxState,
    ItemResult? Charge = null);

/// <summary>The result of <c>Calculate</c>; <c>Items</c> is in the same order as <c>Fit.Items</c>.</summary>
public sealed record Calculation(
    ItemResult Ship,
    IReadOnlyList<ItemResult> Items,
    ItemResult Character,
    ItemResult? Mode = null,
    IReadOnlyList<Violation>? Violations = null)
{
    public IReadOnlyList<ProjectedBuff> Buffs { get; init; } = [];
    public Projection Outgoing { get; init; } = new();
}

/// <summary>A fitting resource.</summary>
public enum ResourceKind
{
    Cpu,
    Powergrid,
    Calibration,
    DroneBay,
    DroneBandwidth,
    LaunchedDrones,
    FighterBay,
    FighterTubes,
    LightFighterTubes,
    SupportFighterTubes,
    HeavyFighterTubes,
    CargoBay,
    ChargeCapacity,
}

/// <summary>A kind of slot a ship has a number of.</summary>
public enum SlotKind { High, Medium, Low, Rig, Subsystem, Service, Turret, Launcher }

/// <summary>Which items count towards a group limit.</summary>
public enum GroupLimit { Fitted, Online, Active }

/// <summary>What breaks a rule: the ship, or an item or its charge by its place in <c>Fit.Items</c>.</summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(Ship), "ship")]
[JsonDerivedType(typeof(Item), "item")]
[JsonDerivedType(typeof(Charge), "charge")]
public abstract record Target
{
    private Target() { }

    public sealed record Ship : Target;
    public sealed record Item(int Index) : Target;
    public sealed record Charge(int Index) : Target;
}

/// <summary>A rule of EVE's, and the values that failed it.</summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type")]
[JsonDerivedType(typeof(Resource), "resource")]
[JsonDerivedType(typeof(Slots), "slots")]
[JsonDerivedType(typeof(WrongSlot), "wrong_slot")]
[JsonDerivedType(typeof(WrongSlotIndex), "wrong_slot_index")]
[JsonDerivedType(typeof(Skill), "skill")]
[JsonDerivedType(typeof(NotASkill), "not_a_skill")]
[JsonDerivedType(typeof(RigSize), "rig_size")]
[JsonDerivedType(typeof(MaxGroup), "max_group")]
[JsonDerivedType(typeof(MaxType), "max_type")]
[JsonDerivedType(typeof(ChargeSize), "charge_size")]
[JsonDerivedType(typeof(NotADrone), "not_a_drone")]
[JsonDerivedType(typeof(NotAFighter), "not_a_fighter")]
[JsonDerivedType(typeof(SlotTaken), "slot_taken")]
[JsonDerivedType(typeof(SubsystemTaken), "subsystem_taken")]
[JsonDerivedType(typeof(ShipRestricted), "ship_restricted")]
[JsonDerivedType(typeof(CapitalItem), "capital_item")]
[JsonDerivedType(typeof(StructureItem), "structure_item")]
[JsonDerivedType(typeof(ShipItem), "ship_item")]
[JsonDerivedType(typeof(NotAHull), "not_a_hull")]
[JsonDerivedType(typeof(NotAMode), "not_a_mode")]
[JsonDerivedType(typeof(ChargeGroup), "charge_group")]
public abstract record Rule
{
    private Rule() { }

    public sealed record Resource([property: JsonPropertyName("resource")] ResourceKind Kind, double Used, double Available) : Rule;
    public sealed record Slots(SlotKind Slot, int Used, int Available) : Rule;
    public sealed record WrongSlot(SlotKind Expected) : Rule;
    public sealed record WrongSlotIndex(int Expected) : Rule;
    public sealed record Skill(int TypeId, int Required, int Level) : Rule;
    public sealed record NotASkill(int TypeId) : Rule;
    public sealed record RigSize(int Ship, int Item) : Rule;
    public sealed record MaxGroup(int GroupId, GroupLimit Limit, int Used, int Allowed) : Rule;
    public sealed record MaxType(int TypeId, int Used, int Allowed) : Rule;
    public sealed record ChargeSize(int Module, int Charge) : Rule;
    public sealed record NotADrone : Rule;
    public sealed record NotAFighter : Rule;
    public sealed record SlotTaken : Rule;
    public sealed record SubsystemTaken : Rule;
    public sealed record ShipRestricted : Rule;
    public sealed record CapitalItem : Rule;
    public sealed record StructureItem : Rule;
    public sealed record ShipItem : Rule;
    public sealed record NotAHull : Rule;
    public sealed record NotAMode : Rule;
    public sealed record ChargeGroup : Rule;
}

/// <summary>One rule the fit breaks, and what breaks it.</summary>
public sealed record Violation(Target Target, Rule Rule);
