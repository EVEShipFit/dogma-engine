using System.Text.Json;
using System.Text.Json.Serialization;

namespace EveShipFit.Dogma;

/// <summary>
/// Calculates the dogma attributes of an EVE Online ship fit.
/// Load <c>sde.dat</c> once with <see cref="LoadSde(string)"/>, then calculate as many fits as you like.
/// Every call is thread-safe.
/// </summary>
public static class DogmaEngine
{
    private static readonly JsonSerializerOptions Json = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull,
        Converters = { new JsonStringEnumConverter(JsonNamingPolicy.SnakeCaseLower) },
    };

    private static string Encode<T>(T value) => JsonSerializer.Serialize(value, Json);

    private static T Decode<T>(string json) => JsonSerializer.Deserialize<T>(json, Json)!;

    /// <summary>Load <c>sde.dat</c>; returns its build number. Only once per process.</summary>
    public static int LoadSde(ReadOnlySpan<byte> bytes) => Native.LoadSde(bytes);

    /// <summary>Load <c>sde.dat</c> from disk; returns its build number. Only once per process.</summary>
    public static int LoadSde(string path) => LoadSde(File.ReadAllBytes(path));

    /// <summary>Load <c>names.dat</c>, so imports also match names in other languages than English. Only once per process.</summary>
    public static int LoadNames(ReadOnlySpan<byte> bytes) => Native.LoadNames(bytes);

    /// <summary>Load <c>names.dat</c> from disk; returns its build number. Only once per process.</summary>
    public static int LoadNames(string path) => LoadNames(File.ReadAllBytes(path));

    /// <summary>Load a fit from EFT, the text format EVE copies a fit to the clipboard in.</summary>
    public static Fit LoadEft(string eft) => Decode<Fit>(Native.LoadEft(eft));

    /// <summary>Write a fit as EFT, in English.</summary>
    public static string SaveEft(Fit fit) => Native.SaveEft(Encode(fit));

    /// <summary>Load a fit from an ESI fitting, the fits a character saves in game.</summary>
    public static Fit LoadEsiFitting(EsiFitting fitting) => Decode<Fit>(Native.LoadEsiFitting(Encode(fitting)));

    /// <summary>Write a fit as an ESI fitting.</summary>
    public static EsiFitting SaveEsiFitting(Fit fit) => Decode<EsiFitting>(Native.SaveEsiFitting(Encode(fit)));

    /// <summary>Set an imported fit to the states EVE gives it on import; set the skills of the character first.</summary>
    public static Fit PostLoad(Fit fit) => Decode<Fit>(Native.PostLoad(Encode(fit)));

    /// <summary>Load the fit of the ship that died from a killmail, as ESI returns it.</summary>
    public static Fit LoadKillmail(EsiKillmail killmail) => Decode<Fit>(Native.LoadKillmail(Encode(killmail)));

    /// <summary>Load a fit from DNA, the fits EVE links to in chat.</summary>
    public static Fit LoadDna(string dna) => Decode<Fit>(Native.LoadDna(dna));

    /// <summary>Load a fit from an EVEShip.fit link, given its version and its payload once unbase64'd and gunzipped.</summary>
    public static Fit LoadLink(string version, string payload) => Decode<Fit>(Native.LoadLink(version, payload));

    /// <summary>Load a fit from an esf/1 document, as text.</summary>
    public static Fit LoadEsf(string text) => Decode<Fit>(Native.LoadEsf(text));

    /// <summary>Load a fit from an esf/1 link: the binary form, in base64url.</summary>
    public static Fit LoadEsfLink(string link) => Decode<Fit>(Native.LoadEsfLink(link));

    /// <summary>Write a fit as an esf/1 document, as text.</summary>
    public static string SaveEsf(Fit fit) => Native.SaveEsf(Encode(fit));

    /// <summary>Write a fit as an esf/1 link: the binary form, in base64url.</summary>
    public static string SaveEsfLink(Fit fit) => Native.SaveEsfLink(Encode(fit));

    /// <summary>Calculate every attribute of the ship, its items and the character.</summary>
    public static Calculation Calculate(Fit fit, Options? options = null) =>
        Decode<Calculation>(Native.Calculate(Encode(fit), Encode(options ?? new Options())));

    /// <summary>What a beacon in space hands to every fit in there with it; put it in <see cref="Fit.Incoming"/>.</summary>
    public static Projection Beacon(int typeId) => Decode<Projection>(Native.Beacon(typeId));
}
