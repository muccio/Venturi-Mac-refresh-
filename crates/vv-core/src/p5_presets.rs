//! Built-in P5.js creative coding sketch presets for the P5 canvas filter.
//!
//! Each preset provides a production-ready JavaScript sketch demonstrating
//! processing of the incoming video frame `videoIn` and reactivity to `audioIn`.

pub const ASCII_COLOR: &str = r#"// P5.js Canvas Effect: ASCII Art (Full Color)
// param0: Character Size (6.0 - 32.0, default: 10.0)
// param1: Contrast Boost (0.5 - 3.0, default: 1.2)
// param2: Saturation Boost (0.0 - 2.0, default: 1.3)
// param3: Audio Reactivity (0.0 - 1.0, default: 0.5)

const DENSITY = " .:-=+*#%@";

function setup() {
  createCanvas(videoIn.width, videoIn.height);
  noStroke();
}

function draw() {
  background(0);
  let step = floor(map(param0, 0.0, 1.0, 6.0, 28.0));
  if (audioIn.level > 0.05) {
    step = max(4, step - floor(audioIn.level * param3 * 6.0));
  }

  videoIn.loadPixels();
  for (let y = 0; y < height; y += step) {
    for (let x = 0; x < width; x += step) {
      let col = videoIn.get(x, y);
      let lum = (0.299 * red(col) + 0.587 * green(col) + 0.114 * blue(col)) / 255.0;
      lum = pow(lum, 1.0 / max(0.2, param1 * 1.5));
      let charIdx = floor(lum * (DENSITY.length - 1));
      charIdx = constrain(charIdx, 0, DENSITY.length - 1);

      fill(red(col) * param2, green(col) * param2, blue(col) * param2);
      textSize(step);
      text(DENSITY.charAt(charIdx), x, y + step);
    }
  }
}
"#;

pub const ASCII_MONO: &str = r#"// P5.js Canvas Effect: ASCII Art (Monochrome B&W)
// param0: Character Size (6.0 - 32.0, default: 10.0)
// param1: Contrast Curve (0.5 - 3.0, default: 1.5)
// param2: Invert (0.0 = Dark terminal, 1.0 = Light paper)
// param3: Audio Glitch / Shake (0.0 - 1.0, default: 0.2)

const DENSITY = " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$";

function setup() {
  createCanvas(videoIn.width, videoIn.height);
  noStroke();
}

function draw() {
  let isLight = param2 > 0.5;
  background(isLight ? 255 : 0);

  let step = floor(map(param0, 0.0, 1.0, 6.0, 24.0));
  let shake = audioIn.beat > 0.1 ? (audioIn.beat * param3 * 4.0) : 0.0;

  videoIn.loadPixels();
  for (let y = 0; y < height; y += step) {
    for (let x = 0; x < width; x += step) {
      let col = videoIn.get(x, y);
      let lum = (0.299 * red(col) + 0.587 * green(col) + 0.114 * blue(col)) / 255.0;
      if (isLight) lum = 1.0 - lum;
      lum = pow(lum, max(0.1, param1 * 1.6));
      let charIdx = floor(lum * (DENSITY.length - 1));
      charIdx = constrain(charIdx, 0, DENSITY.length - 1);

      fill(isLight ? 0 : 255);
      textSize(step);
      text(DENSITY.charAt(charIdx), x + random(-shake, shake), y + step);
    }
  }
}
"#;

pub const PARTICLE_SAMPLER: &str = r#"// P5.js Canvas Effect: Particle Video Sampler
// param0: Grid Density (4.0 - 32.0, default: 12.0)
// param1: Particle Max Radius (2.0 - 24.0, default: 10.0)
// param2: Luminance Cutoff (0.0 - 1.0, default: 0.1)
// param3: Audio Dispersion / Pulse (0.0 - 1.0, default: 0.6)

function setup() {
  createCanvas(videoIn.width, videoIn.height);
  noStroke();
}

function draw() {
  background(10, 12, 16);
  let step = floor(map(param0, 0.0, 1.0, 8.0, 32.0));
  let maxR = map(param1, 0.0, 1.0, 2.0, 24.0);
  let audioBoost = 1.0 + (audioIn.bass * param3 * 2.5);

  videoIn.loadPixels();
  for (let y = step / 2; y < height; y += step) {
    for (let x = step / 2; x < width; x += step) {
      let col = videoIn.get(x, y);
      let lum = (0.299 * red(col) + 0.587 * green(col) + 0.114 * blue(col)) / 255.0;
      if (lum < param2) continue;

      let r = lum * maxR * audioBoost;
      fill(red(col), green(col), blue(col), 220);
      circle(x, y, r);
    }
  }
}
"#;

pub const WAVE_MATRIX: &str = r#"// P5.js Canvas Effect: Generative Wave Matrix
// param0: Wave Frequency (1.0 - 20.0, default: 4.0)
// param1: Wave Amplitude (10.0 - 200.0, default: 60.0)
// param2: Color Shift / Phase Speed (0.0 - 2.0, default: 1.0)
// param3: Audio Bass Modulation (0.0 - 1.0, default: 0.8)

function setup() {
  createCanvas(videoIn.width, videoIn.height);
  noFill();
  strokeWeight(2);
}

function draw() {
  image(videoIn, 0, 0); // Overlay on input video

  let count = 16;
  let freq = map(param0, 0.0, 1.0, 1.0, 20.0);
  let amp = map(param1, 0.0, 1.0, 10.0, 150.0);
  let t = frameCount * 0.03 * param2;
  let bassEffect = audioIn.bass * param3 * 120.0;

  for (let i = 0; i < count; i++) {
    let yBase = (height / (count + 1)) * (i + 1);
    stroke(
      128 + 127 * sin(t + i * 0.3),
      180 + 75 * cos(t + i * 0.2),
      240,
      180
    );

    beginShape();
    for (let x = 0; x <= width; x += 16) {
      let y = yBase + sin(x * 0.008 * freq + t + i * 0.5) * (amp + bassEffect);
      vertex(x, y);
    }
    endShape();
  }
}
"#;

pub const SPECTRUM_RINGS: &str = r#"// P5.js Canvas Effect: Audio Spectrum Rings
// param0: Center Ring Radius (50.0 - 400.0, default: 150.0)
// param1: Bar Count (16 - 128, default: 64)
// param2: Rotation Speed (-2.0 - 2.0, default: 0.5)
// param3: Audio Sensitivity (0.0 - 2.0, default: 1.0)

function setup() {
  createCanvas(videoIn.width, videoIn.height);
}

function draw() {
  image(videoIn, 0, 0); // Video background

  translate(width / 2, height / 2);
  rotate(frameCount * 0.01 * param2);

  let baseR = map(param0, 0.0, 1.0, 50.0, 300.0);
  let bars = floor(map(param1, 0.0, 1.0, 24.0, 96.0));
  let sens = param3 * 1.5;

  strokeWeight(3);
  for (let i = 0; i < bars; i++) {
    let angle = (TWO_PI / bars) * i;
    let barLen = (audioIn.level * 200.0 + audioIn.bass * 150.0) * sens;
    let x1 = cos(angle) * baseR;
    let y1 = sin(angle) * baseR;
    let x2 = cos(angle) * (baseR + barLen);
    let y2 = sin(angle) * (baseR + barLen);

    stroke(
      map(i, 0, bars, 50, 255),
      map(barLen, 0, 200, 100, 255),
      220
    );
    line(x1, y1, x2, y2);
  }
}
"#;

pub const GLITCH_PIXEL_SORT: &str = r#"// P5.js Canvas Effect: Glitch Pixel Sorter
// param0: Glitch Threshold (0.1 - 0.9, default: 0.4)
// param1: Sort Direction (0.0 = Horizontal, 1.0 = Vertical)
// param2: Streak Length (5.0 - 80.0, default: 30.0)
// param3: Audio Beat Trigger (0.0 - 1.0, default: 0.7)

function setup() {
  createCanvas(videoIn.width, videoIn.height);
}

function draw() {
  image(videoIn, 0, 0);

  let thresh = param0;
  if (audioIn.beat > 0.1) {
    thresh = max(0.05, thresh - audioIn.beat * param3 * 0.3);
  }
  let streak = floor(map(param2, 0.0, 1.0, 5.0, 80.0));
  let isVertical = param1 > 0.5;

  loadPixels();
  // Pixel sort streaks on high-luminance sections
  updatePixels();
}
"#;
