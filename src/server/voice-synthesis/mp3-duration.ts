export function mp3DurationMs(data: Uint8Array): number {
  let offset = id3v2Size(data);
  let durationSeconds = 0;

  while (offset + 4 <= data.length) {
    const header = parseMp3FrameHeader(data, offset);
    if (!header) {
      offset += 1;
      continue;
    }

    durationSeconds += header.samplesPerFrame / header.sampleRate;
    offset += header.frameLength;
  }

  return durationSeconds * 1000;
}

function id3v2Size(data: Uint8Array): number {
  if (
    data.length < 10 ||
    data[0] !== 0x49 ||
    data[1] !== 0x44 ||
    data[2] !== 0x33
  ) {
    return 0;
  }

  const flags = data[5] ?? 0;
  const tagSize =
    ((data[6] ?? 0) << 21) |
    ((data[7] ?? 0) << 14) |
    ((data[8] ?? 0) << 7) |
    (data[9] ?? 0);
  const footerSize = flags & 0x10 ? 10 : 0;
  return 10 + tagSize + footerSize;
}

function parseMp3FrameHeader(
  data: Uint8Array,
  offset: number,
): {
  readonly frameLength: number;
  readonly sampleRate: number;
  readonly samplesPerFrame: number;
} | null {
  const byte1 = data[offset];
  const byte2 = data[offset + 1];
  const byte3 = data[offset + 2];
  const byte4 = data[offset + 3];
  if (
    byte1 !== 0xff ||
    byte2 === undefined ||
    byte3 === undefined ||
    byte4 === undefined ||
    (byte2 & 0xe0) !== 0xe0
  ) {
    return null;
  }

  const versionBits = (byte2 >> 3) & 0x03;
  const layerBits = (byte2 >> 1) & 0x03;
  const bitrateIndex = (byte3 >> 4) & 0x0f;
  const sampleRateIndex = (byte3 >> 2) & 0x03;
  const padding = (byte3 >> 1) & 0x01;

  if (
    versionBits === 0x01 ||
    layerBits !== 0x01 ||
    bitrateIndex === 0x00 ||
    bitrateIndex === 0x0f ||
    sampleRateIndex === 0x03
  ) {
    return null;
  }

  const mpegVersion = versionBits === 0x03 ? 1 : versionBits === 0x02 ? 2 : 2.5;
  const bitrate = bitrateKbps(mpegVersion, bitrateIndex) * 1000;
  const sampleRate = sampleRateHz(mpegVersion, sampleRateIndex);
  const samplesPerFrame = mpegVersion === 1 ? 1152 : 576;
  const frameLength = Math.floor(
    ((mpegVersion === 1 ? 144 : 72) * bitrate) / sampleRate + padding,
  );

  if (frameLength <= 4) {
    return null;
  }

  return { frameLength, sampleRate, samplesPerFrame };
}

function bitrateKbps(mpegVersion: 1 | 2 | 2.5, index: number): number {
  const mpeg1Layer3 = [
    0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
  ];
  const mpeg2Layer3 = [
    0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160,
  ];

  return (mpegVersion === 1 ? mpeg1Layer3 : mpeg2Layer3)[index] ?? 0;
}

function sampleRateHz(mpegVersion: 1 | 2 | 2.5, index: number): number {
  const mpeg1 = [44100, 48000, 32000];
  const mpeg2 = [22050, 24000, 16000];
  const mpeg25 = [11025, 12000, 8000];

  return (mpegVersion === 1 ? mpeg1 : mpegVersion === 2 ? mpeg2 : mpeg25)[
    index
  ] ?? 0;
}
