const DICT_SIZE: usize = 4096;

pub fn lz_match(data: &Vec<u8>, pos1: usize, pos2: usize, length: usize) -> bool {
    for i in 0..length {
        if data[pos1 + i] != data[pos2 + i] {
            return false;
        }
    }
    return true;
}

fn lz78_encode(data: &Vec<u8>) -> Vec<(u32, u8)> {
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

pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let encoded = lz78_encode(data);

    for (value, byte) in &encoded {
        let value_bytes: [u8; 4] = unsafe { std::mem::transmute(*value) };
        result.push(value_bytes[0]);
        result.push(value_bytes[1]);
        result.push(*byte);
    }

    return result;
}

pub fn decompress(data: &Vec<u8>) -> Vec<u8> {
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
