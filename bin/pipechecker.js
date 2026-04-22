#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

const baseDir = path.resolve(__dirname, '..');
const binPath = path.join(baseDir, 'npm', getBinaryName());

const child = spawn(binPath, process.argv.slice(2), { stdio: 'inherit' });

child.on('exit', (code) => {
  process.exit(code);
});

function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;
  
  if (platform === 'win32') {
    return `pipechecker-${arch}.exe`;
  }
  return `pipechecker-${platform}-${arch}`;
}
