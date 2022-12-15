pub fn lz_match(data: &Vec<u8>, pos1: usize, pos2: usize, length: usize) -> bool {
    for i in 0..length {
        if data[pos1 + i] != data[pos2 + i] {
            return false;
        }
    }
    return true;
}

pub fn lz77_encode(data: &Vec<u8>, window_size: usize) -> Vec<(u32, u8, u8)> {
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
            encoded.push((offset as u32, l as u8, data[ptr + l]));
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
    let encoded = lz77_encode(data, 8192);

    for (offset, length, byte) in &encoded {
        let offset_bytes: [u8; 4] = unsafe { std::mem::transmute(*offset) };
        result.push(offset_bytes[0]);
        result.push(offset_bytes[1]);

        result.push(*length);
        result.push(*byte);
    }

    return result;
}

pub fn decompress(archive: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let mut ptr: usize = 0;
    let block_size = 4;
    for i in 0..archive.len() / block_size {
        let b_offset = i * block_size;

        let offset_bytes: [u8; 4] = [archive[b_offset + 0], archive[b_offset + 1], 0, 0];
        let offset: u32 = unsafe { std::mem::transmute(offset_bytes) };
        let length = archive[b_offset + 2] as usize;
        let byte = archive[b_offset + 3];

        if length == 0 {
            result.push(byte);
            ptr += 1;
        } else {
            for j in 0..length as usize {
                result.push(result[ptr - offset as usize + j]);
            }
            result.push(byte);
            ptr += length as usize + 1;
        }
    }
    return result;
}
