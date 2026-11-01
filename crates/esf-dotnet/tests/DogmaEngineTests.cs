using Xunit;

namespace EveShipFit.Dogma.Tests;

public class DogmaEngineTests
{
    private const int Rifter = 587;
    private const int Autocannon = 2873;
    private const int Autocannon200mm = 486;
    private const int EmpS = 185;
    private const int StasisWebifier = 527;
    private const int ReactiveArmorHardener = 4403;
    private const int Gunnery = 3300;
    private const int MinmatarFrigate = 3329;
    private const int StructureHp = 9;
    private const int RateOfFire = 51;
    private const int MaxVelocity = 37;
    private const int ArmorKineticResonance = 269;

    private static string Data(string variable, string file) =>
        System.Environment.GetEnvironmentVariable(variable) ?? $"../../../../../../node_modules/@eveshipfit/sde/dist/{file}";

    static DogmaEngineTests()
    {
        DogmaEngine.LoadSde(Data("ESF_SDE", "sde.dat"));
        DogmaEngine.LoadNames(Data("ESF_NAMES", "names.dat"));
    }

    private static readonly FitItem AutocannonItem = new(Autocannon, new Slot.High(0), State.Active);

    /// <summary>A Rifter with one autocannon, plus whatever else is asked for.</summary>
    private static Fit RifterFit(
        IEnumerable<FitItem>? items = null,
        Character? character = null,
        FitEnvironment? environment = null,
        Projection? incoming = null) => new()
    {
        Ship = new Ship(Rifter),
        Items = [AutocannonItem, .. items ?? []],
        Character = character ?? new Character(),
        Environment = environment ?? new FitEnvironment(),
        Incoming = incoming ?? new Projection(),
    };

    private static Character Skilled(int level) => new() { Skills = new Dictionary<int, int> { [Gunnery] = level } };

    private static double ShipAttribute(Calculation calculation, int attribute) =>
        calculation.Ship.Attributes[attribute].Value;

    private static double RateOfFireOf(Calculation calculation) =>
        calculation.Items[0].Attributes[RateOfFire].Value;

    [Fact]
    public void CalculatesAFit()
    {
        var calculation = DogmaEngine.Calculate(RifterFit());

        Assert.True(ShipAttribute(calculation, StructureHp) > 0);
        Assert.Equal(State.Active, calculation.Items[0].State);
        Assert.Single(calculation.Items);
    }

    [Fact]
    public void SkillsApply()
    {
        var trained = DogmaEngine.Calculate(RifterFit(character: Skilled(5)));
        var untrained = DogmaEngine.Calculate(RifterFit(character: Skilled(0)));

        Assert.True(RateOfFireOf(trained) < RateOfFireOf(untrained));
    }

    [Fact]
    public void OutgoingFeedsBackIntoIncoming()
    {
        var webifier = DogmaEngine.Calculate(new Fit
        {
            Ship = new Ship(Rifter),
            Items = [new FitItem(StasisWebifier, new Slot.Medium(0), State.Active)],
        });
        Assert.NotEmpty(webifier.Outgoing.Effects);

        var webbed = DogmaEngine.Calculate(RifterFit(incoming: webifier.Outgoing));
        var unwebbed = DogmaEngine.Calculate(RifterFit());

        Assert.True(ShipAttribute(webbed, MaxVelocity) < ShipAttribute(unwebbed, MaxVelocity));
    }

    [Fact]
    public void SourcesAreOnlyReportedWhenAskedFor()
    {
        var skilled = RifterFit(character: Skilled(5));

        var without = DogmaEngine.Calculate(skilled);
        var withSources = DogmaEngine.Calculate(skilled, new Options(Sources: true));

        Assert.Empty(without.Items[0].Attributes[RateOfFire].Sources);
        Assert.Contains(
            new SourceRef.Skill(Gunnery),
            withSources.Items[0].Attributes[RateOfFire].Sources.Select(source => source.From));
    }

    [Fact]
    public void ReactiveArmorTakesAProfileOfItsOwn()
    {
        var hardener = new FitItem(ReactiveArmorHardener, new Slot.Low(0), State.Active);

        var inert = DogmaEngine.Calculate(RifterFit(items: [hardener]));
        var adapting = DogmaEngine.Calculate(RifterFit(
            items: [hardener],
            environment: new FitEnvironment { ReactiveArmor = new ReactiveArmor.Profile(new DamageProfile(Kinetic: 1.0)) }));

        Assert.True(ShipAttribute(adapting, ArmorKineticResonance) < ShipAttribute(inert, ArmorKineticResonance));
    }

    [Fact]
    public void ValidateReportsAFullRack()
    {
        var extra = Enumerable.Range(1, 3).Select(index => new FitItem(Autocannon, new Slot.High(index), State.Active));

        var calculation = DogmaEngine.Calculate(RifterFit(items: extra), new Options(Validate: true));

        Assert.Contains(
            new Violation(new Target.Ship(), new Rule.Slots(SlotKind.High, Used: 4, Available: 3)),
            calculation.Violations!);
    }

    [Fact]
    public void ValidateReportsAMissingSkill()
    {
        var calculation = DogmaEngine.Calculate(RifterFit(), new Options(Validate: true));

        Assert.Contains(
            new Violation(new Target.Ship(), new Rule.Skill(MinmatarFrigate, Required: 1, Level: 0)),
            calculation.Violations!);
    }

    [Fact]
    public void BeaconReturnsAProjection()
    {
        DogmaEngine.Beacon(Rifter);
    }

    [Fact]
    public void LoadingTheSdeTwiceThrows()
    {
        Assert.Throws<InvalidOperationException>(() => DogmaEngine.LoadSde([]));
    }

    [Fact]
    public void LoadingTheNamesTwiceThrows()
    {
        Assert.Throws<InvalidOperationException>(() => DogmaEngine.LoadNames([]));
    }

    [Fact]
    public void LoadsAnEft()
    {
        var fit = DogmaEngine.LoadEft("[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n");

        Assert.Equal(Rifter, fit.Ship.TypeId);
        Assert.Equal(Autocannon200mm, fit.Items[0].TypeId);
        Assert.Equal(new Slot.High(0), fit.Items[0].Slot);
        Assert.Equal(new Charge(EmpS), fit.Items[0].Charge);
    }

    [Fact]
    public void AnEftMatchesNamesInAnotherLanguage()
    {
        var fit = DogmaEngine.LoadEft("[Rifter, Mon Rifter]\nCanon Automatique 200mm I\n");

        Assert.Equal(Autocannon200mm, fit.Items[0].TypeId);
    }

    [Fact]
    public void SavesAnEft()
    {
        const string eft = "[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n";

        Assert.Equal(eft, DogmaEngine.SaveEft(DogmaEngine.LoadEft(eft)));
    }

    [Fact]
    public void ABadEftThrows()
    {
        Assert.Throws<ArgumentException>(() => DogmaEngine.LoadEft("not a fit"));
    }

    [Fact]
    public void AnEsiFittingRoundTrips()
    {
        var fitting = new EsiFitting(
            Name: "Gun",
            ShipTypeId: Rifter,
            Items:
            [
                new EsiFittingItem(Flag: "HiSlot0", Quantity: 1, TypeId: Autocannon200mm),
                new EsiFittingItem(Flag: "HiSlot0", Quantity: 1, TypeId: EmpS),
            ]);

        var fit = DogmaEngine.LoadEsiFitting(fitting);
        Assert.Equal(new Charge(EmpS), fit.Items[0].Charge);

        var saved = DogmaEngine.SaveEsiFitting(fit);
        Assert.Equal((fitting.Name, fitting.ShipTypeId, fitting.Description), (saved.Name, saved.ShipTypeId, saved.Description));
        Assert.Equal(fitting.Items, saved.Items);
    }

    [Fact]
    public void PostLoadOnlinesACloak()
    {
        var fit = DogmaEngine.LoadEft("[Rifter, Cloak]\n\n\nPrototype Cloaking Device I\n");
        Assert.Equal(State.Active, fit.Items[0].State);

        Assert.Equal(State.Online, DogmaEngine.PostLoad(fit).Items[0].State);
    }

    [Fact]
    public void AKillmailGivesTheFitOfTheShipThatDied()
    {
        var killmail = new EsiKillmail(
            KillmailId: 1,
            Victim: new EsiKillmailVictim(ShipTypeId: Rifter)
            {
                Items =
                [
                    new EsiKillmailItem(Flag: 27, ItemTypeId: Autocannon200mm, QuantityDestroyed: 1),
                    new EsiKillmailItem(Flag: 27, ItemTypeId: EmpS, QuantityDropped: 1),
                ],
            });

        var fit = DogmaEngine.LoadKillmail(killmail);
        Assert.Equal("Killmail 1", fit.Name);
        Assert.Equal(new Slot.High(0), fit.Items[0].Slot);
        Assert.Equal(new Charge(EmpS), fit.Items[0].Charge);
    }

    [Fact]
    public void DnaGivesItsFit()
    {
        var fit = DogmaEngine.LoadDna($"fitting:{Rifter}:{Autocannon200mm};2:{EmpS};100::");

        Assert.Equal(Rifter, fit.Ship.TypeId);
        Assert.Equal(new Slot.High(1), fit.Items[1].Slot);
        Assert.Equal(new Slot.Cargo(), fit.Items[2].Slot);
    }

    [Fact]
    public void ALinkGivesItsFit()
    {
        var fit = DogmaEngine.LoadLink("v2", $"{Rifter},Gun,\n27,{Autocannon200mm},1,{EmpS},Active");

        Assert.Equal("Gun", fit.Name);
        Assert.Equal(new Charge(EmpS), fit.Items[0].Charge);
    }

    [Fact]
    public void AnEsfRoundTrips()
    {
        const string esf = "%esf/1\nRifter \"My Rifter\"\n\n200mm AutoCannon I :EMP S\n";
        var fit = DogmaEngine.LoadEsf(esf);

        Assert.Equal(Rifter, fit.Ship.TypeId);
        Assert.Equal(esf, DogmaEngine.SaveEsf(fit));
    }

    [Fact]
    public void AnEsfLinkRoundTrips()
    {
        var fit = DogmaEngine.LoadEsf("%esf/1\nRifter\n200mm AutoCannon I :EMP S\n");
        var link = DogmaEngine.SaveEsfLink(fit);

        Assert.Equal(DogmaEngine.SaveEsf(fit), DogmaEngine.SaveEsf(DogmaEngine.LoadEsfLink(link)));
    }

    [Fact]
    public void ABadEsfThrows()
    {
        var error = Assert.Throws<ArgumentException>(() => DogmaEngine.LoadEsf("%esf/1\nNot A Ship\n"));

        Assert.Contains("line 2", error.Message);
    }
}
