use otd_core::{
    campaign_json, catalog_json, challenges_json, daily_json, modifiers_json, presets_json,
    resolve_pack_json, stock_pack_json, strikes_json, theater_doc_json, theaters_json,
    validate_json_report, validate_pack_json, verify_replay_json, Game, Modifier,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmGame {
    inner: Game,
}

#[wasm_bindgen]
impl WasmGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame {
            inner: Game::kilo_outpost(),
        }
    }

    #[wasm_bindgen(js_name = withMap)]
    pub fn with_map(id: u8) -> WasmGame {
        WasmGame {
            inner: Game::theater(id),
        }
    }

    #[wasm_bindgen(js_name = withMatch)]
    pub fn with_match(map_id: u8, modifier_id: u8) -> WasmGame {
        WasmGame {
            inner: Game::start(map_id, Modifier::from_u8(modifier_id), None),
        }
    }

    #[wasm_bindgen(js_name = withDaily)]
    pub fn with_daily(utc_day: u32) -> WasmGame {
        WasmGame {
            inner: Game::daily(utc_day),
        }
    }

    #[wasm_bindgen(js_name = withMission)]
    pub fn with_mission(id: u8) -> Result<WasmGame, JsValue> {
        Game::mission(id)
            .map(|inner| WasmGame { inner })
            .ok_or_else(|| JsValue::from_str("Unknown mission"))
    }

    #[wasm_bindgen(js_name = withChallenge)]
    pub fn with_challenge(id: u8) -> Result<WasmGame, JsValue> {
        Game::challenge(id)
            .map(|inner| WasmGame { inner })
            .ok_or_else(|| JsValue::from_str("Unknown challenge"))
    }

    #[wasm_bindgen(js_name = withSeed)]
    pub fn with_seed(map_id: u8, modifier_id: u8, seed_hex: &str) -> WasmGame {
        let seed = parse_seed(seed_hex);
        WasmGame {
            inner: Game::start(map_id, Modifier::from_u8(modifier_id), Some(seed)),
        }
    }

    #[wasm_bindgen(js_name = fromMapJson)]
    pub fn from_map_json(json: &str, modifier_id: u8) -> Result<WasmGame, JsValue> {
        Game::from_map_json(json, Modifier::from_u8(modifier_id))
            .map(|inner| WasmGame { inner })
            .map_err(|e| JsValue::from_str(&e.message()))
    }

    #[wasm_bindgen(js_name = fromReplay)]
    pub fn from_replay(json: &str) -> Result<WasmGame, JsValue> {
        Game::from_replay_json(json)
            .map(|inner| WasmGame { inner })
            .map_err(|e| JsValue::from_str(&e.message()))
    }

    pub fn step(&mut self) {
        self.inner.step();
    }

    #[wasm_bindgen(js_name = pumpRecorded)]
    pub fn pump_recorded(&mut self) {
        self.inner.pump_recorded();
    }

    #[wasm_bindgen(js_name = stepRecorded)]
    pub fn step_recorded(&mut self) {
        self.inner.step_recorded();
    }

    pub fn snapshot(&self) -> String {
        self.inner.snapshot_json()
    }

    #[wasm_bindgen(js_name = mapStatic)]
    pub fn map_static(&self) -> String {
        self.inner.map_static_json()
    }

    pub fn catalog() -> String {
        catalog_json()
    }

    pub fn strikes() -> String {
        strikes_json()
    }

    pub fn theaters() -> String {
        theaters_json()
    }

    pub fn modifiers() -> String {
        modifiers_json()
    }

    pub fn daily(utc_day: u32) -> String {
        daily_json(utc_day)
    }

    pub fn campaign() -> String {
        campaign_json()
    }

    pub fn challenges() -> String {
        challenges_json()
    }

    #[wasm_bindgen(js_name = verifyReplay)]
    pub fn verify_replay(json: &str) -> String {
        verify_replay_json(json)
    }

    #[wasm_bindgen(js_name = validateMap)]
    pub fn validate_map(json: &str) -> String {
        validate_json_report(json)
    }

    #[wasm_bindgen(js_name = theaterDoc)]
    pub fn theater_doc(id: u8) -> String {
        theater_doc_json(id)
    }

    #[wasm_bindgen(js_name = stockPack)]
    pub fn stock_pack() -> String {
        stock_pack_json()
    }

    #[wasm_bindgen(js_name = packPresets)]
    pub fn pack_presets() -> String {
        presets_json()
    }

    #[wasm_bindgen(js_name = validatePack)]
    pub fn validate_pack(json: &str) -> String {
        validate_pack_json(json)
    }

    #[wasm_bindgen(js_name = resolvePack)]
    pub fn resolve_pack(json: &str) -> String {
        resolve_pack_json(json)
    }

    #[wasm_bindgen(js_name = applyPack)]
    pub fn apply_pack(&mut self, json: &str) -> Result<(), JsValue> {
        self.inner
            .apply_pack_json(json)
            .map_err(|e| JsValue::from_str(&e.message()))
    }

    #[wasm_bindgen(js_name = matchCatalog)]
    pub fn match_catalog(&self) -> String {
        self.inner.match_catalog_json()
    }

    #[wasm_bindgen(js_name = matchStrikes)]
    pub fn match_strikes(&self) -> String {
        self.inner.match_strikes_json()
    }

    pub fn replay(&self) -> String {
        self.inner.replay_json()
    }

    #[wasm_bindgen(js_name = setBuild)]
    pub fn set_build(&mut self, kind: u8) {
        self.inner.set_build(kind);
    }

    #[wasm_bindgen(js_name = setStrike)]
    pub fn set_strike(&mut self, kind: u8) {
        self.inner.set_strike(kind);
    }

    #[wasm_bindgen(js_name = setHover)]
    pub fn set_hover(&mut self, x: i32, y: i32) {
        self.inner.set_hover(x, y);
    }

    #[wasm_bindgen(js_name = clearHover)]
    pub fn clear_hover(&mut self) {
        self.inner.clear_hover();
    }

    pub fn cancel(&mut self) -> bool {
        self.inner.cancel()
    }

    pub fn click(&mut self, x: i32, y: i32) {
        self.inner.click(x, y);
    }

    #[wasm_bindgen(js_name = upgradeSelected)]
    pub fn upgrade_selected(&mut self) -> bool {
        self.inner.upgrade().is_ok()
    }

    #[wasm_bindgen(js_name = sellSelected)]
    pub fn sell_selected(&mut self) -> bool {
        self.inner.sell().is_ok()
    }

    #[wasm_bindgen(js_name = cycleTargeting)]
    pub fn cycle_targeting(&mut self) -> bool {
        self.inner.cycle_targeting()
    }

    #[wasm_bindgen(js_name = convertSelected)]
    pub fn convert_selected(&mut self) -> bool {
        self.inner.convert().is_ok()
    }

    #[wasm_bindgen(js_name = callWave)]
    pub fn call_wave(&mut self) -> bool {
        self.inner.call_wave()
    }

    pub fn repair(&mut self) -> bool {
        self.inner.repair().is_ok()
    }

    #[wasm_bindgen(js_name = liftSelected)]
    pub fn lift_selected(&mut self) -> bool {
        self.inner.lift().is_ok()
    }

    pub fn overcharge(&mut self) -> bool {
        self.inner.overcharge().is_ok()
    }
}

fn parse_seed(raw: &str) -> u64 {
    let t = raw.trim().trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(t, 16)
        .or_else(|_| t.parse::<u64>())
        .unwrap_or(1)
}

impl Default for WasmGame {
    fn default() -> Self {
        Self::new()
    }
}
