const fs = require('fs');
const { execSync } = require('child_process');
const toml = require('@iarna/toml');

// Lire la version du Cargo.toml
const cargoToml = toml.parse(fs.readFileSync('./src-tauri/Cargo.toml', 'utf-8'));
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