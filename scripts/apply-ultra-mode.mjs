import { readFileSync, writeFileSync } from "node:fs";

function update(path, replacements) {
  let content = readFileSync(path, "utf8");
  for (const [from, to] of replacements) {
    if (!content.includes(from)) {
      throw new Error(`Expected source fragment was not found in ${path}: ${from.slice(0, 100)}`);
    }
    content = content.replace(from, to);
  }
  writeFileSync(path, content);
}

update("src/features/scan/ScanView.tsx", [
  [
    'type BooleanScanOption =\n  | "includeKnownTargets"\n  | "includeProjectOutputs"\n  | "discoverUnknown"\n  | "includeAppData";\n\nconst modeCopy: Record<ScanMode, string> = {\n  quick: "Known locations and a bounded discovery pass.",\n  balanced: "Broader project and unclassified directory discovery.",\n  deep: "Largest entry budget and deepest directory traversal."\n};',
    'type BooleanScanOption =\n  | "includeKnownTargets"\n  | "includeProjectOutputs"\n  | "discoverUnknown"\n  | "includeAppData";\n\ntype ScanProfile = ScanMode | "ultra";\n\nconst modeCopy: Record<ScanProfile, string> = {\n  quick: "Known locations and a bounded discovery pass.",\n  balanced: "Broader project and unclassified directory discovery.",\n  deep: "Largest bounded scan for heavily nested workspaces.",\n  ultra: "Exhaustive profile traversal with every scan source enabled and no depth or entry cap."\n};'
  ],
  [
    '  const [options, setOptions] = useState<ScanOptions>({\n    mode: "balanced",',
    '  const [selectedProfile, setSelectedProfile] = useState<ScanProfile>("balanced");\n  const [options, setOptions] = useState<ScanOptions>({\n    mode: "balanced",'
  ],
  [
    '  function setFlag(key: BooleanScanOption, value: boolean) {\n    setOptions((current) => ({ ...current, [key]: value }));\n  }',
    '  function setFlag(key: BooleanScanOption, value: boolean) {\n    setOptions((current) => ({ ...current, [key]: value }));\n  }\n\n  function selectProfile(profile: ScanProfile) {\n    setSelectedProfile(profile);\n    if (profile === "ultra") {\n      setOptions({\n        mode: "deep",\n        includeKnownTargets: true,\n        includeProjectOutputs: true,\n        discoverUnknown: true,\n        includeAppData: true,\n        minimumFindingBytes: 64 * 1024 * 1024,\n        maxUnknownFindings: 100\n      });\n      return;\n    }\n    setOptions((current) => ({ ...current, mode: profile }));\n  }'
  ],
  [
    '{(["quick", "balanced", "deep"] as ScanMode[]).map((mode) => (',
    '{(["quick", "balanced", "deep", "ultra"] as ScanProfile[]).map((mode) => ('
  ],
  [
    'className={options.mode === mode ? "is-active" : ""}',
    'className={selectedProfile === mode ? "is-active" : ""}'
  ],
  [
    'onClick={() => setOptions((current) => ({ ...current, mode }))}',
    'onClick={() => selectProfile(mode)}'
  ],
  [
    '<p className="config-help">{modeCopy[options.mode]}</p>',
    '<p className="config-help">{modeCopy[selectedProfile]}</p>'
  ],
  [
    'disabled={scanning}\n                onChange={(event) =>',
    'disabled={scanning || selectedProfile === "ultra"}\n                onChange={(event) =>'
  ],
  [
    'disabled={scanning || !options.discoverUnknown}',
    'disabled={scanning || !options.discoverUnknown || selectedProfile === "ultra"}'
  ],
  [
    '<option value={75}>75</option>',
    '<option value={75}>75</option>\n                <option value={100}>100</option>'
  ],
  [
    'disabled={scanning}\n                onChange={(value) => setFlag("includeKnownTargets", value)}',
    'disabled={scanning || selectedProfile === "ultra"}\n                onChange={(value) => setFlag("includeKnownTargets", value)}'
  ],
  [
    'disabled={scanning}\n                onChange={(value) => setFlag("includeProjectOutputs", value)}',
    'disabled={scanning || selectedProfile === "ultra"}\n                onChange={(value) => setFlag("includeProjectOutputs", value)}'
  ],
  [
    'disabled={scanning}\n                onChange={(value) => setFlag("discoverUnknown", value)}',
    'disabled={scanning || selectedProfile === "ultra"}\n                onChange={(value) => setFlag("discoverUnknown", value)}'
  ],
  [
    'disabled={scanning || !options.discoverUnknown}\n                onChange={(value) => setFlag("includeAppData", value)}',
    'disabled={scanning || !options.discoverUnknown || selectedProfile === "ultra"}\n                onChange={(value) => setFlag("includeAppData", value)}'
  ]
]);

update("src/desktop-tuning.css", [
  [
    ".segmented-control { display: grid; grid-template-columns: repeat(3, 1fr);",
    ".segmented-control { display: grid; grid-template-columns: repeat(4, 1fr);"
  ]
]);

update("src-tauri/src/scanner/profile.rs", [
  [
    "    let mode = mode_limits(request.mode);",
    "    let ultra = request.mode == ScanMode::Deep\n        && request.include_known_targets\n        && request.include_project_outputs\n        && request.discover_unknown\n        && request.include_app_data\n        && request.minimum_finding_bytes <= 64 * 1024 * 1024\n        && request.max_unknown_findings >= 100;\n    let mode = mode_limits(request.mode, ultra);"
  ],
  [
    "fn mode_limits(mode: ScanMode) -> ModeLimits {\n    match mode {",
    "fn mode_limits(mode: ScanMode, ultra: bool) -> ModeLimits {\n    if ultra {\n        return ModeLimits {\n            project_depth: usize::MAX,\n            discovery_depth: usize::MAX,\n            max_entries: u64::MAX,\n        };\n    }\n\n    match mode {"
  ]
]);
