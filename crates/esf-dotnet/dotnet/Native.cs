using System.Runtime.InteropServices;
using System.Text;

namespace EveShipFit.Dogma;

internal static unsafe partial class Native
{
    private const string Library = "esf_dogma_engine_dotnet";

    [StructLayout(LayoutKind.Sequential)]
    private struct Result
    {
        public int Status;
        public int Value;
        public byte* Text;
        public nuint Length;
    }

    [LibraryImport(Library)] private static partial void esf_free(byte* text, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_sde(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_names(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_eft(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_save_eft(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_esi_fitting(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_save_esi_fitting(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_post_load(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_killmail(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_dna(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_esf(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_esf_link(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_save_esf(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_save_esf_link(byte* data, nuint length);
    [LibraryImport(Library)] private static partial Result esf_load_link(byte* version, nuint versionLength, byte* payload, nuint payloadLength);
    [LibraryImport(Library)] private static partial Result esf_calculate(byte* fit, nuint fitLength, byte* options, nuint optionsLength);
    [LibraryImport(Library)] private static partial Result esf_beacon(int typeId);

    private static string? Take(Result result)
    {
        string? text = null;
        if (result.Text != null)
        {
            text = Encoding.UTF8.GetString(result.Text, checked((int)result.Length));
            esf_free(result.Text, result.Length);
        }

        return result.Status switch
        {
            0 => text,
            1 => throw new ArgumentException(text),
            _ => throw new InvalidOperationException(text),
        };
    }

    private static int Number(Result result)
    {
        Take(result);
        return result.Value;
    }

    private static string Text(Result result) => Take(result)!;

    private static string Call(delegate*<byte*, nuint, Result> function, string input)
    {
        var bytes = Encoding.UTF8.GetBytes(input);
        fixed (byte* data = bytes)
        {
            return Text(function(data, (nuint)bytes.Length));
        }
    }

    private static string Call(delegate*<byte*, nuint, byte*, nuint, Result> function, string first, string second)
    {
        var firstBytes = Encoding.UTF8.GetBytes(first);
        var secondBytes = Encoding.UTF8.GetBytes(second);
        fixed (byte* firstData = firstBytes)
        fixed (byte* secondData = secondBytes)
        {
            return Text(function(firstData, (nuint)firstBytes.Length, secondData, (nuint)secondBytes.Length));
        }
    }

    public static int LoadSde(ReadOnlySpan<byte> bytes)
    {
        fixed (byte* data = bytes)
        {
            return Number(esf_load_sde(data, (nuint)bytes.Length));
        }
    }

    public static int LoadNames(ReadOnlySpan<byte> bytes)
    {
        fixed (byte* data = bytes)
        {
            return Number(esf_load_names(data, (nuint)bytes.Length));
        }
    }

    public static string LoadEft(string eft) => Call(&esf_load_eft, eft);
    public static string SaveEft(string fit) => Call(&esf_save_eft, fit);
    public static string LoadEsiFitting(string fitting) => Call(&esf_load_esi_fitting, fitting);
    public static string SaveEsiFitting(string fit) => Call(&esf_save_esi_fitting, fit);
    public static string PostLoad(string fit) => Call(&esf_post_load, fit);
    public static string LoadKillmail(string killmail) => Call(&esf_load_killmail, killmail);
    public static string LoadDna(string dna) => Call(&esf_load_dna, dna);
    public static string LoadEsf(string text) => Call(&esf_load_esf, text);
    public static string LoadEsfLink(string link) => Call(&esf_load_esf_link, link);
    public static string SaveEsf(string fit) => Call(&esf_save_esf, fit);
    public static string SaveEsfLink(string fit) => Call(&esf_save_esf_link, fit);
    public static string LoadLink(string version, string payload) => Call(&esf_load_link, version, payload);
    public static string Calculate(string fit, string options) => Call(&esf_calculate, fit, options);
    public static string Beacon(int typeId) => Text(esf_beacon(typeId));
}
