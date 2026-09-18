#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const packageJsonPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

function getPackageVersion() {
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  return pkg.version;
}

function getGitTagVersion() {
  try {
    const tag = execSync('git describe --tags --exact-match 2>/dev/null', {
      cwd: rootDir,
      encoding: 'utf8'
    }).trim();
    if (tag) {
      return tag.startsWith('v') ? tag.slice(1) : tag;
    }
  } catch {
    // Not on an exact tag or git not available
  }
  return null;
}

function normalizeVersion(ver) {
  let v = ver.trim();
  if (v.startsWith('v')) {
    v = v.slice(1);
  }
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(v)) {
    throw new Error(`Invalid version format: "${ver}". Expected semantic version (e.g., 0.1.0 or 0.1.0-beta.1)`);
  }
  return v;
}

function updateVersions(newVersion) {
  const v = normalizeVersion(newVersion);

  // 1. package.json
  const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  const oldVersion = pkg.version;
  pkg.version = v;
  fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');

  // 2. src-tauri/tauri.conf.json
  if (fs.existsSync(tauriConfPath)) {
    const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
    tauriConf.version = v;
    fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  }

  // 3. src-tauri/Cargo.toml
  if (fs.existsSync(cargoTomlPath)) {
    let cargoToml = fs.readFileSync(cargoTomlPath, 'utf8');
    // Replace version under [package]
    const updated = cargoToml.replace(
      /(\[package\][\s\S]*?^version\s*=\s*")[^"]+(")/m,
      `$1${v}$2`
    );
    fs.writeFileSync(cargoTomlPath, updated, 'utf8');
  }

  console.log(`Synchronized version: ${oldVersion} -> ${v}`);
  return v;
}

const args = process.argv.slice(2);

if (args.includes('--get') || args.includes('-g') || args.includes('--print')) {
  process.stdout.write(getPackageVersion() + '\n');
  process.exit(0);
}

if (args.includes('--from-tag')) {
  const tagVer = getGitTagVersion();
  if (tagVer) {
    updateVersions(tagVer);
    process.exit(0);
  } else {
    console.log('No exact git tag found on HEAD. Keeping existing version:', getPackageVersion());
    process.exit(0);
  }
}

const targetArg = args.find((arg) => !arg.startsWith('-'));
if (targetArg) {
  updateVersions(targetArg);
} else {
  // Synchronize existing version from package.json to other files
  const current = getPackageVersion();
  updateVersions(current);
}
