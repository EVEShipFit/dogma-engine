package fit.eveship.dogma

import java.nio.file.Files
import java.nio.file.Path
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy

/**
 * Calculates the dogma attributes of an EVE Online ship fit.
 *
 * Load `sde.dat` once with [loadSde], then calculate as many fits as you like.
 * Every call is thread-safe.
 */
object DogmaEngine {
    @OptIn(ExperimentalSerializationApi::class)
    private val json = Json {
        namingStrategy = JsonNamingStrategy.SnakeCase
        explicitNulls = false
        encodeDefaults = true
    }

    /** Load `sde.dat`; returns its build number. Only once per process. */
    fun loadSde(bytes: ByteArray): Int = Native.loadSde(bytes)

    /** Load `sde.dat` from disk; returns its build number. Only once per process. */
    fun loadSde(path: Path): Int = loadSde(Files.readAllBytes(path))

    /** Load `names.dat`, so imports also match names in other languages than English. Only once per process. */
    fun loadNames(bytes: ByteArray): Int = Native.loadNames(bytes)

    /** Load `names.dat` from disk; returns its build number. Only once per process. */
    fun loadNames(path: Path): Int = loadNames(Files.readAllBytes(path))

    /** Load a fit from EFT, the text format EVE copies a fit to the clipboard in. */
    fun loadEft(eft: String): Fit = json.decodeFromString(Native.loadEft(eft))

    /** Write a fit as EFT, in English. */
    fun saveEft(fit: Fit): String = Native.saveEft(json.encodeToString(fit))

    /** Load a fit from an ESI fitting, the fits a character saves in game. */
    fun loadEsiFitting(fitting: EsiFitting): Fit =
        json.decodeFromString(Native.loadEsiFitting(json.encodeToString(fitting)))

    /** Write a fit as an ESI fitting. */
    fun saveEsiFitting(fit: Fit): EsiFitting =
        json.decodeFromString(Native.saveEsiFitting(json.encodeToString(fit)))

    /** Set an imported fit to the states EVE gives it on import; set the skills of the character first. */
    fun postLoad(fit: Fit): Fit = json.decodeFromString(Native.postLoad(json.encodeToString(fit)))

    /** Load the fit of the ship that died from a killmail, as ESI returns it. */
    fun loadKillmail(killmail: EsiKillmail): Fit =
        json.decodeFromString(Native.loadKillmail(json.encodeToString(killmail)))

    /** Load a fit from DNA, the fits EVE links to in chat. */
    fun loadDna(dna: String): Fit = json.decodeFromString(Native.loadDna(dna))

    /** Load a fit from an EVEShip.fit link, given its version and its payload once unbase64'd and gunzipped. */
    fun loadLink(version: String, payload: String): Fit =
        json.decodeFromString(Native.loadLink(version, payload))

    /** Load a fit from an esf/1 document, as text. */
    fun loadEsf(text: String): Fit = json.decodeFromString(Native.loadEsf(text))

    /** Load a fit from an esf/1 link: the binary form, in base64url. */
    fun loadEsfLink(link: String): Fit = json.decodeFromString(Native.loadEsfLink(link))

    /** Write a fit as an esf/1 document, as text. */
    fun saveEsf(fit: Fit): String = Native.saveEsf(json.encodeToString(fit))

    /** Write a fit as an esf/1 link: the binary form, in base64url. */
    fun saveEsfLink(fit: Fit): String = Native.saveEsfLink(json.encodeToString(fit))

    /** Calculate every attribute of the ship, its items and the character. */
    fun calculate(fit: Fit, options: Options = Options()): Calculation =
        json.decodeFromString(Native.calculate(json.encodeToString(fit), json.encodeToString(options)))

    /** What a beacon in space hands to every fit in there with it; put it in [Fit.incoming]. */
    fun beacon(typeId: Int): Projection = json.decodeFromString(Native.beacon(typeId))
}
