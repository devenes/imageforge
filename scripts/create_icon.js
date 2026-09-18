import fs from 'fs';

// Generate an uncompressed or simple valid 1024x1024 PNG
// We can use a minimal canvas or raw PNG chunks
function createMinimalPng(width, height) {
  // Simple PPM or canvas-free PNG generator
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  
  // IHDR
  const ihdrData = Buffer.alloc(13);
  ihdrData.writeUInt32BE(width, 0);
  ihdrData.writeUInt32BE(height, 4);
  ihdrData[8] = 8; // bit depth
  ihdrData[9] = 6; // color type RGBA
  ihdrData[10] = 0; // compression
  ihdrData[11] = 0; // filter
  ihdrData[12] = 0; // interlace
  
  const ihdrChunk = makeChunk('IHDR', ihdrData);
  
  // IDAT with zlib uncompressed blocks
  // Scanlines: (1 byte filter 0 + 4 * width bytes) * height
  const rowBytes = 1 + width * 4;
  const rawData = Buffer.alloc(rowBytes * height);
  
  for (let y = 0; y < height; y++) {
    const rowOffset = y * rowBytes;
    rawData[rowOffset] = 0; // filter None
    for (let x = 0; x < width; x++) {
      const pxOffset = rowOffset + 1 + x * 4;
      // Beautiful Mac indigo/blue gradient with rounded icon vibe
      const cx = x - width / 2;
      const cy = y - height / 2;
      const dist = Math.sqrt(cx * cx + cy * cy);
      if (dist < width * 0.45) {
        rawData[pxOffset] = 0x25;     // R
        rawData[pxOffset + 1] = 0x63; // G
        rawData[pxOffset + 2] = 0xeb; // B (royal blue)
        rawData[pxOffset + 3] = 0xff; // A
      } else {
        rawData[pxOffset] = 0;
        rawData[pxOffset + 1] = 0;
        rawData[pxOffset + 2] = 0;
        rawData[pxOffset + 3] = 0;
      }
    }
  }
  
  import('zlib').then(zlib => {
    const compressed = zlib.deflateSync(rawData);
    const idatChunk = makeChunk('IDAT', compressed);
    const iendChunk = makeChunk('IEND', Buffer.alloc(0));
    
    const png = Buffer.concat([signature, ihdrChunk, idatChunk, iendChunk]);
    fs.writeFileSync('app-icon.png', png);
    console.log('Generated app-icon.png successfully');
  });
}

function makeChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const body = Buffer.concat([typeBuf, data]);
  
  // CRC32
  let crc = 0xffffffff;
  for (let i = 0; i < body.length; i++) {
    crc = crcTable[(crc ^ body[i]) & 0xff] ^ (crc >>> 8);
  }
  crc = (crc ^ 0xffffffff) >>> 0;
  const crcBuf = Buffer.alloc(4);
  crcBuf.writeUInt32BE(crc, 0);
  
  return Buffer.concat([len, body, crcBuf]);
}

const crcTable = new Uint32Array(256);
for (let n = 0; n < 256; n++) {
  let c = n;
  for (let k = 0; k < 8; k++) {
    c = (c & 1) ? (0xedb88320 ^ (c >>> 1)) : (c >>> 1);
  }
  crcTable[n] = c >>> 0;
}

createMinimalPng(1024, 1024);
