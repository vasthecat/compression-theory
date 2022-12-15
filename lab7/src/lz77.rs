pub fn lz_match(data: &Vec<u8>, pos1: usize, pos2: usize, length: usize) -> bool {
    for i in 0..length {
        if data[pos1 + i] != data[pos2 + i] {
            return false;
        }
    }
    return true;
}

pub fn lz77_encode(data: &Vec<u8>, window_size: usize) -> Vec<(usize, usize, u8)> {
    let mut ptr = 0;
    let mut encoded = Vec::new();

    while ptr < data.len() {
        let mut saved = None;
        let mut l = 1;
        while ptr + l < data.len() && l < 256 {
            let mut found = false;
            for offset in 1..window_size {
                if (ptr as i32) - (offset as i32) < 0 {
                    break;
                }
                if lz_match(data, ptr - offset as usize, ptr, l) {
                    found = true;
                    saved = Some((l, offset));
                    l += 1;
                    break;
                }
            }
            if !found {
                break;
            }
        }
        if let Some((l, offset)) = saved {
            encoded.push((offset, l, data[ptr + l]));
            ptr += l + 1;
        } else {
            encoded.push((0, 0, data[ptr]));
            ptr += 1;
        }
    }

    return encoded;
}

pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let encoded = lz77_encode(data, 256);

    for (offset, length, byte) in &encoded {
        result.push(*offset as u8);
        result.push(*length as u8);
        result.push(*byte);
    }

    return result;
}

pub fn decompress(archive: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let mut ptr: usize = 0;
    for i in 0..archive.len() / 3 {
        let offset = archive[i * 3 + 0] as usize;
        let length = archive[i * 3 + 1] as usize;
        let byte = archive[i * 3 + 2];
        if length == 0 {
            result.push(byte);
            ptr += 1;
        } else {
            for j in 0..length {
                result.push(result[ptr - offset + j]);
            }
            result.push(byte);
            ptr += length + 1;
        }
    }
    return result;
}
