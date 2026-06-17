#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');
const { spawnSync } = require('child_process');
const https = require('https');

const VERSION = '0.1.0';
const REPO = 'anilcan-kara/lightagent';

function getTarget() {
    const platform = os.platform();
    const arch = os.arch();

    if (platform === 'linux') {
        if (arch === 'x64') return { triple: 'x86_64-unknown-linux-musl', ext: 'tar.gz', bin: 'lightagent' };
        if (arch === 'arm64') return { triple: 'aarch64-unknown-linux-musl', ext: 'tar.gz', bin: 'lightagent' };
    } else if (platform === 'darwin') {
        if (arch === 'x64') return { triple: 'x86_64-apple-darwin', ext: 'tar.gz', bin: 'lightagent' };
        if (arch === 'arm64') return { triple: 'aarch64-apple-darwin', ext: 'tar.gz', bin: 'lightagent' };
    } else if (platform === 'win32') {
        if (arch === 'x64') return { triple: 'x86_64-pc-windows-msvc', ext: 'zip', bin: 'lightagent.exe' };
    }
    return null;
}

function downloadFile(url, destPath) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(destPath);
        
        const request = (targetUrl) => {
            https.get(targetUrl, (response) => {
                if (response.statusCode === 301 || response.statusCode === 302) {
                    request(response.headers.location);
                    return;
                }
                
                if (response.statusCode !== 200) {
                    reject(new Error(`Failed to download binary: HTTP ${response.statusCode}`));
                    return;
                }

                response.pipe(file);
                
                file.on('finish', () => {
                    file.close();
                    resolve();
                });
            }).on('error', (err) => {
                fs.unlink(destPath, () => {});
                reject(err);
            });
        };

        request(url);
    });
}

function main() {
    const target = getTarget();
    if (!target) {
        console.error(`Error: Unsupported platform/architecture (${os.platform()}/${os.arch()}).`);
        console.error("lightagent-cli supports 64-bit Linux, macOS, and Windows.");
        process.exit(1);
    }

    const homeDir = os.homedir();
    const installDir = path.join(homeDir, '.lightagent', `v${VERSION}`);
    const binPath = path.join(installDir, target.bin);

    if (fs.existsSync(binPath)) {
        runBinary(binPath);
        return;
    }

    console.error(`lightagent binary not found locally. Downloading for ${target.triple}...`);
    
    if (!fs.existsSync(installDir)) {
        fs.mkdirSync(installDir, { recursive: true });
    }

    const archiveName = `lightagent-${target.triple}.${target.ext}`;
    const archivePath = path.join(installDir, archiveName);
    const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;

    downloadFile(downloadUrl, archivePath)
        .then(() => {
            console.error("Extracting archive...");
            const tarResult = spawnSync('tar', ['-xf', archivePath, '-C', installDir]);
            if (tarResult.status !== 0) {
                throw new Error(`Failed to extract archive: ${tarResult.stderr.toString()}`);
            }

            try {
                fs.unlinkSync(archivePath);
            } catch (e) {}

            try {
                fs.chmodSync(binPath, '755');
            } catch (e) {}

            console.error("Download and extraction complete.\n");
            runBinary(binPath);
        })
        .catch((err) => {
            console.error(`Error: Failed to install binary: ${err.message}`);
            try {
                fs.unlinkSync(archivePath);
            } catch (e) {}
            process.exit(1);
        });
}

function runBinary(binaryPath) {
    const args = process.argv.slice(2);
    const result = spawnSync(binaryPath, args, { stdio: 'inherit' });
    process.exit(result.status ?? 0);
}

main();
