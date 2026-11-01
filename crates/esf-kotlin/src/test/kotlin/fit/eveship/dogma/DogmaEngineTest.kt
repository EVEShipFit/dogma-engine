package fit.eveship.dogma

import java.nio.file.Path
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

const val RIFTER = 587
const val AUTOCANNON = 2873
const val AUTOCANNON_200MM = 486
const val EMP_S = 185
const val STASIS_WEBIFIER = 527
const val REACTIVE_ARMOR_HARDENER = 4403
const val GUNNERY = 3300
const val MINMATAR_FRIGATE = 3329
const val STRUCTURE_HP = 9
const val RATE_OF_FIRE = 51
const val MAX_VELOCITY = 37
const val ARMOR_KINETIC_RESONANCE = 269

private fun data(variable: String, file: String): Path =
    Path.of(System.getenv(variable) ?: "../../node_modules/@eveshipfit/sde/dist/$file")

private object Loaded {
    init {
        DogmaEngine.loadSde(data("ESF_SDE", "sde.dat"))
        DogmaEngine.loadNames(data("ESF_NAMES", "names.dat"))
    }

    fun ensure() = Unit
}

private val autocannon = FitItem(typeId = AUTOCANNON, slot = Slot.High(0), state = State.ACTIVE)

/** A Rifter with one autocannon, plus whatever else is asked for. */
private fun fit(
    items: List<FitItem> = emptyList(),
    character: Character = Character(),
    environment: Environment = Environment(),
    incoming: Projection = Projection(),
) = Fit(
    ship = Ship(RIFTER),
    items = listOf(autocannon) + items,
    character = character,
    environment = environment,
    incoming = incoming,
)

private fun Calculation.ship(attribute: Int) = ship.attributes.getValue(attribute).value

private fun Calculation.rateOfFire() = items[0].attributes.getValue(RATE_OF_FIRE).value

class DogmaEngineTest {
    @BeforeTest
    fun load() = Loaded.ensure()

    @Test
    fun calculatesAFit() {
        val calculation = DogmaEngine.calculate(fit())

        assertTrue(calculation.ship(STRUCTURE_HP) > 0)
        assertEquals(State.ACTIVE, calculation.items[0].state)
        assertEquals(1, calculation.items.size)
    }

    @Test
    fun skillsApply() {
        val trained = DogmaEngine.calculate(fit(character = Character(mapOf(GUNNERY to 5))))
        val untrained = DogmaEngine.calculate(fit(character = Character(mapOf(GUNNERY to 0))))

        assertTrue(trained.rateOfFire() < untrained.rateOfFire())
    }

    @Test
    fun outgoingFeedsBackIntoIncoming() {
        val webifier = DogmaEngine.calculate(
            Fit(
                ship = Ship(RIFTER),
                items = listOf(FitItem(typeId = STASIS_WEBIFIER, slot = Slot.Medium(0), state = State.ACTIVE)),
            ),
        )
        assertTrue(webifier.outgoing.effects.isNotEmpty())

        val webbed = DogmaEngine.calculate(fit(incoming = webifier.outgoing))
        val unwebbed = DogmaEngine.calculate(fit())

        assertTrue(webbed.ship(MAX_VELOCITY) < unwebbed.ship(MAX_VELOCITY))
    }

    @Test
    fun sourcesAreOnlyReportedWhenAskedFor() {
        val skilled = fit(character = Character(mapOf(GUNNERY to 5)))

        val without = DogmaEngine.calculate(skilled)
        val withSources = DogmaEngine.calculate(skilled, Options(sources = true))

        assertTrue(without.items[0].attributes.getValue(RATE_OF_FIRE).sources.isEmpty())
        assertContains(
            withSources.items[0].attributes.getValue(RATE_OF_FIRE).sources.map { it.from },
            SourceRef.Skill(GUNNERY),
        )
    }

    @Test
    fun reactiveArmorTakesAProfileOfItsOwn() {
        val hardener = FitItem(typeId = REACTIVE_ARMOR_HARDENER, slot = Slot.Low(0), state = State.ACTIVE)

        val inert = DogmaEngine.calculate(fit(items = listOf(hardener)))
        val adapting = DogmaEngine.calculate(
            fit(
                items = listOf(hardener),
                environment = Environment(reactiveArmor = ReactiveArmor.Profile(DamageProfile(kinetic = 1.0))),
            ),
        )

        assertTrue(adapting.ship(ARMOR_KINETIC_RESONANCE) < inert.ship(ARMOR_KINETIC_RESONANCE))
    }

    @Test
    fun validateReportsAFullRack() {
        val extra = (1..3).map { FitItem(typeId = AUTOCANNON, slot = Slot.High(it), state = State.ACTIVE) }

        val calculation = DogmaEngine.calculate(fit(items = extra), Options(validate = true))

        assertContains(
            calculation.violations.orEmpty(),
            Violation(Target.Ship, Rule.Slots(SlotKind.HIGH, used = 4, available = 3)),
        )
    }

    @Test
    fun validateReportsAMissingSkill() {
        val calculation = DogmaEngine.calculate(fit(), Options(validate = true))

        assertContains(
            calculation.violations.orEmpty(),
            Violation(Target.Ship, Rule.Skill(MINMATAR_FRIGATE, required = 1, level = 0)),
        )
    }

    @Test
    fun beaconReturnsAProjection() {
        DogmaEngine.beacon(RIFTER)
    }

    @Test
    fun loadingTheSdeTwiceThrows() {
        assertFailsWith<IllegalStateException> { DogmaEngine.loadSde(ByteArray(0)) }
    }

    @Test
    fun loadingTheNamesTwiceThrows() {
        assertFailsWith<IllegalStateException> { DogmaEngine.loadNames(ByteArray(0)) }
    }

    @Test
    fun loadsAnEft() {
        val fit = DogmaEngine.loadEft("[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n")

        assertEquals(RIFTER, fit.ship.typeId)
        assertEquals(AUTOCANNON_200MM, fit.items[0].typeId)
        assertEquals(Slot.High(0), fit.items[0].slot)
        assertEquals(Charge(EMP_S), fit.items[0].charge)
    }

    @Test
    fun anEftMatchesNamesInAnotherLanguage() {
        val fit = DogmaEngine.loadEft("[Rifter, Mon Rifter]\nCanon Automatique 200mm I\n")

        assertEquals(AUTOCANNON_200MM, fit.items[0].typeId)
    }

    @Test
    fun savesAnEft() {
        val eft = "[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n"

        assertEquals(eft, DogmaEngine.saveEft(DogmaEngine.loadEft(eft)))
    }

    @Test
    fun aBadEftThrows() {
        assertFailsWith<IllegalArgumentException> { DogmaEngine.loadEft("not a fit") }
    }

    @Test
    fun anEsiFittingRoundTrips() {
        val fitting = EsiFitting(
            name = "Gun",
            shipTypeId = RIFTER,
            items = listOf(
                EsiFittingItem(flag = "HiSlot0", quantity = 1, typeId = AUTOCANNON_200MM),
                EsiFittingItem(flag = "HiSlot0", quantity = 1, typeId = EMP_S),
            ),
        )

        val fit = DogmaEngine.loadEsiFitting(fitting)
        assertEquals(Charge(EMP_S), fit.items[0].charge)
        assertEquals(fitting, DogmaEngine.saveEsiFitting(fit))
    }

    @Test
    fun postLoadSetsStates() {
        val fit = DogmaEngine.loadEft("[Rifter, My Rifter]\n200mm AutoCannon I\n")

        val settled = DogmaEngine.postLoad(fit.copy(character = Character(mapOf(GUNNERY to 5))))

        assertEquals(State.ACTIVE, settled.items[0].state)
    }

    @Test
    fun aKillmailGivesTheFitOfTheShipThatDied() {
        val killmail = EsiKillmail(
            killmailId = 1,
            victim = EsiKillmailVictim(
                shipTypeId = RIFTER,
                items = listOf(
                    EsiKillmailItem(flag = 27, itemTypeId = AUTOCANNON_200MM, quantityDestroyed = 1),
                    EsiKillmailItem(flag = 27, itemTypeId = EMP_S, quantityDropped = 1),
                ),
            ),
        )

        val fit = DogmaEngine.loadKillmail(killmail)
        assertEquals("Killmail 1", fit.name)
        assertEquals(Slot.High(0), fit.items[0].slot)
        assertEquals(Charge(EMP_S), fit.items[0].charge)
    }

    @Test
    fun dnaGivesItsFit() {
        val fit = DogmaEngine.loadDna("fitting:$RIFTER:$AUTOCANNON_200MM;2:$EMP_S;100::")

        assertEquals(RIFTER, fit.ship.typeId)
        assertEquals(Slot.High(1), fit.items[1].slot)
        assertEquals(Slot.Cargo, fit.items[2].slot)
    }

    @Test
    fun aLinkGivesItsFit() {
        val fit = DogmaEngine.loadLink("v2", "$RIFTER,Gun,\n27,$AUTOCANNON_200MM,1,$EMP_S,Active")

        assertEquals("Gun", fit.name)
        assertEquals(Charge(EMP_S), fit.items[0].charge)
    }

    @Test
    fun anEsfRoundTrips() {
        val esf = "%esf/1\nRifter \"My Rifter\"\n\n200mm AutoCannon I :EMP S\n"
        val fit = DogmaEngine.loadEsf(esf)

        assertEquals(RIFTER, fit.ship.typeId)
        assertEquals(esf, DogmaEngine.saveEsf(fit))
    }

    @Test
    fun anEsfLinkRoundTrips() {
        val fit = DogmaEngine.loadEsf("%esf/1\nRifter\n200mm AutoCannon I :EMP S\n")
        val link = DogmaEngine.saveEsfLink(fit)

        assertEquals(DogmaEngine.saveEsf(fit), DogmaEngine.saveEsf(DogmaEngine.loadEsfLink(link)))
    }

    @Test
    fun aBadEsfThrows() {
        val error = assertFailsWith<IllegalArgumentException> { DogmaEngine.loadEsf("%esf/1\nNot A Ship\n") }

        assertContains(error.message.orEmpty(), "line 2")
    }
}
