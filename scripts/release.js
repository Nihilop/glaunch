import { readFileSync, writeFileSync } from 'fs';
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

// Mettre à jour tauri.conf.json
const tauriConfigPath = join(__dirname, '..', 'src-tauri', 'tauri.conf.json');
const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, 'utf-8'));
if (tauriConfig.version !== version) {
    console.log(`Updating tauri.conf.json version from ${tauriConfig.version} to ${version}`);
    tauriConfig.version = version;
    writeFileSync(tauriConfigPath, JSON.stringify(tauriConfig, null, 2));
}

// Vérifier si le tag existe déjà
try {
    execSync(`git rev-parse v${version}`, { stdio: 'ignore' });
    console.error(`Error: Tag v${version} already exists`);
    process.exit(1);
} catch {
    // Tag n'existe pas, on continue
}

// Commiter les changements de version si nécessaire
try {
    const status = execSync('git status --porcelain').toString();
    if (status.includes('tauri.conf.json')) {
        console.log('Committing version update...');
        execSync('git add src-tauri/tauri.conf.json');
        execSync(`git commit -m "chore: update version to ${version}"`);
    }
} catch (error) {
    console.error('Error committing changes:', error);
    process.exit(1);
}

// Créer le tag
console.log(`Creating tag v${version}...`);
execSync(`git tag -a v${version} -m "Release v${version}"`);

// Pousser les changements et le tag
console.log('Pushing changes and tag...');
execSync('git push origin main');
execSync('git push origin --tags');

console.log('Release process completed!');