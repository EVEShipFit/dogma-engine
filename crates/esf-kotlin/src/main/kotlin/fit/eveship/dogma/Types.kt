package fit.eveship.dogma

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonPrimitive

/** The state of an item, lowest first. */
@Serializable
enum class State {
    @SerialName("offline") OFFLINE,
    @SerialName("online") ONLINE,
    @SerialName("active") ACTIVE,
    @SerialName("overload") OVERLOAD,
}

/** The security of a solar system. */
@Serializable
enum class Security {
    @SerialName("high_sec") HIGH_SEC,
    @SerialName("low_sec") LOW_SEC,
    @SerialName("null_sec") NULL_SEC,
    @SerialName("wormhole") WORMHOLE,
}

/** Where an item is; `index` starts at 0, but for implants and boosters is the slot as EVE numbers it. */
@Serializable
sealed interface Slot {
    @Serializable @SerialName("high") data class High(val index: Int) : Slot
    @Serializable @SerialName("medium") data class Medium(val index: Int) : Slot
    @Serializable @SerialName("low") data class Low(val index: Int) : Slot
    @Serializable @SerialName("rig") data class Rig(val index: Int) : Slot
    @Serializable @SerialName("subsystem") data class Subsystem(val index: Int) : Slot
    @Serializable @SerialName("service") data class Service(val index: Int) : Slot
    @Serializable @SerialName("fighter_tube") data class FighterTube(val index: Int) : Slot
    @Serializable @SerialName("implant") data class Implant(val index: Int) : Slot
    @Serializable @SerialName("booster") data class Booster(val index: Int) : Slot
    @Serializable @SerialName("fighter_bay") data object FighterBay : Slot
    @Serializable @SerialName("drone_bay") data object DroneBay : Slot
    @Serializable @SerialName("cargo") data object Cargo : Slot
}

/** A charge loaded in a module. */
@Serializable
data class Charge(val typeId: Int)

/** How a module or drone was mutated. */
@Serializable
data class Mutation(val base: Int, val attributes: Map<Int, Double> = emptyMap())

/** How far a module has spooled; 0.0 is unspooled, 2.125 is +212.5%. */
@Serializable
data class Spool(val multiplierBonus: Double)

/** A module, drone, fighter squadron, implant, booster or item in cargo. */
@Serializable
data class FitItem(
    val typeId: Int,
    val slot: Slot,
    val state: State,
    val quantity: Int = 1,
    val charge: Charge? = null,
    val mutation: Mutation? = null,
    val fighterAbilities: Set<Int>? = null,
    val boosterSideEffects: Set<Int> = emptySet(),
    val spool: Spool? = null,
)

/** The ship of a fit. */
@Serializable
data class Ship(val typeId: Int, val mode: Int? = null)

/** The character flying the ship. */
@Serializable
data class Character(
    val skills: Map<Int, Int> = emptyMap(),
    val securityStatus: Double = 0.0,
)

/** Incoming damage, relative to each other. */
@Serializable
data class DamageProfile(
    val em: Double = 0.0,
    val explosive: Double = 0.0,
    val kinetic: Double = 0.0,
    val thermal: Double = 0.0,
)

/** What a Reactive Armor Hardener shifts its resistances towards. */
@Serializable(with = ReactiveArmorSerializer::class)
sealed interface ReactiveArmor {
    data object DoNotAdapt : ReactiveArmor
    data object DamageProfile : ReactiveArmor
    data class Profile(val profile: fit.eveship.dogma.DamageProfile) : ReactiveArmor
}

internal object ReactiveArmorSerializer : KSerializer<ReactiveArmor> {
    override val descriptor: SerialDescriptor = JsonElement.serializer().descriptor

    override fun serialize(encoder: Encoder, value: ReactiveArmor) {
        val json = (encoder as JsonEncoder).json
        encoder.encodeJsonElement(
            when (value) {
                ReactiveArmor.DoNotAdapt -> JsonPrimitive("do_not_adapt")
                ReactiveArmor.DamageProfile -> JsonPrimitive("damage_profile")
                is ReactiveArmor.Profile -> buildJsonObject {
                    put("profile", json.encodeToJsonElement(DamageProfile.serializer(), value.profile))
                }
            },
        )
    }

    override fun deserialize(decoder: Decoder): ReactiveArmor {
        val json = (decoder as JsonDecoder).json
        return when (val element = decoder.decodeJsonElement()) {
            is JsonObject -> ReactiveArmor.Profile(
                json.decodeFromJsonElement(DamageProfile.serializer(), element.getValue("profile")),
            )
            else -> when (val name = element.jsonPrimitive.content) {
                "do_not_adapt" -> ReactiveArmor.DoNotAdapt
                "damage_profile" -> ReactiveArmor.DamageProfile
                else -> throw IllegalArgumentException("unknown reactive armor: $name")
            }
        }
    }
}

/** Where the ship is. */
@Serializable
data class Environment(
    val damageProfile: DamageProfile = DamageProfile(0.25, 0.25, 0.25, 0.25),
    val security: Security = Security.HIGH_SEC,
    val reactiveArmor: ReactiveArmor = ReactiveArmor.DoNotAdapt,
)

/** A buff, as `dbuffCollections` in the SDE numbers them. */
@Serializable
data class ProjectedBuff(val id: Int, val value: Double)

/** An effect aimed at a fit, like a stasis webifier. */
@Serializable
data class ProjectedEffect(
    val typeId: Int,
    val effectId: Int,
    val attributes: Map<Int, Double> = emptyMap(),
    val quantity: Int = 1,
)

/** What one fit hands to another. */
@Serializable
data class Projection(
    val buffs: List<ProjectedBuff> = emptyList(),
    val effects: List<ProjectedEffect> = emptyList(),
)

/** A ship, what is fitted to it, and the character flying it. */
@Serializable
data class Fit(
    val ship: Ship,
    val items: List<FitItem> = emptyList(),
    val name: String? = null,
    val character: Character = Character(),
    val environment: Environment = Environment(),
    val incoming: Projection = Projection(),
)

/** What `calculate` reports on top of the values. */
@Serializable
data class Options(val sources: Boolean = false, val validate: Boolean = false)

/** An item of an ESI fitting; `flag` is where it is, like `HiSlot0`, `DroneBay` or `Cargo`. */
@Serializable
data class EsiFittingItem(val flag: String, val quantity: Int, val typeId: Int)

/** A fitting as ESI saves it for a character. */
@Serializable
data class EsiFitting(
    val name: String,
    val shipTypeId: Int,
    val items: List<EsiFittingItem>,
    val description: String = "",
    val fittingId: Int? = null,
)

/** An item of a killmail; `flag` is where it was, like 27 for the first high slot. */
@Serializable
data class EsiKillmailItem(
    val flag: Int,
    val itemTypeId: Int,
    val quantityDestroyed: Int = 0,
    val quantityDropped: Int = 0,
)

/** The ship that died. */
@Serializable
data class EsiKillmailVictim(val shipTypeId: Int, val items: List<EsiKillmailItem> = emptyList())

/** A killmail as ESI returns it; only what a fit is made from. */
@Serializable
data class EsiKillmail(val killmailId: Int, val victim: EsiKillmailVictim)

/** How a modifier changes an attribute, in the order they are applied. */
@Serializable
enum class EffectOperator {
    @SerialName("pre_assign") PRE_ASSIGN,
    @SerialName("pre_mul") PRE_MUL,
    @SerialName("pre_div") PRE_DIV,
    @SerialName("mod_add") MOD_ADD,
    @SerialName("mod_sub") MOD_SUB,
    @SerialName("post_mul") POST_MUL,
    @SerialName("post_div") POST_DIV,
    @SerialName("post_percent") POST_PERCENT,
    @SerialName("post_assign") POST_ASSIGN,
}

/** Where a modifier came from; `index` points into `Fit.items`, or `Fit.incoming.effects` for `Projected`. */
@Serializable
sealed interface SourceRef {
    @Serializable @SerialName("ship") data object Ship : SourceRef
    @Serializable @SerialName("mode") data object Mode : SourceRef
    @Serializable @SerialName("character") data object Character : SourceRef
    @Serializable @SerialName("item") data class Item(val index: Int) : SourceRef
    @Serializable @SerialName("charge") data class Charge(val index: Int) : SourceRef
    @Serializable @SerialName("skill") data class Skill(val typeId: Int) : SourceRef
    @Serializable @SerialName("projected") data class Projected(val index: Int) : SourceRef
    @Serializable @SerialName("buff") data class Buff(val id: Int) : SourceRef
}

/** Reads a JSON `null` as NaN, which is how the engine writes a non-finite value. */
internal object NonFiniteDouble : KSerializer<Double> {
    override val descriptor: SerialDescriptor = Double.serializer().descriptor

    override fun serialize(encoder: Encoder, value: Double) = encoder.encodeDouble(value)

    override fun deserialize(decoder: Decoder): Double =
        (decoder as JsonDecoder).decodeJsonElement().jsonPrimitive.doubleOrNull ?: Double.NaN
}

/** One modifier on an attribute, and where it came from. */
@Serializable
data class Source(
    val from: SourceRef,
    val effectId: Int?,
    val sourceAttributeId: Int?,
    val operator: EffectOperator,
    @Serializable(with = NonFiniteDouble::class) val value: Double,
    val quantity: Int,
    val penalty: Double?,
    val applied: Boolean,
)

/** One attribute, before and after the effects on it. */
@Serializable
data class AttributeValue(
    @Serializable(with = NonFiniteDouble::class) val base: Double,
    @Serializable(with = NonFiniteDouble::class) val value: Double,
    val sources: List<Source> = emptyList(),
)

/** The calculated attributes of the ship, its mode, the character, or one item. */
@Serializable
data class ItemResult(
    val attributes: Map<Int, AttributeValue>,
    val state: State,
    val maxState: State,
    val charge: ItemResult? = null,
)

/** The result of `calculate`; `items` is in the same order as `Fit.items`. */
@Serializable
data class Calculation(
    val ship: ItemResult,
    val items: List<ItemResult>,
    val character: ItemResult,
    val mode: ItemResult? = null,
    val buffs: List<ProjectedBuff> = emptyList(),
    val outgoing: Projection = Projection(),
    val violations: List<Violation>? = null,
)

/** A fitting resource. */
@Serializable
enum class Resource {
    @SerialName("cpu") CPU,
    @SerialName("powergrid") POWERGRID,
    @SerialName("calibration") CALIBRATION,
    @SerialName("drone_bay") DRONE_BAY,
    @SerialName("drone_bandwidth") DRONE_BANDWIDTH,
    @SerialName("launched_drones") LAUNCHED_DRONES,
    @SerialName("fighter_bay") FIGHTER_BAY,
    @SerialName("fighter_tubes") FIGHTER_TUBES,
    @SerialName("light_fighter_tubes") LIGHT_FIGHTER_TUBES,
    @SerialName("support_fighter_tubes") SUPPORT_FIGHTER_TUBES,
    @SerialName("heavy_fighter_tubes") HEAVY_FIGHTER_TUBES,
    @SerialName("cargo_bay") CARGO_BAY,
    @SerialName("charge_capacity") CHARGE_CAPACITY,
}

/** A kind of slot a ship has a number of. */
@Serializable
enum class SlotKind {
    @SerialName("high") HIGH,
    @SerialName("medium") MEDIUM,
    @SerialName("low") LOW,
    @SerialName("rig") RIG,
    @SerialName("subsystem") SUBSYSTEM,
    @SerialName("service") SERVICE,
    @SerialName("turret") TURRET,
    @SerialName("launcher") LAUNCHER,
}

/** Which items count towards a group limit. */
@Serializable
enum class GroupLimit {
    @SerialName("fitted") FITTED,
    @SerialName("online") ONLINE,
    @SerialName("active") ACTIVE,
}

/** What breaks a rule: the ship, or an item or its charge by its place in `Fit.items`. */
@Serializable
sealed interface Target {
    @Serializable @SerialName("ship") data object Ship : Target
    @Serializable @SerialName("item") data class Item(val index: Int) : Target
    @Serializable @SerialName("charge") data class Charge(val index: Int) : Target
}

/** A rule of EVE's, and the values that failed it. */
@Serializable
sealed interface Rule {
    @Serializable @SerialName("resource")
    data class Resource(val resource: fit.eveship.dogma.Resource, val used: Double, val available: Double) : Rule

    @Serializable @SerialName("slots")
    data class Slots(val slot: SlotKind, val used: Int, val available: Int) : Rule

    @Serializable @SerialName("wrong_slot")
    data class WrongSlot(val expected: SlotKind) : Rule

    @Serializable @SerialName("wrong_slot_index")
    data class WrongSlotIndex(val expected: Int) : Rule

    @Serializable @SerialName("skill")
    data class Skill(val typeId: Int, val required: Int, val level: Int) : Rule

    @Serializable @SerialName("not_a_skill")
    data class NotASkill(val typeId: Int) : Rule

    @Serializable @SerialName("rig_size")
    data class RigSize(val ship: Int, val item: Int) : Rule

    @Serializable @SerialName("max_group")
    data class MaxGroup(val groupId: Int, val limit: GroupLimit, val used: Int, val allowed: Int) : Rule

    @Serializable @SerialName("max_type")
    data class MaxType(val typeId: Int, val used: Int, val allowed: Int) : Rule

    @Serializable @SerialName("charge_size")
    data class ChargeSize(val module: Int, val charge: Int) : Rule

    @Serializable @SerialName("not_a_drone") data object NotADrone : Rule
    @Serializable @SerialName("not_a_fighter") data object NotAFighter : Rule
    @Serializable @SerialName("slot_taken") data object SlotTaken : Rule
    @Serializable @SerialName("subsystem_taken") data object SubsystemTaken : Rule
    @Serializable @SerialName("ship_restricted") data object ShipRestricted : Rule
    @Serializable @SerialName("capital_item") data object CapitalItem : Rule
    @Serializable @SerialName("structure_item") data object StructureItem : Rule
    @Serializable @SerialName("ship_item") data object ShipItem : Rule
    @Serializable @SerialName("not_a_hull") data object NotAHull : Rule
    @Serializable @SerialName("not_a_mode") data object NotAMode : Rule
    @Serializable @SerialName("charge_group") data object ChargeGroup : Rule
}

/** One rule the fit breaks, and what breaks it. */
@Serializable
data class Violation(val target: Target, val rule: Rule)
