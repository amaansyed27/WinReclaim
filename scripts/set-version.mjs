import { readFile, writeFile } from "node:fs/promises";

const input = process.argv[2]?.trim().replace(/^v/i, "");

if (!input || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(input)) {
  console.error("Usage: npm run version:set -- 1.2.3");
  process.exit(1);
}

async function updateJson(path) {
  const data = JSON.parse(await readFile(path, "utf8"));
  data.version = input;
  await writeFile(path, `${JSON.stringify(data, null, 2)}\n`);
}

await updateJson("package.json");
await updateJson("src-tauri/tauri.conf.json");

const cargoPath = "src-tauri/Cargo.toml";
const cargo = await readFile(cargoPath, "utf8");
const packageVersionPattern = /(^\[package\][\s\S]*?^version\s*=\s*)"([^"]+)"/m;
const match = cargo.match(packageVersionPattern);

if (!match) {
  throw new Error("Unable to locate the Cargo package version");
}

const updatedCargo = cargo.replace(packageVersionPattern, `$1"${input}"`);
if (updatedCargo !== cargo) {
  await writeFile(cargoPath, updatedCargo);
}

console.log(`WinReclaim version set to ${input}`);
