//! The MCP tools and resources.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use esf_data::fold_case;
use esf_dogma_engine::{Calculation, Fit, ItemResult, SourceRef};
use esf_format::{eft, esi};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ListResourcesResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult, Resource,
    ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::fits::{Settings, label, slot_label};
use crate::game::Game;
use crate::stats::differences;

const SPEC: &str = include_str!("../esf-spec.md");
const SPEC_URI: &str = "esf://spec";

const INSTRUCTIONS: &str = "Calculates EVE Online ship fits with the EVEShip.fit dogma engine. \
Write fits in esf/1 (described in calculate_fit, full spec in the resource esf://spec); EFT, DNA \
and ESI fitting JSON work too. Every tool takes the fit as text and remembers nothing between \
calls. Skills default to all level 5. Use find_type when unsure of an exact item name.";

const CALCULATE: &str = r#"Calculate a fit: DPS, volley, EHP, resists, capacitor, speed, align time, targeting and fitting use, plus every rule the fit breaks (CPU, powergrid, slots, skills, ...). Also returns the fit as canonical esf/1.

Write fits in esf/1, one item per line:

%esf/1
Rifter "Shield Buffer"
3x 200mm AutoCannon II :Republic Fleet EMP S
1MN Afterburner II
Warp Scrambler II
Medium Shield Extender II
Damage Control II !off
2x Small Projectile Collision Accelerator I
2x Warrior II
1000x Republic Fleet EMP S @cargo

- First the line `%esf/1`, then the hull with an optional "name". A tactical mode goes on the hull line: `Hecate /Sharpshooter`.
- Then one line per item, in any order. Each item goes in its own rack; there are no slot sections, and empty slots need no line.
- `3x`: on modules, three of them; on a drone line, that many drones in space; in cargo or a bay, a stack.
- `:Name`: the charge loaded in the module.
- `!off`, `!on` (online, not running), `!heat` (overloaded). Default: running if it can be, else online.
- `@cargo`, `@bay` (drone or fighter bay, not launched), or another hold.
- `+Mutaplasmid {attributeName value, ...}`: an abyssal module.
- `// comment` until the end of the line.

EFT (as EVE copies it), DNA and ESI fitting JSON are read as well."#;

#[derive(Deserialize, JsonSchema)]
pub struct FitParams {
    /// The fit, in esf/1 (preferred), EFT, DNA or ESI fitting JSON.
    pub fit: String,
    #[serde(flatten)]
    pub settings: Settings,
}

#[derive(Deserialize, JsonSchema)]
pub struct AttributesParams {
    /// The fit, in the same formats as calculate_fit.
    pub fit: String,
    /// What to read: "ship" (default), "character", "mode", a slot like "high 0", or the name of an item or charge in the fit.
    pub target: Option<String>,
    /// Only attributes whose name holds one of these, ignoring case, like ["falloff", "optimal"]. Default: every published attribute.
    #[serde(default)]
    pub attributes: Vec<String>,
    /// Only attributes that skills, modules or effects changed.
    #[serde(default)]
    pub changed_only: bool,
    /// Also list the modifiers each value was calculated from.
    #[serde(default)]
    pub sources: bool,
    #[serde(flatten)]
    pub settings: Settings,
}

#[derive(Deserialize, JsonSchema)]
pub struct FindTypeParams {
    /// Words the name holds, ignoring case, like "200mm auto ii".
    pub query: String,
    /// Only this category, like "Ship", "Module", "Charge", "Drone", "Implant", "Skill", "Fighter" or "Subsystem".
    pub category: Option<String>,
    /// At most this many results. Default 20.
    pub limit: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct VariationsParams {
    /// Name of an item, like "Warp Scrambler II".
    pub item: String,
    /// A fit holding the item (or one of its variants). With it, each variant is swapped into the fit and recalculated.
    pub fit: Option<String>,
    #[serde(flatten)]
    pub settings: Settings,
}

#[derive(Deserialize, JsonSchema)]
pub struct CompareParams {
    /// The first fit, in the same formats as calculate_fit.
    pub fit_a: String,
    /// The second fit.
    pub fit_b: String,
    #[serde(flatten)]
    pub settings: Settings,
}

#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ConvertTarget {
    Esf,
    Eft,
    Esi,
}

#[derive(Deserialize, JsonSchema)]
pub struct ConvertParams {
    /// The fit, in esf/1, EFT, DNA, ESI fitting JSON or ESI killmail JSON.
    pub fit: String,
    /// The format to write.
    pub to: ConvertTarget,
}

#[derive(Clone)]
pub struct Server {
    game: Arc<Game>,
    tool_router: ToolRouter<Self>,
}

fn reply(result: Result<Value, String>) -> CallToolResult {
    match result {
        Ok(value) => CallToolResult::success(vec![ContentBlock::text(
            serde_json::to_string_pretty(&value).unwrap_or_default(),
        )]),
        Err(error) => CallToolResult::error(vec![ContentBlock::text(error)]),
    }
}

#[tool_router]
impl Server {
    pub fn new(game: Game) -> Server {
        Server {
            game: Arc::new(game),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = CALCULATE)]
    fn calculate_fit(&self, Parameters(params): Parameters<FitParams>) -> CallToolResult {
        reply(self.do_calculate(&params))
    }

    /// Read the raw dogma attributes of the ship, an item or a charge after a fit is calculated, like a gun's optimal range, tracking or cycle time. Values are shown as EVE shows them (seconds, percent, ...).
    #[tool]
    fn get_attributes(&self, Parameters(params): Parameters<AttributesParams>) -> CallToolResult {
        reply(self.do_attributes(&params))
    }

    /// Search items by name. Returns the exact name to use in a fit, with its category, group, meta group and the rack it goes in.
    #[tool]
    fn find_type(&self, Parameters(params): Parameters<FindTypeParams>) -> CallToolResult {
        if params.query.trim().is_empty() {
            return reply(Err("query is empty".to_string()));
        }
        let found = self.game.search(
            &params.query,
            params.category.as_deref(),
            params.limit.unwrap_or(20),
        );
        reply(serde_json::to_value(found).map_err(|error| error.to_string()))
    }

    /// List the variants of an item: its Tech I, Tech II, faction, deadspace and officer versions, with the attributes that set them apart. Given a fit, each variant is swapped in and the fit recalculated, showing how the stats change and what rules it would break.
    #[tool]
    fn find_variations(&self, Parameters(params): Parameters<VariationsParams>) -> CallToolResult {
        reply(self.do_variations(&params))
    }

    /// Compare two fits: every stat that differs, the items only one of them has, and the rules each breaks.
    #[tool]
    fn compare_fits(&self, Parameters(params): Parameters<CompareParams>) -> CallToolResult {
        reply(self.do_compare(&params))
    }

    /// Convert a fit to esf/1, EFT or ESI fitting JSON.
    #[tool]
    fn convert_fit(&self, Parameters(params): Parameters<ConvertParams>) -> CallToolResult {
        let game = &self.game;
        let converted =
            game.load_fit(&params.fit, &Settings::default())
                .and_then(|fit| match params.to {
                    ConvertTarget::Esf => game.to_esf(&fit),
                    ConvertTarget::Eft => {
                        eft::save_eft(&game.info, &fit).map_err(|error| error.to_string())
                    }
                    ConvertTarget::Esi => {
                        serde_json::to_string_pretty(&esi::save_esi_fitting(&game.info, &fit))
                            .map_err(|error| error.to_string())
                    }
                });
        match converted {
            Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
            Err(error) => CallToolResult::error(vec![ContentBlock::text(error)]),
        }
    }
}

impl Server {
    fn do_calculate(&self, params: &FitParams) -> Result<Value, String> {
        let game = &self.game;
        let fit = game.load_fit(&params.fit, &params.settings)?;
        let calculation = game.calculate(&fit, false);
        Ok(json!({
            "name": fit.name,
            "ship": game.type_name(fit.ship.type_id),
            "mode": fit.ship.mode.map(|mode| game.type_name(mode)),
            "stats": game.stats(&fit, &calculation),
            "violations": game.violations(&fit, &calculation),
            "items": game.items(&fit, &calculation),
            "esf": game.to_esf(&fit).unwrap_or_else(|error| format!("cannot be written as esf/1: {error}")),
        }))
    }

    fn do_attributes(&self, params: &AttributesParams) -> Result<Value, String> {
        let game = &self.game;
        let fit = game.load_fit(&params.fit, &params.settings)?;
        let calculation = game.calculate(&fit, params.sources);
        let (target, result) = find_target(game, &fit, &calculation, params.target.as_deref())?;

        let filters: Vec<String> = params
            .attributes
            .iter()
            .map(|text| fold_case(text))
            .collect();
        let attributes: Vec<Value> = result
            .attributes
            .iter()
            .filter(|(attribute_id, attribute)| {
                if params.changed_only && attribute.value == attribute.base {
                    return false;
                }
                if !game.attribute_published(**attribute_id)
                    && (filters.is_empty() || attribute.value == 0.0 && attribute.base == 0.0)
                {
                    return false;
                }
                if filters.is_empty() {
                    return true;
                }
                let name = fold_case(&game.attribute_name(**attribute_id));
                let display = fold_case(game.attribute_display_name(**attribute_id).unwrap_or_default());
                filters
                    .iter()
                    .any(|filter| name.contains(filter) || display.contains(filter))
            })
            .map(|(attribute_id, attribute)| {
                let (value, unit) = game.display(*attribute_id, attribute.value);
                let (base, _) = game.display(*attribute_id, attribute.base);
                let mut entry = json!({
                    "name": game.attribute_name(*attribute_id),
                    "value": value,
                });
                if let Some(display) = game.attribute_display_name(*attribute_id) {
                    entry["display_name"] = json!(display);
                }
                if !unit.is_empty() {
                    entry["unit"] = json!(unit);
                }
                if base != value {
                    entry["base"] = json!(base);
                }
                if params.sources && !attribute.sources.is_empty() {
                    entry["sources"] = attribute
                        .sources
                        .iter()
                        .map(|source| {
                            json!({
                                "from": source_label(game, &fit, source.from),
                                "effect": source.effect_id.map(|effect_id| game.effect_name(effect_id)),
                                "operator": label(source.operator),
                                "value": source.value,
                                "quantity": source.quantity,
                                "stacking_penalty": source.penalty,
                                "applied": source.applied,
                            })
                        })
                        .collect();
                }
                entry
            })
            .collect();
        Ok(json!({"target": target, "attributes": attributes}))
    }

    fn do_variations(&self, params: &VariationsParams) -> Result<Value, String> {
        let game = &self.game;
        let type_id = game.require_type_id(&params.item)?;
        let mut family = game.variations(type_id);
        family.sort_by(|left, right| {
            let left_level = game.meta_level(*left).unwrap_or_default();
            let right_level = game.meta_level(*right).unwrap_or_default();
            left_level
                .total_cmp(&right_level)
                .then_with(|| game.type_name(*left).cmp(&game.type_name(*right)))
        });

        let describe = |variant: i32| {
            let summary = game.summarize_type(variant);
            json!({
                "name": summary.name,
                "meta_group": summary.meta_group,
                "meta_level": game.meta_level(variant),
            })
        };

        let Some(text) = &params.fit else {
            let bases: Vec<BTreeMap<i32, f64>> = family
                .iter()
                .map(|variant| {
                    game.info_attributes(*variant)
                        .into_iter()
                        .filter(|(attribute_id, _)| game.attribute_published(*attribute_id))
                        .collect()
                })
                .collect();
            let all: BTreeSet<i32> = bases.iter().flat_map(|base| base.keys().copied()).collect();
            let differing: Vec<i32> = all
                .into_iter()
                .filter(|attribute_id| {
                    let first = bases[0].get(attribute_id);
                    bases.iter().any(|base| base.get(attribute_id) != first)
                })
                .collect();

            let variants: Vec<Value> = family
                .iter()
                .zip(&bases)
                .map(|(variant, base)| {
                    let mut entry = describe(*variant);
                    let attributes: serde_json::Map<String, Value> = differing
                        .iter()
                        .filter_map(|attribute_id| {
                            let value = base.get(attribute_id)?;
                            Some((
                                game.attribute_name(*attribute_id),
                                game.display_value(*attribute_id, *value),
                            ))
                        })
                        .collect();
                    entry["attributes"] = Value::Object(attributes);
                    entry
                })
                .collect();
            return Ok(json!({"item": game.type_name(type_id), "variants": variants}));
        };

        let fit = game.load_fit(text, &params.settings)?;
        let mut swapped = spots(&fit, |candidate| candidate == type_id);
        if swapped.is_empty() {
            swapped = spots(&fit, |candidate| family.contains(&candidate));
        }
        let Some(first) = swapped.first() else {
            return Err(format!(
                "the fit holds no {} or a variant of it, as hull, item or charge",
                game.type_name(type_id)
            ));
        };
        let current = first.type_id(&fit);

        let calculation = game.calculate(&fit, false);
        let before = game.stats(&fit, &calculation);
        let before_violations: BTreeSet<String> = calculation
            .violations
            .iter()
            .flatten()
            .map(|violation| game.violation_kind(violation))
            .collect();

        let variants: Vec<Value> = family
            .iter()
            .filter(|variant| **variant != current)
            .map(|variant| {
                let mut variant_fit = fit.clone();
                for spot in &swapped {
                    spot.swap(&mut variant_fit, *variant);
                }
                let calculation = game.calculate(&variant_fit, false);
                let after = game.stats(&variant_fit, &calculation);
                let new_violations: Vec<String> = calculation
                    .violations
                    .iter()
                    .flatten()
                    .filter(|violation| {
                        !before_violations.contains(&game.violation_kind(violation))
                    })
                    .map(|violation| game.violation(&variant_fit, violation))
                    .collect();
                let mut entry = describe(*variant);
                entry["changes"] = json!(differences(&before, &after));
                entry["new_violations"] = json!(new_violations);
                entry
            })
            .collect();

        Ok(json!({
            "current": describe(current),
            "swapped": swapped.iter().map(|spot| spot.label(&fit)).collect::<Vec<_>>(),
            "note": "In changes, a is the fit as given, b with the variant.",
            "variants": variants,
        }))
    }

    fn do_compare(&self, params: &CompareParams) -> Result<Value, String> {
        let game = &self.game;
        let fit_a = game
            .load_fit(&params.fit_a, &params.settings)
            .map_err(|error| format!("fit_a: {error}"))?;
        let fit_b = game
            .load_fit(&params.fit_b, &params.settings)
            .map_err(|error| format!("fit_b: {error}"))?;
        let calculation_a = game.calculate(&fit_a, false);
        let calculation_b = game.calculate(&fit_b, false);

        let mut ship_a = game.stats(&fit_a, &calculation_a);
        let mut ship_b = game.stats(&fit_b, &calculation_b);
        ship_a["ship"] = json!(game.type_name(fit_a.ship.type_id));
        ship_b["ship"] = json!(game.type_name(fit_b.ship.type_id));

        let count = |fit: &Fit| {
            let mut counts: BTreeMap<String, i64> = BTreeMap::new();
            for item in &fit.items {
                *counts.entry(game.type_name(item.type_id)).or_default() +=
                    i64::from(item.quantity);
                if let Some(charge) = &item.charge {
                    *counts
                        .entry(format!("{} (charge)", game.type_name(charge.type_id)))
                        .or_default() += 1;
                }
            }
            counts
        };
        let counts_a = count(&fit_a);
        let counts_b = count(&fit_b);
        let only = |left: &BTreeMap<String, i64>,
                    right: &BTreeMap<String, i64>|
         -> BTreeMap<String, i64> {
            left.iter()
                .filter_map(|(name, count)| {
                    let extra = count - right.get(name).copied().unwrap_or_default();
                    (extra > 0).then(|| (name.clone(), extra))
                })
                .collect()
        };

        Ok(json!({
            "stats": differences(&ship_a, &ship_b),
            "only_in_a": only(&counts_a, &counts_b),
            "only_in_b": only(&counts_b, &counts_a),
            "violations_a": game.violations(&fit_a, &calculation_a),
            "violations_b": game.violations(&fit_b, &calculation_b),
        }))
    }
}

/// Where a type sits in a fit.
#[derive(Clone, Copy)]
enum Spot {
    Hull,
    Item(usize),
    Charge(usize),
}

impl Spot {
    fn type_id(self, fit: &Fit) -> i32 {
        match self {
            Spot::Hull => fit.ship.type_id,
            Spot::Item(index) => fit.items[index].type_id,
            Spot::Charge(index) => fit.items[index]
                .charge
                .as_ref()
                .map_or(0, |charge| charge.type_id),
        }
    }

    fn swap(self, fit: &mut Fit, type_id: i32) {
        match self {
            Spot::Hull => {
                fit.ship.type_id = type_id;
                fit.ship.mode = None;
            }
            Spot::Item(index) => {
                fit.items[index].type_id = type_id;
                fit.items[index].mutation = None;
            }
            Spot::Charge(index) => {
                if let Some(charge) = &mut fit.items[index].charge {
                    charge.type_id = type_id;
                }
            }
        }
    }

    fn label(self, fit: &Fit) -> String {
        match self {
            Spot::Hull => "hull".to_string(),
            Spot::Item(index) => slot_label(fit.items[index].slot),
            Spot::Charge(index) => format!("charge of {}", slot_label(fit.items[index].slot)),
        }
    }
}

/// Every spot in a fit whose type passes `matches`.
fn spots(fit: &Fit, matches: impl Fn(i32) -> bool) -> Vec<Spot> {
    let mut found = Vec::new();
    if matches(fit.ship.type_id) {
        found.push(Spot::Hull);
    }
    for (index, item) in fit.items.iter().enumerate() {
        if matches(item.type_id) {
            found.push(Spot::Item(index));
        }
        if item
            .charge
            .as_ref()
            .is_some_and(|charge| matches(charge.type_id))
        {
            found.push(Spot::Charge(index));
        }
    }
    found
}

fn find_target<'a>(
    game: &Game,
    fit: &Fit,
    calculation: &'a Calculation,
    target: Option<&str>,
) -> Result<(String, &'a ItemResult), String> {
    let original = target.unwrap_or("ship").trim();
    let target = fold_case(original);
    match target.as_str() {
        "ship" | "" => {
            return Ok((
                format!("ship: {}", game.type_name(fit.ship.type_id)),
                &calculation.ship,
            ));
        }
        "character" => return Ok(("character".to_string(), &calculation.character)),
        "mode" => {
            let result = calculation
                .mode
                .as_ref()
                .ok_or("the fit has no tactical mode")?;
            let mode = fit
                .ship
                .mode
                .map(|mode| game.type_name(mode))
                .unwrap_or_default();
            return Ok((format!("mode: {mode}"), result));
        }
        _ => {}
    }

    for (index, (item, result)) in fit.items.iter().zip(&calculation.items).enumerate() {
        if fold_case(&slot_label(item.slot)) == target
            || fold_case(&game.type_name(item.type_id)) == target
        {
            return Ok((game.item_label(fit, index), result));
        }
    }
    for (index, (item, result)) in fit.items.iter().zip(&calculation.items).enumerate() {
        if let (Some(charge), Some(charge_result)) = (&item.charge, &result.charge)
            && fold_case(&game.type_name(charge.type_id)) == target
        {
            return Ok((
                format!("charge of {}", game.item_label(fit, index)),
                charge_result,
            ));
        }
    }
    Err(format!(
        "nothing in the fit is \"{original}\"; use \"ship\", \"character\", a slot like \"high 0\", or an item's name"
    ))
}

fn source_label(game: &Game, fit: &Fit, from: SourceRef) -> String {
    match from {
        SourceRef::Ship => game.type_name(fit.ship.type_id),
        SourceRef::Mode => fit
            .ship
            .mode
            .map_or_else(|| "mode".to_string(), |mode| game.type_name(mode)),
        SourceRef::Character => "character".to_string(),
        SourceRef::Item { index } => game.item_label(fit, index),
        SourceRef::Charge { index } => format!("charge of {}", game.item_label(fit, index)),
        SourceRef::Skill { type_id } => format!("skill {}", game.type_name(type_id)),
        SourceRef::Projected { index } => format!(
            "projected by {}",
            game.type_name(fit.incoming.effects[index].type_id)
        ),
        SourceRef::Buff { id } => format!("buff {}", game.buff_name(id)),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(INSTRUCTIONS)
        .with_server_info(Implementation::new("esf-mcp", env!("CARGO_PKG_VERSION")))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let resource = Resource::new(SPEC_URI, "esf-spec")
            .with_title("esf/1 specification")
            .with_description(
                "The full esf/1 fit format: every token, placement rule and the canonical form.",
            )
            .with_mime_type("text/markdown");
        Ok(ListResourcesResult::with_all_items(vec![resource]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != SPEC_URI {
            return Err(ErrorData::resource_not_found(
                format!("no resource {}", request.uri),
                None,
            ));
        }
        let contents = ResourceContents::text(SPEC, SPEC_URI).with_mime_type("text/markdown");
        Ok(ReadResourceResult::new(vec![contents]).into())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    const RIFTER: &str = "%esf/1
Rifter \"Shield Buffer\"
3x 200mm AutoCannon II :Republic Fleet EMP S
1MN Afterburner II
Warp Scrambler II
Medium Shield Extender II
Damage Control II
Gyrostabilizer II
";

    const RIFTER_EFT: &str = "[Rifter, Armor]
Damage Control II
Small Armor Repairer II

1MN Afterburner II
Warp Scrambler II
Stasis Webifier II

200mm AutoCannon II, Republic Fleet EMP S
200mm AutoCannon II, Republic Fleet EMP S
200mm AutoCannon II, Republic Fleet EMP S
";

    fn read(file: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../node_modules/@eveshipfit/sde/dist/{file}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(&path).unwrap_or_else(|error| panic!("cannot read {path}: {error}"))
    }

    static SERVER: LazyLock<Server> =
        LazyLock::new(|| Server::new(Game::new(read("sde.dat"), Some(read("names.dat"))).unwrap()));

    fn fit(text: &str) -> FitParams {
        FitParams {
            fit: text.to_string(),
            settings: Settings::default(),
        }
    }

    #[test]
    fn calculates() {
        let result = SERVER.do_calculate(&fit(RIFTER)).unwrap();
        assert_eq!(result["ship"], "Rifter");
        assert!(result["stats"]["offense"]["dps"].as_f64().unwrap() > 100.0);
        assert!(result["stats"]["defense"]["ehp"].as_f64().unwrap() > 1000.0);
        assert_eq!(result["stats"]["fitting"]["slots"]["high"]["used"], 3);
        assert!(result["esf"].as_str().unwrap().starts_with("%esf/1\n"));
    }

    #[test]
    fn suggests_names() {
        let error = SERVER
            .do_calculate(&fit("Riftr\n200mm AutoCannon II\n"))
            .unwrap_err();
        assert!(error.starts_with("esf/1: line 1:"), "{error}");
        assert!(error.contains("did you mean: Rifter"), "{error}");
    }

    #[test]
    fn reads_attributes() {
        let params = AttributesParams {
            fit: RIFTER.to_string(),
            target: Some("200mm autocannon ii".to_string()),
            attributes: vec!["falloff".to_string()],
            changed_only: false,
            sources: true,
            settings: Settings::default(),
        };
        let result = SERVER.do_attributes(&params).unwrap();
        assert_eq!(result["target"], "high 0: 200mm AutoCannon II");
        let falloff = result["attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|attribute| attribute["name"] == "falloff")
            .unwrap();
        assert_eq!(falloff["unit"], "m");
        assert!(!falloff["sources"].as_array().unwrap().is_empty());
    }

    #[test]
    fn finds_types() {
        let found = SERVER.game.search("scram", Some("Module"), 20);
        assert!(found.iter().any(|found| found.name == "Warp Scrambler II"));
        assert!(
            SERVER
                .game
                .search("200mm auto", None, 20)
                .iter()
                .all(|found| found.category != "Blueprint")
        );
    }

    #[test]
    fn lists_variations() {
        let params = VariationsParams {
            item: "Warp Scrambler II".to_string(),
            fit: None,
            settings: Settings::default(),
        };
        let result = SERVER.do_variations(&params).unwrap();
        let names: Vec<&str> = result["variants"]
            .as_array()
            .unwrap()
            .iter()
            .map(|variant| variant["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Warp Scrambler I"));
        assert!(names.contains(&"Warp Scrambler II"));
    }

    #[test]
    fn swaps_variations_into_a_fit() {
        for (item, spot) in [
            ("Gyrostabilizer II", "low 1"),
            ("Republic Fleet EMP S", "charge of high 0"),
            ("Rifter", "hull"),
        ] {
            let params = VariationsParams {
                item: item.to_string(),
                fit: Some(RIFTER.to_string()),
                settings: Settings::default(),
            };
            let result = SERVER.do_variations(&params).unwrap();
            assert_eq!(result["swapped"][0], spot, "{item}");
            assert!(!result["variants"].as_array().unwrap().is_empty(), "{item}");
        }
    }

    #[test]
    fn compares() {
        let params = CompareParams {
            fit_a: RIFTER.to_string(),
            fit_b: RIFTER_EFT.to_string(),
            settings: Settings::default(),
        };
        let result = SERVER.do_compare(&params).unwrap();
        assert_eq!(result["only_in_a"]["Medium Shield Extender II"], 1);
        assert_eq!(result["only_in_b"]["Stasis Webifier II"], 1);
        assert!(
            result["stats"]
                .as_array()
                .unwrap()
                .iter()
                .any(|stat| stat["stat"] == "defense.ehp")
        );
    }
}
