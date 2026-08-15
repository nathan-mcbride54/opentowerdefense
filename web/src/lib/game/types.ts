export type BuildKind =
	| 'inspect'
	| 'barricade'
	| 'autocannon'
	| 'howitzer'
	| 'skystinger'
	| 'inferno'
	| 'arcLance'
	| 'pulseArray'
	| 'helios'
	| 'swarmRack'
	| 'siegeRail';

export type CreepKind =
	| 'runner'
	| 'lorry'
	| 'bulwark'
	| 'wasp'
	| 'colossus'
	| 'mite'
	| 'medic'
	| 'shade'
	| 'flicker';
export type MatchStatus = 'fortify' | 'incoming' | 'defeat';
export type FireMode = 'shell' | 'cone' | 'line' | 'pulse' | 'beam';
export type TargetMode = 'first' | 'last' | 'strong' | 'weak' | 'flying' | 'camo';

export interface CatalogItem {
	id: number;
	hotkey: string;
	name: string;
	role: string;
	blurb: string;
	cost: number;
	range: number;
	hitsGround: boolean;
	hitsAir: boolean;
	detects: boolean;
	fire: FireMode;
}

export interface StrikeItem {
	id: number;
	hotkey: string;
	name: string;
	blurb: string;
	cost: number;
	radius: number;
	cooldown: number;
}

export interface TheaterInfo {
	id: number;
	slug: string;
	name: string;
	blurb: string;
	hazard: string;
}

export interface ModifierInfo {
	id: number;
	slug: string;
	name: string;
	blurb: string;
	hazard: string;
}

export interface DailyPick {
	utcDay: number;
	mapId: number;
	modifierId: number;
	mapName: string;
	mapHazard: string;
	modifierName: string;
	modifierHazard: string;
	seed: number;
}

export interface MapDoc {
	slug: string;
	name: string;
	blurb: string;
	hazard: string;
	w: number;
	h: number;
	seed: number;
	cores: [number, number][];
	spawns: [number, number][];
	rocks: [number, number][];
}

export interface MapStatic {
	w: number;
	h: number;
	id: number;
	name: string;
	core: [number, number][];
	spawns: [number, number][];
	rocks: [number, number][];
}

export interface HoverInfo {
	x: number;
	y: number;
	valid: boolean;
	reason: string;
	range: number;
	hitsGround: boolean;
	hitsAir: boolean;
	strike: boolean;
	walkAfter: number | null;
}

export interface SelectedInfo {
	id: number;
	kind: BuildKind;
	name: string;
	x: number;
	y: number;
	tier: number;
	maxTier: number;
	tierName: string;
	range: number;
	damage: number;
	fireInterval: number;
	splash: number;
	hitsGround: boolean;
	hitsAir: boolean;
	detects: boolean;
	fire: FireMode;
	targeting: TargetMode;
	targetingLabel: string;
	canConvert: boolean;
	convertCost: number | null;
	invested: number;
	upgradeCost: number | null;
	sellValue: number;
	kills: number;
	damageDealt: number;
}

export interface TowerView {
	id: number;
	kind: BuildKind;
	x: number;
	y: number;
	aim: number;
	tier: number;
	airFocus: boolean;
	stunned: boolean;
	overcharged: boolean;
}

export interface CreepView {
	id: number;
	kind: CreepKind;
	x: number;
	y: number;
	hp: number;
	hpMax: number;
	flying: boolean;
	heading: number;
	radius: number;
	slowed: boolean;
}

export interface ProjView {
	x: number;
	y: number;
	vx: number;
	vy: number;
	kind: BuildKind;
}

export interface FxView {
	kind: string;
	x: number;
	y: number;
	life: number;
	mag: number;
	heading: number;
}

export interface BeamView {
	x0: number;
	y0: number;
	x1: number;
	y1: number;
	kind: BuildKind;
	life: number;
}

export interface StrikeHud {
	id: number;
	ready: boolean;
	cooldown: number;
	cost: number;
}

export interface Snapshot {
	tick: number;
	time: number;
	status: MatchStatus;
	defeated: boolean;
	credits: number;
	integrity: number;
	integrityMax: number;
	wave: number;
	nextWaveIn: number;
	canCallWave: boolean;
	creepsAlive: number;
	creepsRemaining: number;
	kills: number;
	leaks: number;
	banner: string | null;
	bannerLife: number;
	message: string | null;
	hurtFlash: number;
	build: number;
	strike: number;
	mapId: number;
	mapName: string;
	modifierId: number;
	modifierName: string;
	turretCount: number;
	turretCap: number | null;
	hover: HoverInfo | null;
	selected: SelectedInfo | null;
	strikes: StrikeHud[];
	walls: [number, number][];
	towers: TowerView[];
	creeps: CreepView[];
	projectiles: ProjView[];
	fx: FxView[];
	beams: BeamView[];
	core: [number, number];
	cores: [number, number][];
	objectiveWave: number | null;
	objectiveCleared: boolean;
	missionId: number | null;
	challengeId: number | null;
	missionName: string | null;
	seedHex: string;
	packName: string | null;
	build2: number;
	strike2: number;
	hover2: HoverInfo | null;
	selected2: SelectedInfo | null;
	waveIntel: WaveIntel;
	after: AfterAction;
	interestPaid: number;
	interestBps: number;
	walk: number;
	relocating: boolean;
	relocating2: boolean;
	walkPaths: [number, number][][];
}

export interface KindCount {
	kind: CreepKind;
	name: string;
	count: number;
}

export interface WaveIntel {
	script: string;
	total: number;
	parts: KindCount[];
}

export interface GunScore {
	name: string;
	kills: number;
	damage: number;
}

export interface AfterAction {
	spent: number;
	kills: number;
	leaks: number;
	wave: number;
	killKinds: KindCount[];
	leakKinds: KindCount[];
	guns: GunScore[];
}

export interface VerifyReport {
	ok: boolean;
	hash: string;
	hashOk: boolean;
	outcomeOk: boolean;
	error: string | null;
	wave: number;
	integrity: number;
	kills: number;
	leaks: number;
	defeated: boolean;
	ticks: number;
}

export interface GunPatch {
	id: number;
	name?: string | null;
	role?: string | null;
	blurb?: string | null;
	enabled?: boolean | null;
	cost?: number | null;
	range?: number | null;
	fireInterval?: number | null;
	damage?: number | null;
	splash?: number | null;
	projSpeed?: number | null;
	hitsGround?: boolean | null;
	hitsAir?: boolean | null;
	homing?: boolean | null;
	fire?: FireMode | null;
	volley?: number | null;
	slow?: number | null;
	slowTtl?: number | null;
}

export interface StrikePatch {
	id: number;
	name?: string | null;
	blurb?: string | null;
	enabled?: boolean | null;
	cost?: number | null;
	radius?: number | null;
	damage?: number | null;
	slow?: number | null;
	slowTtl?: number | null;
	cooldown?: number | null;
	hitsGround?: boolean | null;
	hitsAir?: boolean | null;
}

export interface PackDoc {
	slug: string;
	name: string;
	blurb: string;
	guns: GunPatch[];
	strikes: StrikePatch[];
}

export interface MissionInfo {
	id: number;
	slug: string;
	name: string;
	briefing: string;
	objective: string;
	holdUntilWave: number;
	mapId: number;
	mapName: string;
	modifierId: number;
	modifierName: string;
	hazard: string;
	seed: number;
}

export interface ChallengeInfo {
	id: number;
	slug: string;
	name: string;
	blurb: string;
	mapId: number;
	mapName: string;
	modifierId: number;
	modifierName: string;
	seed: number;
	holdUntilWave: number | null;
}

export const SPEED_STEPS = [1, 2, 4] as const;
export type Speed = (typeof SPEED_STEPS)[number];

export function bestWaveKey(mapId: number, modifierId = 0) {
	return `otd-best-wave-${mapId}-${modifierId}`;
}

export function readBestWave(mapId: number, modifierId = 0): number {
	if (typeof localStorage === 'undefined') return 0;
	const keyed = Number(localStorage.getItem(bestWaveKey(mapId, modifierId)) || '0') || 0;
	if (keyed) return keyed;
	if (modifierId === 0) {
		return Number(localStorage.getItem(`otd-best-wave-${mapId}`) || '0') || 0;
	}
	return 0;
}

export function writeBestWave(mapId: number, wave: number, modifierId = 0) {
	if (typeof localStorage === 'undefined') return;
	const prev = readBestWave(mapId, modifierId);
	if (wave > prev) localStorage.setItem(bestWaveKey(mapId, modifierId), String(wave));
}

export function utcDay(now = Date.now()): number {
	return Math.floor(now / 86_400_000);
}

export const CAMPAIGN_KEY = 'otd-campaign';

export function readCampaignCleared(): number[] {
	if (typeof localStorage === 'undefined') return [];
	try {
		const raw = JSON.parse(localStorage.getItem(CAMPAIGN_KEY) || '{}') as { cleared?: unknown };
		if (!Array.isArray(raw.cleared)) return [];
		return raw.cleared.map(Number).filter((n) => Number.isFinite(n) && n >= 0);
	} catch {
		return [];
	}
}

export function markCampaignCleared(id: number) {
	if (typeof localStorage === 'undefined') return;
	const cleared = new Set(readCampaignCleared());
	cleared.add(id);
	localStorage.setItem(
		CAMPAIGN_KEY,
		JSON.stringify({ cleared: [...cleared].sort((a, b) => a - b) })
	);
}

export function missionUnlocked(id: number, cleared: number[] = readCampaignCleared()) {
	return id === 0 || cleared.includes(id - 1);
}
