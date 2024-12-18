// scripts/release.js
import { readFileSync } from 'fs';
import { execSync } from 'child_process';
import { parse } from '@iarna/toml';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Lire la version du Cargo.toml
const cargoTomlPath = join(__dirname, '..', 'src-tauri', 'Cargo.toml');
const cargoToml = parse(readFileSync(cargoTomlPath, 'utf-8'));
const version = cargoToml.package.version;

// Vérifier si le tag existe déjà
try {
    execSync(`git rev-parse v${version}`, { stdio: 'ignore' });
    console.error(`Error: Tag v${version} already exists`);
    process.exit(1);
} catch {
    // Tag n'existe pas, on continue
}

// Créer le tag
console.log(`Creating tag v${version}...`);
execSync(`git tag -a v${version} -m "Release v${version}"`);

// Pousser le tag
console.log(`Pushing tag v${version}...`);
execSync('git push origin --tags');

console.log('Release process completed!');