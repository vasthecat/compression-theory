use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Weighted<T> {
    pub weight: f32,
    pub value: T,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Bit {
    Zero,
    One,
}
impl Bit {
    fn from_u8(x: u8) -> Bit {
        if x == 0 {
            Bit::Zero
        } else {
            Bit::One
        }
    }

    fn to_u8(x: &Bit) -> u8 {
        match *x {
            Bit::Zero => 0,
            Bit::One => 1,
        }
    }
}

fn get_probabilities(data: &Vec<u8>) -> Vec<Weighted<u8>> {
    let mut counts = HashMap::new();
    data.iter().for_each(|byte| {
        *counts.entry(*byte).or_insert(0) += 1;
    });
    let mut counts: Vec<(u8, u32)> = counts.drain().collect();
    counts.sort();

    let total = data.len() as f32;
    let mut weights = Vec::new();
    counts.iter().for_each(|(byte, count)| {
        weights.push(Weighted {
            value: *byte,
            weight: *count as f32 / total,
        });
    });
    return weights;
}

fn prefix_sum(weights: &Vec<Weighted<u8>>) -> Vec<f32> {
    let mut pf = Vec::new();
    for i in 0..weights.len() {
        if i == 0 {
            pf.push(0f32);
        } else {
            pf.push(weights[i - 1].weight + pf[i - 1]);
        }
    }
    return pf;
}

fn get_code(probabilities: &Vec<Weighted<u8>>) -> HashMap<u8, Vec<Bit>> {
    let mut code = HashMap::new();

    let mut ps = probabilities.clone();
    ps.sort_by(|a, b| a.weight.partial_cmp(&b.weight).unwrap_or(Ordering::Equal));
    ps.reverse();
    let prefix = prefix_sum(&ps);

    for i in 0..ps.len() {
        let l = (-ps[i].weight.log2()).ceil() as i32;
        if l == 0 {
            code.insert(ps[i].value, vec![Bit::Zero]);
        } else {
            let bits = get_bits(prefix[i], l);
            code.insert(ps[i].value, bits);
        }
    }

    return code;
}

fn get_bits(x: f32, count: i32) -> Vec<Bit> {
    let mut bits = Vec::new();

    let mut tmp = x;
    for _ in 0..count {
        tmp *= 2f32;
        if tmp >= 1f32 {
            bits.push(Bit::One);
            tmp -= 1f32;
        } else {
            bits.push(Bit::Zero);
        }
    }

    return bits;
}

#[derive(Debug)]
struct Metadata {
    probabilities: Vec<Weighted<u8>>,
    code: HashMap<u8, Vec<Bit>>,
    remainder: u8,
}

impl Metadata {
    fn compute(data: &Vec<u8>) -> Self {
        let probs = get_probabilities(&data);
        let code = get_code(&probs);

        return Self {
            probabilities: probs,
            code,
            remainder: 0,
        };
    }

    fn load(data: &Vec<u8>) -> Self {
        let remainder = data[0];
        let dict_len = data[1] as usize + 1;
        let mut probabilities = Vec::new();
        for i in 0..dict_len {
            let pstart = 2 + 5 * i + 1;
            let prob = [
                data[pstart + 0],
                data[pstart + 1],
                data[pstart + 2],
                data[pstart + 3],
            ];
            let prob: f32 = unsafe { std::mem::transmute(prob) };
            probabilities.push(Weighted {
                value: data[2 + 5 * i],
                weight: prob,
            });
        }
        let code = get_code(&probabilities);

        return Self {
            probabilities,
            code,
            remainder,
        };
    }

    fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.push(self.remainder);
        result.push((self.probabilities.len() - 1) as u8);
        for p in &self.probabilities {
            result.push(p.value);
            let bweight: [u8; 4] = unsafe { std::mem::transmute(p.weight) };
            for b in bweight {
                result.push(b);
            }
        }

        return result;
    }
}

struct BitWriter {
    buffer: Vec<Bit>,
    remainder: u8,
    result: Vec<u8>,
}

impl BitWriter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            remainder: 0,
            result: Vec::new(),
        }
    }

    fn dump_byte(&mut self) {
        let mut byte = 0_u8;
        self.buffer
            .iter()
            .map(Bit::to_u8)
            .enumerate()
            .for_each(|(i, bit)| byte |= bit << i);
        self.result.push(byte);
        self.buffer.clear();
    }

    fn write_bit(&mut self, bit: Bit) {
        self.buffer.push(bit);
        if self.buffer.len() == 8 {
            self.dump_byte();
        }
    }

    fn write_bits(&mut self, bits: &Vec<Bit>) {
        bits.iter().for_each(|bit| self.write_bit(*bit));
    }

    fn finish(&mut self) {
        let remainder = 8 - self.buffer.len() as u8 % 8;
        if remainder != 0 {
            self.dump_byte();
        }
        self.remainder = remainder;
    }
}

pub fn compress(data: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let mut metadata = Metadata::compute(data);

    let mut writer = BitWriter::new();
    data.iter().for_each(|byte| {
        let bits = metadata.code.get(byte).unwrap();
        writer.write_bits(&bits);
    });
    writer.finish();
    metadata.remainder = writer.remainder;

    let md_dump = metadata.dump();
    md_dump.iter().for_each(|byte| result.push(*byte));
    writer.result.iter().for_each(|byte| result.push(*byte));

    return result;
}

struct BitReader<'a> {
    data: &'a Vec<u8>,
    metadata: &'a Metadata,
    buffer: Vec<Bit>,
    ptr: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a Vec<u8>, metadata: &'a Metadata) -> Self {
        Self {
            data,
            metadata,
            buffer: Vec::new(),
            ptr: 0,
        }
    }

    fn read_byte(&mut self) {
        if let Some(byte) = self.data.get(self.ptr) {
            for i in 0..=7 {
                let bit = byte & (1 << i);
                self.buffer.push(Bit::from_u8(bit));
            }
            if self.ptr == self.data.len() - 1 {
                for _ in 0..self.metadata.remainder {
                    self.buffer.pop();
                }
            }
            self.buffer.reverse();
            self.ptr += 1;
        }
    }

    fn read_bit(&mut self) -> Option<Bit> {
        if self.buffer.len() == 0 {
            self.read_byte();
        }
        self.buffer.pop()
    }
}

pub fn decompress(archive: &Vec<u8>) -> Vec<u8> {
    let mut result = Vec::new();
    let metadata = Metadata::load(archive);

    let data = archive[2 + metadata.probabilities.len() * 5..].to_vec();
    let mut reader = BitReader::new(&data, &metadata);

    let mut run = Vec::new();
    while let Some(bit) = reader.read_bit() {
        run.push(bit);
        'checker: for p in &metadata.code {
            if p.1.len() != run.len() {
                continue;
            }
            for i in 0..run.len() {
                if p.1[i] != run[i] {
                    continue 'checker;
                }
            }
            result.push(*p.0);
            run.clear();
            break;
        }
    }

    return result;
}
