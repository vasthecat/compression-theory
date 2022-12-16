const DICT_SIZE: usize = 4096;

pub fn lz_match(data: &Vec<u8>, pos1: usize, pos2: usize, length: usize) -> bool {
    for i in 0..length {
        if data[pos1 + i] != data[pos2 + i] {
            return false;
        }
    }
    return true;
}

fn lzw_encode(data: &Vec<u8>) -> Vec<(u32, u8)> {
    let mut encoded = Vec::new();
    let mut dict = Vec::new();
    let mut count = 1;

    let mut ptr = 0;
    while ptr < data.len() {
        let mut saved = None;
        let mut l = 1;
        while ptr + l < data.len() && l < 256 {
            let mut found = false;
            for i in (0..dict.len()).rev() {
                let (pos1, l1, val) = dict[i];
                if l != l1 {
                    continue;
                }
                if lz_match(data, pos1, ptr, l) {
                    found = true;
                    saved = Some((pos1, l, val));
                    l += 1;
                    break;
                }
            }
            if !found {
                break;
            }
        }
        if let Some((_, l, val)) = saved {
            let t = (val, data[ptr + l]);
            encoded.push(t);
            if dict.len() < DICT_SIZE {
                dict.push((ptr, l + 1, count));
                count += 1;
            }
            ptr += l + 1;
        } else {
            encoded.push((0, data[ptr]));
            if dict.len() < DICT_SIZE {
                dict.push((ptr, 1, count));
                count += 1;
            }
            ptr += 1;
        }
    }

    return encoded;
}

pub fn lzw_compress(data: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let encoded = lzw_encode(data);

    for (value, byte) in &encoded {
        let value_bytes: [u8; 4] = unsafe { std::mem::transmute(*value) };
        result.push(value_bytes[0]);
        result.push(value_bytes[1]);
        result.push(*byte);
    }

    return result;
}

pub fn lzw_decompress(data: &Vec<u8>) -> Vec<u8> {
    let mut decoded = Vec::new();
    let mut dict = Vec::new();
    let mut count = 1;
    let mut ptr = 0;

    let mut caret = 0;
    while caret < data.len() {
        let value_bytes: [u8; 4] = [data[caret + 0], data[caret + 1], 0, 0];
        let value: u32 = unsafe { std::mem::transmute(value_bytes) };
        let byte = data[caret + 2];
        caret += 3;

        if value == 0 {
            if dict.len() < DICT_SIZE {
                dict.push((ptr, 1, count));
                count += 1;
            }
            decoded.push(byte);
            ptr += 1;
        } else {
            let (pos1, l, _) = dict[value as usize - 1];
            for i in 0..l {
                decoded.push(decoded[pos1 + i]);
            }
            decoded.push(byte);
            if dict.len() < DICT_SIZE {
                dict.push((ptr, l + 1, count));
                count += 1;
            }
            ptr += l + 1;
        }
    }

    return decoded;
}

fn split_data(data: &Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut r = Vec::new();
    let mut g = Vec::new();
    let mut b = Vec::new();

    for i in 0..data.len() / 3 {
        r.push(data[i * 3 + 0]);
        g.push(data[i * 3 + 1]);
        b.push(data[i * 3 + 2]);
    }

    return (r, g, b);
}

fn dump_u32(data: &mut Vec<u8>, val: u32) {
    let val_bytes: [u8; 4] = unsafe { std::mem::transmute(val) };
    data.push(val_bytes[0]);
    data.push(val_bytes[1]);
    data.push(val_bytes[2]);
    data.push(val_bytes[3]);
}

fn read_u32(data: &Vec<u8>, offset: usize) -> u32 {
    let val_bytes = [
        data[offset + 0],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ];
    return unsafe { std::mem::transmute(val_bytes) };
}

pub fn compress_rgb(data: &Vec<u8>, dim: (u32, u32)) -> Vec<u8> {
    let mut result = Vec::new();
    let (r, g, b) = split_data(data);
    let r_enc = lzw_compress(&r);
    let g_enc = lzw_compress(&g);
    let b_enc = lzw_compress(&b);

    result.push(1);
    dump_u32(&mut result, dim.0);
    dump_u32(&mut result, dim.1);

    dump_u32(&mut result, r_enc.len() as u32);
    for elem in r_enc {
        result.push(elem as u8);
    }

    dump_u32(&mut result, g_enc.len() as u32);
    for elem in g_enc {
        result.push(elem as u8);
    }

    dump_u32(&mut result, b_enc.len() as u32);
    for elem in b_enc {
        result.push(elem as u8);
    }

    return result;
}

pub fn compress_gray(data: &Vec<u8>, dim: (u32, u32)) -> Vec<u8> {
    let mut result = Vec::new();
    let enc = lzw_compress(data);

    result.push(0);
    dump_u32(&mut result, dim.0);
    dump_u32(&mut result, dim.1);

    dump_u32(&mut result, enc.len() as u32);
    for elem in enc {
        result.push(elem as u8);
    }

    return result;
}

pub fn decompress(data: &Vec<u8>) -> (Vec<u8>, (u32, u32)) {
    let mut result = Vec::new();
    let is_gray = data[0] == 0;
    let width = read_u32(&data, 1);
    let height = read_u32(&data, 5);

    if is_gray {
        let archive = Vec::from(&data[13..]);
        let mut decoded = lzw_decompress(&archive);
        result.append(&mut decoded);
    } else {
        let mut shift = 9;
        let rsize = read_u32(&data, shift) as usize;
        shift += 4;
        let r_archive = Vec::from(&data[shift..shift + rsize]);
        shift += rsize;

        let gsize = read_u32(&data, shift) as usize;
        shift += 4;
        let g_archive = Vec::from(&data[shift..shift + gsize]);
        shift += gsize;

        let bsize = read_u32(&data, shift) as usize;
        shift += 4;
        let b_archive = Vec::from(&data[shift..shift + bsize]);

        let r_decode = lzw_decompress(&r_archive);
        let g_decode = lzw_decompress(&g_archive);
        let b_decode = lzw_decompress(&b_archive);

        for i in 0..r_decode.len() {
            result.push(r_decode[i]);
            result.push(g_decode[i]);
            result.push(b_decode[i]);
        }
    }

    return (result, (width, height));
}
