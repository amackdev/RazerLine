// TypeScript mirrors of the Rust button types from `crates/desktop/src-tauri/src/buttons.rs`.
// Keep in sync with the Rust definitions.

export type MouseButtonId =
	| "LeftClick"
	| "RightClick"
	| "MiddleClick"
	| "TiltLeft"
	| "TiltRight"
	| "ScrollUp"
	| "ScrollDown"
	| "SideBack"
	| "SideForward"
	| "MultiFunctionClutch"
	| "DpiSwitch"
	| "ScrollMode"
	| "ProfileSwitch";

export const MouseButtonId = {
	ALL: [
		"LeftClick",
		"RightClick",
		"MiddleClick",
		"TiltLeft",
		"TiltRight",
		"ScrollUp",
		"ScrollDown",
		"SideBack",
		"SideForward",
		"MultiFunctionClutch",
		"DpiSwitch",
		"ScrollMode",
		"ProfileSwitch",
	] as const,

	CONFIGURABLE: [
		"MiddleClick",
		"TiltLeft",
		"TiltRight",
		"ScrollUp",
		"ScrollDown",
		"SideBack",
		"SideForward",
		"MultiFunctionClutch",
		"DpiSwitch",
		"ScrollMode",
		"ProfileSwitch",
	] as const,

	isConfigurable: (id: MouseButtonId): boolean => {
		return !["LeftClick", "RightClick"].includes(id);
	},

	displayName: (id: MouseButtonId): string => {
		const names: Record<MouseButtonId, string> = {
			LeftClick: "Left Click",
			RightClick: "Right Click",
			MiddleClick: "Scroll Click",
			TiltLeft: "Repeat Scroll Left",
			TiltRight: "Repeat Scroll Right",
			ScrollUp: "Scroll Up",
			ScrollDown: "Scroll Down",
			SideBack: "Mouse Button 4",
			SideForward: "Mouse Button 5",
			MultiFunctionClutch: "Sensitivity Clutch",
			DpiSwitch: "Cycle Up Sensitivity Stages",
			ScrollMode: "Scroll Mode Toggle",
			ProfileSwitch: "Cycle Up Profile",
		};
		return names[id];
	},

	shortName: (id: MouseButtonId): string => {
		const names: Record<MouseButtonId, string> = {
			LeftClick: "Left Click",
			RightClick: "Right Click",
			MiddleClick: "Scroll Click",
			TiltLeft: "Scroll Left",
			TiltRight: "Scroll Right",
			ScrollUp: "Scroll Up",
			ScrollDown: "Scroll Down",
			SideBack: "Button 4",
			SideForward: "Button 5",
			MultiFunctionClutch: "Clutch",
			DpiSwitch: "Cycle Sensitivity",
			ScrollMode: "Scroll Mode",
			ProfileSwitch: "Cycle Profile",
		};
		return names[id];
	},

	defaultAction: (id: MouseButtonId): ButtonAction => {
		switch (id) {
			case "LeftClick":
			case "RightClick":
			case "MiddleClick":
			case "ScrollUp":
			case "ScrollDown":
			case "SideBack":
			case "SideForward":
			case "ScrollMode":
				return { type: "Default" };
			case "TiltLeft":
				return { type: "Mouse", function: "ScrollLeft" };
			case "TiltRight":
				return { type: "Mouse", function: "ScrollRight" };
			case "MultiFunctionClutch":
				return { type: "Sensitivity", action: { type: "SniperClutch", clutch_dpi: 400 } };
			case "DpiSwitch":
				return { type: "Sensitivity", action: { type: "CycleDpi" } };
			case "ProfileSwitch":
				return { type: "ProfileCycle" };
		}
	},
} as const;

export type MouseFunction =
	| "LeftClick"
	| "RightClick"
	| "MiddleClick"
	| "ScrollLeft"
	| "ScrollRight"
	| "Back"
	| "Forward"
	| "DoubleClick";

export type SensitivityAction =
	| { type: "CycleDpi" }
	| { type: "DpiUp" }
	| { type: "DpiDown" }
	| { type: "SniperClutch"; clutch_dpi: number };

export type MediaAction =
	| "PlayPause"
	| "VolumeUp"
	| "VolumeDown"
	| "Mute"
	| "NextTrack"
	| "PrevTrack";

export type WinShortcut =
	| "LockPc"
	| "TaskView"
	| "ShowDesktop"
	| "OpenExplorer";

export type ButtonAction =
	| { type: "Default" }
	| { type: "Mouse"; function: MouseFunction }
	| { type: "Keyboard"; key_name: string }
	| { type: "Sensitivity"; action: SensitivityAction }
	| { type: "Multimedia"; action: MediaAction }
	| { type: "WindowsShortcut"; action: WinShortcut }
	| { type: "ProfileCycle" }
	| { type: "Disabled" };

export function buttonActionLabel(action: ButtonAction): string {
	switch (action.type) {
		case "Default":
			return "Default Function";
		case "Mouse":
			return `Mouse: ${mouseFunctionLabel(action.function)}`;
		case "Keyboard":
			return `Key: ${action.key_name}`;
		case "Sensitivity":
			return `Sensitivity: ${sensitivityActionLabel(action.action)}`;
		case "Multimedia":
			return `Media: ${mediaActionLabel(action.action)}`;
		case "WindowsShortcut":
			return `Windows: ${winShortcutLabel(action.action)}`;
		case "ProfileCycle":
			return "Profile: Cycle Next Profile";
		case "Disabled":
			return "Disabled";
	}
}

function mouseFunctionLabel(fn: MouseFunction): string {
	const labels: Record<MouseFunction, string> = {
		LeftClick: "Left Click",
		RightClick: "Right Click",
		MiddleClick: "Middle Click",
		ScrollLeft: "Scroll Left",
		ScrollRight: "Scroll Right",
		Back: "Back (M4)",
		Forward: "Forward (M5)",
		DoubleClick: "Double Click",
	};
	return labels[fn];
}

function sensitivityActionLabel(action: SensitivityAction): string {
	switch (action.type) {
		case "CycleDpi":
			return "DPI Cycle";
		case "DpiUp":
			return "DPI Stage Up";
		case "DpiDown":
			return "DPI Stage Down";
		case "SniperClutch":
			return `Sniper Clutch (${action.clutch_dpi} DPI)`;
	}
}

function mediaActionLabel(action: MediaAction): string {
	const labels: Record<MediaAction, string> = {
		PlayPause: "Play / Pause",
		VolumeUp: "Volume Up",
		VolumeDown: "Volume Down",
		Mute: "Mute Audio",
		NextTrack: "Next Track",
		PrevTrack: "Previous Track",
	};
	return labels[action];
}

function winShortcutLabel(action: WinShortcut): string {
	const labels: Record<WinShortcut, string> = {
		LockPc: "Lock PC (Win+L)",
		TaskView: "Task View (Win+Tab)",
		ShowDesktop: "Show Desktop (Win+D)",
		OpenExplorer: "File Explorer (Win+E)",
	};
	return labels[action];
}