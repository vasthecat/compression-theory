fn dump_unique(encoded: &mut Vec<i8>, unique: &mut Vec<u8>) {
    let mut ptr = 0;
    while ptr < unique.len() {
        let l = std::cmp::min(127, unique.len() - ptr);
        encoded.push(-(l as i8));
        for i in 0..l {
            encoded.push(unique[ptr + i] as i8);
        }
        ptr += l;
    }
}

fn dump_repeat(encoded: &mut Vec<i8>, byte: u8, repeat: usize) {
    let mut ptr = 0;
    while ptr < repeat {
        let l = std::cmp::min(127, repeat - ptr);
        encoded.push(l as i8);
        encoded.push(byte as i8);
        ptr += l;
    }
}

fn rle_encode(data: &Vec<u8>) -> Vec<i8> {
    let mut encoded = Vec::new();

    let mut last = data[0];
    let mut count = 1;
    let mut unique = Vec::new();

    let mut ptr = 1;
    while ptr < data.len() {
        let byte = data[ptr];
        if byte == last {
            count += 1;
        } else {
            if count == 1 {
                unique.push(last);
            } else {
                dump_unique(&mut encoded, &mut unique);
                unique.clear();
                dump_repeat(&mut encoded, last, count);
            }
            last = byte;
            count = 1;
        }
        ptr += 1;
    }
    dump_unique(&mut encoded, &mut unique);
    dump_repeat(&mut encoded, last, count);

    return encoded;
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
    let r_enc = rle_encode(&r);
    let g_enc = rle_encode(&g);
    let b_enc = rle_encode(&b);

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
    let enc = rle_encode(data);

    result.push(0);
    dump_u32(&mut result, dim.0);
    dump_u32(&mut result, dim.1);

    dump_u32(&mut result, enc.len() as u32);
    for elem in enc {
        result.push(elem as u8);
    }

    return result;
}

fn rle_decode(data: &Vec<i8>) -> Vec<u8> {
    let mut result = Vec::new();

    let mut ptr = 0;
    while ptr < data.len() {
        let repeat = data[ptr];
        ptr += 1;
        if repeat < 0 {
            for i in 0..(repeat as i32).abs() {
                result.push(data[ptr + i as usize] as u8);
            }
            ptr += repeat.abs() as usize;
        } else {
            let byte = data[ptr];
            ptr += 1;
            for _ in 0..repeat {
                result.push(byte as u8);
            }
        }
    }

    return result;
}

pub fn decompress(data: &Vec<u8>) -> (Vec<u8>, (u32, u32)) {
    let mut result = Vec::new();
    let is_gray = data[0] == 0;
    let width = read_u32(&data, 1);
    let height = read_u32(&data, 5);

    if is_gray {
        let mut archive = Vec::new();
        for i in &data[13..] {
            archive.push(*i as i8);
        }
        let mut decoded = rle_decode(&archive);
        result.append(&mut decoded);
    } else {
        let mut shift = 9;
        let rsize = read_u32(&data, shift) as usize;
        shift += 4;
        let mut r_archive = Vec::new();
        for i in &data[shift..shift + rsize] {
            r_archive.push(*i as i8);
        }
        shift += rsize;

        let gsize = read_u32(&data, shift) as usize;
        shift += 4;
        let mut g_archive = Vec::new();
        for i in &data[shift..shift + gsize] {
            g_archive.push(*i as i8);
        }
        shift += gsize;

        let bsize = read_u32(&data, shift) as usize;
        shift += 4;
        let mut b_archive = Vec::new();
        for i in &data[shift..shift + bsize] {
            b_archive.push(*i as i8);
        }

        let r_decode = rle_decode(&r_archive);
        let g_decode = rle_decode(&g_archive);
        let b_decode = rle_decode(&b_archive);

        for i in 0..r_decode.len() {
            result.push(r_decode[i]);
            result.push(g_decode[i]);
            result.push(b_decode[i]);
        }
    }

    return (result, (width, height));
}
