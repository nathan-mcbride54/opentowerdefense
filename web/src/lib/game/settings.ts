import { DEFAULT_KEYS, mergeKeys, type ActionId } from './keys';

export type Palette = 'default' | 'safe' | 'high';
export type UiScale = 'sm' | 'md' | 'lg';

export interface Settings {
	mute: boolean;
	volume: number;
	reducedFx: boolean;
	palette: Palette;
	uiScale: UiScale;
	keys: Record<ActionId, string>;
}

const KEY = 'otd-settings';
const listeners = new Set<() => void>();

const DEFAULTS: Settings = {
	mute: false,
	volume: 0.55,
	reducedFx: false,
	palette: 'default',
	uiScale: 'md',
	keys: { ...DEFAULT_KEYS }
};

function clamp(v: number, lo: number, hi: number) {
	return Math.min(hi, Math.max(lo, v));
}

export function loadSettings(): Settings {
	if (typeof localStorage === 'undefined') return { ...DEFAULTS, keys: { ...DEFAULT_KEYS } };
	try {
		const raw = JSON.parse(localStorage.getItem(KEY) || '{}') as Partial<Settings>;
		return {
			mute: Boolean(raw.mute),
			volume: clamp(Number(raw.volume ?? DEFAULTS.volume), 0, 1),
			reducedFx: Boolean(raw.reducedFx),
			palette: raw.palette === 'safe' || raw.palette === 'high' ? raw.palette : 'default',
			uiScale: raw.uiScale === 'sm' || raw.uiScale === 'lg' ? raw.uiScale : 'md',
			keys: mergeKeys(raw.keys)
		};
	} catch {
		return { ...DEFAULTS, keys: { ...DEFAULT_KEYS } };
	}
}

export function saveSettings(next: Settings) {
	if (typeof localStorage === 'undefined') return;
	localStorage.setItem(KEY, JSON.stringify(next));
	applyUiScale(next.uiScale);
	for (const fn of listeners) fn();
}

export function applyUiScale(scale: UiScale) {
	if (typeof document === 'undefined') return;
	const px = scale === 'sm' ? '13px' : scale === 'lg' ? '17px' : '15px';
	document.documentElement.style.fontSize = px;
}

export function subscribeSettings(fn: () => void) {
	listeners.add(fn);
	return () => {
		listeners.delete(fn);
	};
}

export function patchSettings(partial: Partial<Settings>) {
	saveSettings({ ...loadSettings(), ...partial });
}

export function rebindKey(action: ActionId, key: string) {
	const s = loadSettings();
	const keys = { ...s.keys };
	const other = (Object.keys(keys) as ActionId[]).find((id) => keys[id] === key && id !== action);
	if (other) keys[other] = keys[action];
	keys[action] = key;
	patchSettings({ keys });
}

export function resetKeys() {
	patchSettings({ keys: { ...DEFAULT_KEYS } });
}
