#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const REPO = 'Ayyankhan101/PipeCheck';
const VERSION = require('../package.json').version.replace(/^v/, '');

function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32') {
    return `pipecheck-${arch}.exe`;
  }
  return `pipecheck-${platform}-${arch}`;
}

function getReleaseAssetName() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'win32') {
    return `pipecheck-${arch}.exe`;
  }
  return `pipecheck-${platform}-${arch}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const request = https.get(url, { followRedirects: true }, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        https.get(res.headers.location, { followRedirects: true }, (redirectRes) => {
          const file = fs.createWriteStream(dest);
          redirectRes.pipe(file);
          file.on('finish', () => {
            file.close();
            resolve();
          });
        }).on('error', reject);
        return;
      }

      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    });
    request.on('error', reject);
  });
}

async function install() {
  const binaryName = getBinaryName();
  const assetName = getReleaseAssetName();
  const npmDir = path.join(__dirname, '..', 'npm');
  const binaryPath = path.join(npmDir, binaryName);

  // Check if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('✓ Pipecheck binary already installed');
    return;
  }

  if (!fs.existsSync(npmDir)) {
    fs.mkdirSync(npmDir, { recursive: true });
  }

  console.log(`Installing pipecheck v${VERSION}...`);

  const tag = `v${VERSION}`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${assetName}`;

  try {
    await download(url, binaryPath);
    fs.chmodSync(binaryPath, 0o755);
    console.log('✓ Pipecheck installed successfully');
  } catch (error) {
    console.error(`Failed to download binary from ${url}`);
    console.error('Falling back to building from source...');

    try {
      execSync('cargo build --release', { stdio: 'inherit' });
      const sourceBinary = path.join(__dirname, '..', 'target', 'release',
        process.platform === 'win32' ? 'pipecheck.exe' : 'pipecheck');
      fs.copyFileSync(sourceBinary, binaryPath);
      fs.chmodSync(binaryPath, 0o755);
      console.log('✓ Pipecheck installed from source');
    } catch (buildError) {
      console.error('Failed to build from source. Please ensure Rust is installed.');
      console.error('Visit https://rustup.rs to install Rust');
      process.exit(1);
    }
  }
}

install();
