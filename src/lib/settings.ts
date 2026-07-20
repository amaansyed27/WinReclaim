export type DefaultScanProfile = "quick" | "balanced" | "deep" | "ultra";

export interface AppPreferences {
  automaticUpdateChecks: boolean;
  defaultScanProfile: DefaultScanProfile;
}

const STORAGE_KEY = "winreclaim.preferences.v1";

export const defaultPreferences: AppPreferences = {
  automaticUpdateChecks: true,
  defaultScanProfile: "balanced"
};

export function loadPreferences(): AppPreferences {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...defaultPreferences };
    const parsed = JSON.parse(raw) as Partial<AppPreferences>;
    return {
      automaticUpdateChecks:
        typeof parsed.automaticUpdateChecks === "boolean"
          ? parsed.automaticUpdateChecks
          : defaultPreferences.automaticUpdateChecks,
      defaultScanProfile: isScanProfile(parsed.defaultScanProfile)
        ? parsed.defaultScanProfile
        : defaultPreferences.defaultScanProfile
    };
  } catch {
    return { ...defaultPreferences };
  }
}

export function savePreferences(preferences: AppPreferences): void {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
}

export function resetPreferences(): AppPreferences {
  window.localStorage.removeItem(STORAGE_KEY);
  return { ...defaultPreferences };
}

function isScanProfile(value: unknown): value is DefaultScanProfile {
  return value === "quick" || value === "balanced" || value === "deep" || value === "ultra";
}
