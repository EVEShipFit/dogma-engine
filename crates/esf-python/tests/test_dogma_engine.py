import os
from pathlib import Path

import pytest

import esf_dogma_engine as dogma
from esf_dogma_engine.types import Character, Environment, FitItem

SDE = Path(os.environ.get("ESF_SDE", Path(__file__).parents[3] / "node_modules/@eveshipfit/sde/dist/sde.dat"))
NAMES = Path(os.environ.get("ESF_NAMES", Path(__file__).parents[3] / "node_modules/@eveshipfit/sde/dist/names.dat"))

RIFTER = 587
AUTOCANNON = 2873
AUTOCANNON_200MM = 486
EMP_S = 185
REACTIVE_ARMOR_HARDENER = 4403
GUNNERY = 3300
MINMATAR_FRIGATE = 3329
STRUCTURE_HP = 9
RATE_OF_FIRE = 51
MAX_VELOCITY = 37
ARMOR_KINETIC_RESONANCE = 269


@pytest.fixture(scope="session", autouse=True)
def sde() -> None:
    dogma.load_sde_from_file(SDE)
    dogma.load_names_from_file(NAMES)


def fit(
    *,
    items: list[FitItem] | None = None,
    character: Character | None = None,
    environment: Environment | None = None,
    incoming: dogma.Projection | None = None,
) -> dogma.Fit:
    """A Rifter with one autocannon, plus whatever else is asked for."""
    fit: dogma.Fit = {
        "ship": {"type_id": RIFTER},
        "items": [
            {"type_id": AUTOCANNON, "slot": {"type": "high", "index": 0}, "state": "active"},
            *(items or []),
        ],
    }
    if character is not None:
        fit["character"] = character
    if environment is not None:
        fit["environment"] = environment
    if incoming is not None:
        fit["incoming"] = incoming
    return fit


def test_calculates_a_fit() -> None:
    calculation = dogma.calculate(fit())

    assert calculation["ship"]["attributes"][STRUCTURE_HP]["value"] > 0
    assert calculation["items"][0]["state"] == "active"
    assert len(calculation["items"]) == 1


def test_attribute_keys_are_integers() -> None:
    calculation = dogma.calculate(fit())

    assert all(isinstance(key, int) for key in calculation["ship"]["attributes"])


def rate_of_fire(calculation: dogma.Calculation) -> float:
    return calculation["items"][0]["attributes"][RATE_OF_FIRE]["value"]


def test_skills_take_integer_keys() -> None:
    trained = dogma.calculate(fit(character={"skills": {GUNNERY: 5}}))
    untrained = dogma.calculate(fit(character={"skills": {GUNNERY: 0}}))

    assert rate_of_fire(trained) < rate_of_fire(untrained)


def test_skills_still_take_string_keys() -> None:
    """The WASM and JSON callers hand over string keys."""
    strings = dogma.calculate(fit(character={"skills": {"3300": 5}}))  # type: ignore[arg-type]
    integers = dogma.calculate(fit(character={"skills": {GUNNERY: 5}}))

    assert rate_of_fire(strings) == rate_of_fire(integers)


def test_outgoing_feeds_back_into_incoming() -> None:
    webifier = dogma.calculate(
        {
            "ship": {"type_id": RIFTER},
            "items": [
                {"type_id": 527, "slot": {"type": "medium", "index": 0}, "state": "active"}
            ],
        }
    )
    outgoing = webifier["outgoing"]
    assert outgoing["effects"]

    webbed = dogma.calculate(fit(incoming=outgoing))
    unwebbed = dogma.calculate(fit())

    assert (
        webbed["ship"]["attributes"][MAX_VELOCITY]["value"]
        < unwebbed["ship"]["attributes"][MAX_VELOCITY]["value"]
    )


def test_sources_are_only_reported_when_asked_for() -> None:
    skills: dogma.Fit = fit(character={"skills": {GUNNERY: 5}})

    without = dogma.calculate(skills)
    with_sources = dogma.calculate(skills, {"sources": True})

    assert "sources" not in without["items"][0]["attributes"][RATE_OF_FIRE]

    sources = with_sources["items"][0]["attributes"][RATE_OF_FIRE]["sources"]
    assert {"type": "skill", "type_id": GUNNERY} in [source["from"] for source in sources]


def test_reactive_armor_takes_a_profile_of_its_own() -> None:
    hardener: FitItem = {
        "type_id": REACTIVE_ARMOR_HARDENER,
        "slot": {"type": "low", "index": 0},
        "state": "active",
    }

    inert = dogma.calculate(fit(items=[hardener]))
    adapting = dogma.calculate(
        fit(items=[hardener], environment={"reactive_armor": {"profile": {"kinetic": 1.0}}})
    )

    assert (
        adapting["ship"]["attributes"][ARMOR_KINETIC_RESONANCE]["value"]
        < inert["ship"]["attributes"][ARMOR_KINETIC_RESONANCE]["value"]
    )


def test_validate_reports_a_full_rack() -> None:
    extra: list[FitItem] = [
        {"type_id": AUTOCANNON, "slot": {"type": "high", "index": index}, "state": "active"}
        for index in range(1, 4)
    ]
    calculation = dogma.calculate(
        fit(items=extra, character={"skills": {}}), {"validate": True}
    )
    violations = calculation["violations"]

    assert {
        "target": {"type": "ship"},
        "rule": {"type": "slots", "slot": "high", "used": 4, "available": 3},
    } in violations


def test_validate_reports_a_missing_skill() -> None:
    calculation = dogma.calculate(fit(character={"skills": {}}), {"validate": True})
    violations = calculation["violations"]

    assert {
        "target": {"type": "ship"},
        "rule": {"type": "skill", "type_id": MINMATAR_FRIGATE, "required": 1, "level": 0},
    } in violations


def test_beacon_returns_a_projection() -> None:
    projection = dogma.beacon(RIFTER)

    assert isinstance(projection, dict)


def test_bad_fit_raises_value_error() -> None:
    with pytest.raises(ValueError):
        dogma.calculate({"ship": {"type_id": RIFTER}})  # type: ignore[typeddict-item]


def test_loading_the_sde_twice_raises() -> None:
    with pytest.raises(RuntimeError):
        dogma.load_sde(b"")


def test_loads_an_eft() -> None:
    fit = dogma.load_eft("[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n")

    assert fit["ship"]["type_id"] == RIFTER
    assert fit["items"][0]["type_id"] == AUTOCANNON_200MM
    assert fit["items"][0]["slot"] == {"type": "high", "index": 0}
    assert fit["items"][0]["charge"] == {"type_id": EMP_S}


def test_an_eft_calculates() -> None:
    fit = dogma.load_eft("[Rifter, My Rifter]\n200mm AutoCannon I\n")
    fit["character"] = {"skills": {GUNNERY: 5}}

    calculation = dogma.calculate(fit)

    assert calculation["ship"]["attributes"][STRUCTURE_HP]["value"] > 0
    assert len(calculation["items"]) == 1


def test_an_eft_matches_names_in_another_language() -> None:
    """Only `names.dat` knows these; `sde.dat` holds English alone."""
    fit = dogma.load_eft("[Rifter, Mon Rifter]\nCanon Automatique 200mm I\n")

    assert fit["items"][0]["type_id"] == AUTOCANNON_200MM


def test_saves_an_eft() -> None:
    eft = "[Rifter, My Rifter]\n200mm AutoCannon I, EMP S\n"

    assert dogma.save_eft(dogma.load_eft(eft)) == eft


def test_saving_an_eft_of_an_unknown_type_raises_value_error() -> None:
    with pytest.raises(ValueError):
        dogma.save_eft({"ship": {"type_id": -1}, "items": []})


def test_a_bad_eft_raises_value_error() -> None:
    with pytest.raises(ValueError):
        dogma.load_eft("not a fit")


def test_loading_the_names_twice_raises() -> None:
    with pytest.raises(RuntimeError):
        dogma.load_names(b"")


def test_an_esi_fitting_round_trips() -> None:
    fitting: dogma.EsiFitting = {
        "name": "Gun",
        "description": "",
        "ship_type_id": RIFTER,
        "items": [
            {"flag": "HiSlot0", "quantity": 1, "type_id": AUTOCANNON_200MM},
            {"flag": "HiSlot0", "quantity": 1, "type_id": EMP_S},
        ],
    }

    fit = dogma.load_esi_fitting(fitting)
    assert fit["items"][0]["charge"] == {"type_id": EMP_S}
    assert dogma.save_esi_fitting(fit) == fitting


def test_a_killmail_gives_the_fit_of_the_ship_that_died() -> None:
    killmail: dogma.EsiKillmail = {
        "killmail_id": 1,
        "victim": {
            "ship_type_id": RIFTER,
            "items": [
                {"flag": 27, "item_type_id": AUTOCANNON_200MM, "quantity_destroyed": 1},
                {"flag": 27, "item_type_id": EMP_S, "quantity_dropped": 1},
            ],
        },
    }

    fit = dogma.load_killmail(killmail)
    assert fit["name"] == "Killmail 1"
    assert fit["items"][0]["slot"] == {"type": "high", "index": 0}
    assert fit["items"][0]["charge"] == {"type_id": EMP_S}


def test_a_link_gives_its_fit() -> None:
    fit = dogma.load_link("v2", f"{RIFTER},Gun,\n27,{AUTOCANNON_200MM},1,{EMP_S},Active")

    assert fit["name"] == "Gun"
    assert fit["items"][0]["charge"] == {"type_id": EMP_S}


def test_a_link_of_an_unknown_version_raises_value_error() -> None:
    with pytest.raises(ValueError):
        dogma.load_link("v9", "")
