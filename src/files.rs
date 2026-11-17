//! Small, real files made in code (no downloads, no extra crates): every demo asset can be downloaded and opened.

/// CRC-32 (IEEE), used by PNG chunks and ZIP entries.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// zlib stream with uncompressed ("stored") deflate blocks: valid everywhere, just not smaller.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut chunks = data.chunks(65_535).peekable();
    if chunks.peek().is_none() {
        out.extend_from_slice(&[1, 0, 0, 0xFF, 0xFF]);
    }
    while let Some(chunk) = chunks.next() {
        out.push(u8::from(chunks.peek().is_none()));
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// An RGB colour from a hue (0..360), full saturation, given lightness-ish value.
fn hue_rgb(hue: u32, value: f32) -> [u8; 3] {
    let h = (hue % 360) as f32 / 60.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    [
        (r * value * 255.0) as u8,
        (g * value * 255.0) as u8,
        (b * value * 255.0) as u8,
    ]
}

/// RGB PNG: diagonal stripes and a checker in two shades of `hue`, so every image looks different.
pub fn png(width: u32, height: u32, hue: u32) -> Vec<u8> {
    let (light, dark) = (hue_rgb(hue, 0.95), hue_rgb(hue + 30, 0.55));
    let mut raw = Vec::with_capacity(((width * 3 + 1) * height) as usize);
    for y in 0..height {
        raw.push(0); // filter: none
        for x in 0..width {
            let stripe = ((x + y) / 12) % 2 == 0;
            let checker = (x / 32 + y / 32) % 2 == 0;
            raw.extend_from_slice(if stripe ^ checker { &light } else { &dark });
        }
    }
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bit, RGB
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    png_chunk(&mut out, b"IHDR", &header);
    png_chunk(&mut out, b"IDAT", &zlib_stored(&raw));
    png_chunk(&mut out, b"IEND", &[]);
    out
}

/// 16-bit mono WAV, 22.05 kHz: a tone that fades out, with a second harmonic.
pub fn wav(frequency: f32, millis: u32) -> Vec<u8> {
    const RATE: u32 = 22_050;
    let samples = RATE * millis / 1000;
    let mut data = Vec::with_capacity(samples as usize * 2);
    for i in 0..samples {
        let t = i as f32 / RATE as f32;
        let fade = 1.0 - i as f32 / samples as f32;
        let wave = (t * frequency * std::f32::consts::TAU).sin() * 0.7
            + (t * frequency * 2.0 * std::f32::consts::TAU).sin() * 0.3;
        data.extend_from_slice(&((wave * fade * 0.6 * i16::MAX as f32) as i16).to_le_bytes());
    }
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    out
}

/// glTF 2.0 binary: a coloured box (24 vertices, so each face has its own normal). `scale` stretches it.
pub fn glb(hue: u32, scale: [f32; 3]) -> Vec<u8> {
    let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
        ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
        ([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        ([0.0, -1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ];
    let (mut positions, mut normals, mut indices) = (Vec::new(), Vec::new(), Vec::<u16>::new());
    for (n, u, v) in faces {
        let base = (positions.len() / 3) as u16;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            for axis in 0..3 {
                positions.push((n[axis] + u[axis] * su + v[axis] * sv) * 0.5 * scale[axis]);
                normals.push(n[axis]);
            }
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let mut bin = Vec::new();
    positions
        .iter()
        .for_each(|f| bin.extend_from_slice(&f.to_le_bytes()));
    normals
        .iter()
        .for_each(|f| bin.extend_from_slice(&f.to_le_bytes()));
    indices
        .iter()
        .for_each(|i| bin.extend_from_slice(&i.to_le_bytes()));
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let vertex_bytes = positions.len() * 4;
    let [r, g, b] = hue_rgb(hue, 0.9).map(|c| c as f32 / 255.0);
    let half = scale.map(|s| s * 0.5);
    let json = serde_json::json!({
        "asset": { "version": "2.0", "generator": "assetstore-demo" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0, "name": "Box" }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "material": 0 }] }],
        "materials": [{ "pbrMetallicRoughness": { "baseColorFactor": [r, g, b, 1.0], "metallicFactor": 0.1, "roughnessFactor": 0.6 } }],
        "buffers": [{ "byteLength": bin.len() }],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": vertex_bytes, "target": 34962 },
            { "buffer": 0, "byteOffset": vertex_bytes, "byteLength": vertex_bytes, "target": 34962 },
            { "buffer": 0, "byteOffset": vertex_bytes * 2, "byteLength": indices.len() * 2, "target": 34963 }
        ],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": 24, "type": "VEC3",
              "min": [-half[0], -half[1], -half[2]], "max": half },
            { "bufferView": 1, "componentType": 5126, "count": 24, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5123, "count": indices.len(), "type": "SCALAR" }
        ]
    });
    let mut json = serde_json::to_vec(&json).expect("static JSON");
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    let total = 12 + 8 + json.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin);
    out
}

/// ZIP with stored (uncompressed) entries.
pub fn zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let (mut out, mut central) = (Vec::new(), Vec::new());
    for (name, data) in entries {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        let mut common = Vec::new();
        common.extend_from_slice(&20u16.to_le_bytes()); // version needed
        common.extend_from_slice(&0u16.to_le_bytes()); // flags
        common.extend_from_slice(&0u16.to_le_bytes()); // stored
        common.extend_from_slice(&0u16.to_le_bytes()); // time
        common.extend_from_slice(&0x5D71u16.to_le_bytes()); // date 2026-11-17
        common.extend_from_slice(&crc.to_le_bytes());
        common.extend_from_slice(&size.to_le_bytes());
        common.extend_from_slice(&size.to_le_bytes());
        common.extend_from_slice(&(name.len() as u16).to_le_bytes());
        common.extend_from_slice(&0u16.to_le_bytes()); // extra
        out.extend_from_slice(&0x0403_4B50u32.to_le_bytes());
        out.extend_from_slice(&common);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4B50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // made by
        central.extend_from_slice(&common);
        central.extend_from_slice(&[0; 6]); // comment length, disk number, internal attributes
        central.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let central_offset = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4B50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Tiled map (TMX, CSV layer): a grass field with a river and a few walls.
pub fn tmx(width: u32, height: u32) -> Vec<u8> {
    let mut rows = Vec::new();
    for y in 0..height {
        let row: Vec<String> = (0..width)
            .map(|x| {
                let river = (x as i32 - (width / 2) as i32 + (y as i32 / 3)).abs() <= 1;
                let wall = (x % 7 == 0 && y % 5 != 0) && !river;
                (if river {
                    3
                } else if wall {
                    2
                } else {
                    1
                })
                .to_string()
            })
            .collect();
        rows.push(row.join(","));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<map version=\"1.10\" orientation=\"orthogonal\" renderorder=\"right-down\" \
width=\"{width}\" height=\"{height}\" tilewidth=\"16\" tileheight=\"16\" infinite=\"0\" nextlayerid=\"2\" nextobjectid=\"1\">\n \
<tileset firstgid=\"1\" name=\"terrain\" tilewidth=\"16\" tileheight=\"16\" tilecount=\"3\" columns=\"3\">\n  \
<image source=\"terrain.png\" width=\"48\" height=\"16\"/>\n </tileset>\n \
<layer id=\"1\" name=\"Ground\" width=\"{width}\" height=\"{height}\">\n  <data encoding=\"csv\">\n{}\n</data>\n </layer>\n</map>\n",
        rows.join(",\n")
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_the_reference_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn png_has_signature_and_ends_with_iend() {
        let image = png(64, 48, 200);
        assert_eq!(
            &image[..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
        assert_eq!(&image[image.len() - 8..image.len() - 4], b"IEND");
    }

    #[test]
    fn glb_header_length_matches() {
        let model = glb(120, [1.0, 2.0, 1.0]);
        assert_eq!(&model[..4], b"glTF");
        assert_eq!(
            u32::from_le_bytes(model[8..12].try_into().unwrap()) as usize,
            model.len()
        );
        let json_len = u32::from_le_bytes(model[12..16].try_into().unwrap()) as usize;
        let json: serde_json::Value = serde_json::from_slice(&model[20..20 + json_len]).unwrap();
        assert_eq!(json["asset"]["version"], "2.0");
    }

    #[test]
    fn wav_sizes_are_consistent() {
        let sound = wav(440.0, 500);
        assert_eq!(&sound[..4], b"RIFF");
        assert_eq!(
            u32::from_le_bytes(sound[4..8].try_into().unwrap()) as usize,
            sound.len() - 8
        );
    }

    #[test]
    fn zip_ends_with_the_end_record() {
        let archive = zip(&[("a.txt", b"hello".to_vec()), ("b.txt", b"world".to_vec())]);
        let end = archive.len() - 22;
        assert_eq!(&archive[end..end + 4], &0x0605_4B50u32.to_le_bytes());
        // Walk the central directory: each record is 46 bytes + name and must start with its signature.
        let mut at = u32::from_le_bytes(archive[end + 16..end + 20].try_into().unwrap()) as usize;
        for name in ["a.txt", "b.txt"] {
            assert_eq!(&archive[at..at + 4], &0x0201_4B50u32.to_le_bytes());
            let name_len =
                u16::from_le_bytes(archive[at + 28..at + 30].try_into().unwrap()) as usize;
            assert_eq!(&archive[at + 46..at + 46 + name_len], name.as_bytes());
            at += 46 + name_len;
        }
        assert_eq!(at, end);
    }
}
