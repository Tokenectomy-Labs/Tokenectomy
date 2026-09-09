#!/usr/bin/env node

/**
 * Tokenectomy Razor 🗡️ — Autonomous M2M Model Context Protocol (MCP) Server
 * Lightweight zero-dependency native binary runner for npm/npx.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');
const { spawn } = require('child_process');

const pkg = require('../package.json');

const REPO = 'daffa2555/Tokenectomy';
const VERSION = pkg.version;

function getPlatformAsset() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'linux') {
    if (arch === 'x64') return 'razor-linux-amd64';
    if (arch === 'arm64') {
      // Future-proof fallback or arm64 support
      return 'razor-linux-amd64';
    }
  } else if (platform === 'darwin') {
    if (arch === 'arm64') return 'razor-macos-arm64';
    if (arch === 'x64') return 'razor-macos-intel';
  } else if (platform === 'win32') {
    if (arch === 'x64' || arch === 'ia32') return 'razor-windows-amd64.exe';
  }

  return null;
}

function getCacheDir() {
  if (process.env.TOKENECTOMY_CACHE_DIR) {
    return process.env.TOKENECTOMY_CACHE_DIR;
  }
  return path.join(os.homedir(), '.tokenectomy', 'bin');
}

function downloadFile(url, destPath, maxRedirects = 5) {
  return new Promise((resolve, reject) => {
    if (maxRedirects <= 0) {
      return reject(new Error('Too many redirects while downloading binary'));
    }

    const req = https.get(url, {
      headers: {
        'User-Agent': 'tokenectomy-razor-npm'
      }
    }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return downloadFile(res.headers.location, destPath, maxRedirects - 1).then(resolve, reject);
      }

      if (res.statusCode === 404) {
        const notFoundErr = new Error('NOT_FOUND');
        notFoundErr.statusCode = 404;
        return reject(notFoundErr);
      }

      if (res.statusCode !== 200) {
        return reject(new Error(`Failed to download binary: HTTP ${res.statusCode} ${res.statusMessage || ''}`));
      }

      const file = fs.createWriteStream(destPath);
      res.pipe(file);
      file.on('finish', () => {
        file.close(() => resolve());
      });
      file.on('error', (err) => {
        fs.unlink(destPath, () => reject(err));
      });
    });

    req.on('error', (err) => {
      fs.unlink(destPath, () => reject(err));
    });
  });
}

async function ensureBinary() {
  if (process.env.TOKENECTOMY_BIN && fs.existsSync(process.env.TOKENECTOMY_BIN)) {
    return process.env.TOKENECTOMY_BIN;
  }

  const assetName = getPlatformAsset();
  if (!assetName) {
    process.stderr.write(
      `[tokenectomy-razor] Unsupported platform: ${process.platform} (${process.arch}).\n` +
      `[tokenectomy-razor] Please compile from source: cargo install tokenectomy\n`
    );
    process.exit(1);
  }

  const cacheDir = getCacheDir();
  fs.mkdirSync(cacheDir, { recursive: true });

  const isWindows = process.platform === 'win32';
  const binaryFileName = `razor-v${VERSION}-${assetName}`;
  const binaryPath = path.join(cacheDir, binaryFileName);

  // Check if binary already exists and is healthy
  if (fs.existsSync(binaryPath)) {
    try {
      const stats = fs.statSync(binaryPath);
      if (stats.size > 1024 * 1024) {
        // Healthy binary (> 1MB)
        return binaryPath;
      }
    } catch (_) {
      // Ignore stat error and re-download
    }
  }

  // Download binary from GitHub Releases
  const tempPath = path.join(cacheDir, `${binaryFileName}.tmp-${Date.now()}`);
  const primaryUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;
  const fallbackUrl = `https://github.com/${REPO}/releases/latest/download/${assetName}`;

  process.stderr.write(`[tokenectomy-razor] Fetching native binary (${assetName})...\n`);

  try {
    try {
      await downloadFile(primaryUrl, tempPath);
    } catch (err) {
      if (err.statusCode === 404) {
        process.stderr.write(`[tokenectomy-razor] v${VERSION} release asset not found, trying latest release...\n`);
        await downloadFile(fallbackUrl, tempPath);
      } else {
        throw err;
      }
    }

    fs.renameSync(tempPath, binaryPath);

    if (!isWindows) {
      fs.chmodSync(binaryPath, 0o755);
    }

    process.stderr.write(`[tokenectomy-razor] Native binary ready at ${binaryPath}\n`);
    return binaryPath;
  } catch (err) {
    try {
      if (fs.existsSync(tempPath)) fs.unlinkSync(tempPath);
    } catch (_) {}

    process.stderr.write(
      `[tokenectomy-razor] Download failed: ${err.message}\n` +
      `[tokenectomy-razor] You can set TOKENECTOMY_BIN=/path/to/razor or install via cargo: cargo install tokenectomy\n`
    );
    process.exit(1);
  }
}

async function main() {
  const binaryPath = await ensureBinary();
  const args = process.argv.slice(2);

  const child = spawn(binaryPath, args, {
    stdio: 'inherit'
  });

  child.on('error', (err) => {
    process.stderr.write(`[tokenectomy-razor] Failed to launch binary: ${err.message}\n`);
    process.exit(1);
  });

  child.on('close', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 0);
    }
  });

  process.on('SIGINT', () => child.kill('SIGINT'));
  process.on('SIGTERM', () => child.kill('SIGTERM'));
}

if (require.main === module) {
  main().catch((err) => {
    process.stderr.write(`[tokenectomy-razor] Fatal error: ${err.message}\n`);
    process.exit(1);
  });
}

module.exports = {
  ensureBinary,
  getPlatformAsset,
  getCacheDir
};
