#!/usr/bin/env node
/**
 * Generate ContextPaste app icons from SVG using sharp.
 * Outputs PNGs at various sizes + ICO to src-tauri/icons/
 */

const sharp = require('sharp');
const path = require('path');
const fs = require('fs');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');

const SVG = `<svg viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0.5" y2="1">
      <stop offset="0%" stop-color="#3b82f6"/>
      <stop offset="100%" stop-color="#7c3aed"/>
    </linearGradient>
    <linearGradient id="clip" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#93c5fd"/>
      <stop offset="100%" stop-color="#60a5fa"/>
    </linearGradient>
    <filter id="shadow" x="-4%" y="-4%" width="108%" height="108%">
      <feDropShadow dx="0" dy="2" stdDeviation="6" flood-color="#1e1b4b" flood-opacity="0.25"/>
    </filter>
  </defs>

  <!-- Main rounded square background -->
  <rect x="32" y="32" width="448" height="448" rx="80" fill="url(#bg)"/>

  <!-- Clipboard clip at top -->
  <rect x="186" y="12" width="140" height="64" rx="18" fill="#1e3a8a" filter="url(#shadow)"/>
  <rect x="202" y="20" width="108" height="42" rx="10" fill="url(#clip)"/>

  <!-- Clipboard board (white inner area) -->
  <rect x="104" y="76" width="304" height="356" rx="22" fill="white" opacity="0.95"/>

  <!-- Text lines on clipboard -->
  <rect x="148" y="132" width="216" height="14" rx="7" fill="#cbd5e1"/>
  <rect x="148" y="164" width="170" height="14" rx="7" fill="#e2e8f0"/>
  <rect x="148" y="196" width="192" height="14" rx="7" fill="#cbd5e1"/>

  <!-- AI Sparkle star -->
  <path d="M256 250 L264 280 L296 280 L270 300 L280 332 L256 312 L232 332 L242 300 L216 280 L248 280 Z"
        fill="#2563eb"/>

  <!-- Accent sparkle dots -->
  <circle cx="188" cy="290" r="5" fill="#7c3aed" opacity="0.7"/>
  <circle cx="324" cy="290" r="5" fill="#7c3aed" opacity="0.7"/>
  <circle cx="206" cy="350" r="4" fill="#3b82f6" opacity="0.5"/>
  <circle cx="306" cy="350" r="4" fill="#3b82f6" opacity="0.5"/>
  <circle cx="256" cy="380" r="5" fill="#7c3aed" opacity="0.6"/>

  <!-- Small sparkle crosses for AI feel -->
  <g fill="#60a5fa" opacity="0.6">
    <path d="M340 140 L344 152 L356 152 L346 160 L350 172 L340 164 L330 172 L334 160 L324 152 L336 152 Z" transform="scale(0.6) translate(230, 100)"/>
    <path d="M340 140 L344 152 L356 152 L346 160 L350 172 L340 164 L330 172 L334 160 L324 152 L336 152 Z" transform="scale(0.45) translate(100, 520)"/>
  </g>
</svg>`;

async function generateIcons() {
  // Ensure output directory exists
  fs.mkdirSync(ICONS_DIR, { recursive: true });

  const svgBuffer = Buffer.from(SVG);

  // Generate PNGs at required sizes
  const sizes = [
    { name: '32x32.png', size: 32 },
    { name: '128x128.png', size: 128 },
    { name: '128x128@2x.png', size: 256 },
    { name: 'icon.png', size: 512 },
  ];

  for (const { name, size } of sizes) {
    const outPath = path.join(ICONS_DIR, name);
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(outPath);
    const stat = fs.statSync(outPath);
    console.log(`Created ${name} (${size}x${size}) — ${stat.size} bytes`);
  }

  // Generate ICO (Windows icon) — ICO contains multiple sizes
  // sharp doesn't natively produce .ico, so we'll build one manually
  // ICO format: header + directory entries + PNG image data
  const icoSizes = [16, 32, 48, 256];
  const pngBuffers = [];
  for (const size of icoSizes) {
    const buf = await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toBuffer();
    pngBuffers.push({ size, data: buf });
  }

  const icoBuffer = buildIco(pngBuffers);
  const icoPath = path.join(ICONS_DIR, 'icon.ico');
  fs.writeFileSync(icoPath, icoBuffer);
  console.log(`Created icon.ico — ${icoBuffer.length} bytes`);

  // For macOS, just copy the 512x512 PNG as icon.icns placeholder
  // (Real .icns would need iconutil on macOS; this is sufficient for dev)
  const icnsPath = path.join(ICONS_DIR, 'icon.icns');
  const png512 = await sharp(svgBuffer).resize(512, 512).png().toBuffer();
  fs.writeFileSync(icnsPath, png512);
  console.log(`Created icon.icns (PNG copy) — ${png512.length} bytes`);

  console.log('\nAll icons generated successfully in src-tauri/icons/');
}

/**
 * Build a minimal ICO file from PNG buffers.
 * ICO format spec: https://en.wikipedia.org/wiki/ICO_(file_format)
 */
function buildIco(entries) {
  const numImages = entries.length;
  const headerSize = 6;
  const dirEntrySize = 16;
  const dirSize = dirEntrySize * numImages;
  let dataOffset = headerSize + dirSize;

  // ICO header: reserved(2) + type(2, 1=ico) + count(2)
  const header = Buffer.alloc(headerSize);
  header.writeUInt16LE(0, 0);       // reserved
  header.writeUInt16LE(1, 2);       // type: icon
  header.writeUInt16LE(numImages, 4);

  const dirEntries = [];
  const imageDataParts = [];

  for (const { size, data } of entries) {
    const entry = Buffer.alloc(dirEntrySize);
    entry.writeUInt8(size < 256 ? size : 0, 0);  // width (0 = 256)
    entry.writeUInt8(size < 256 ? size : 0, 1);  // height (0 = 256)
    entry.writeUInt8(0, 2);          // color palette
    entry.writeUInt8(0, 3);          // reserved
    entry.writeUInt16LE(1, 4);       // color planes
    entry.writeUInt16LE(32, 6);      // bits per pixel
    entry.writeUInt32LE(data.length, 8);   // image data size
    entry.writeUInt32LE(dataOffset, 12);   // offset to image data

    dirEntries.push(entry);
    imageDataParts.push(data);
    dataOffset += data.length;
  }

  return Buffer.concat([header, ...dirEntries, ...imageDataParts]);
}

generateIcons().catch(err => {
  console.error('Error generating icons:', err);
  process.exit(1);
});
