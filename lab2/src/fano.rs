use crate::weighted::Weighted;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
enum FanoTree<T>
where
    T: std::cmp::Eq + std::hash::Hash + Copy,
{
    Leaf(T),
    Node(Box<FanoTree<T>>, Box<FanoTree<T>>),
}

impl<T: std::cmp::Eq + std::hash::Hash + Copy> FanoTree<T> {
    fn prefix_sum(weights: &Vec<Weighted<T>>) -> Vec<u32> {
        let mut pf = Vec::new();
        for i in 0..weights.len() {
            if i == 0 {
                pf.push(weights[i].weight);
            } else {
                pf.push(weights[i].weight + pf[i - 1]);
            }
        }
        return pf;
    }

    fn from_weighs(weights: &Vec<Weighted<T>>) -> Self {
        let pf = FanoTree::prefix_sum(weights);
        let m = partition(&pf);

        let l = Vec::from(&weights[..m + 1]);
        let r = Vec::from(&weights[m + 1..]);

        let l_tree = if l.len() == 1 {
            FanoTree::Leaf(l.first().unwrap().value)
        } else {
            FanoTree::from_weighs(&l)
        };

        let r_tree = if r.len() == 1 {
            FanoTree::Leaf(r.first().unwrap().value)
        } else {
            FanoTree::from_weighs(&r)
        };

        return FanoTree::Node(Box::new(l_tree), Box::new(r_tree));
    }

    fn from_hashmap(map: &HashMap<T, u32>) -> Self {
        let mut weights = map
            .iter()
            .map(|(v, p)| Weighted {
                weight: *p,
                value: *v,
            })
            .collect::<Vec<Weighted<T>>>();
        weights.sort_by_key(|p| p.weight);
        return FanoTree::from_weighs(&weights);
    }

    fn get_code_rec(tree: &FanoTree<T>, codes: &mut HashMap<T, Vec<Bit>>, run: Vec<Bit>) {
        match tree {
            FanoTree::Node(left, right) => {
                let mut left_run = run.clone();
                left_run.push(Bit::Zero);
                FanoTree::get_code_rec(left, codes, left_run);

                let mut right_run = run.clone();
                right_run.push(Bit::One);
                FanoTree::get_code_rec(right, codes, right_run);
            }
            FanoTree::Leaf(value) => {
                codes.insert(*value, run.clone());
            }
        }
    }

    fn get_code(&self) -> HashMap<T, Vec<Bit>> {
        let mut code = HashMap::new();
        FanoTree::get_code_rec(&self, &mut code, Vec::new());
        return code;
    }
}

fn partition(pf: &Vec<u32>) -> usize {
    fn inner(pf: &Vec<u32>, l: usize, r: usize, prev: Option<(usize, u32)>) -> usize {
        let m = (l + r) / 2;
        let half = (r - l) / 2;
        let val1 = pf[m];
        let val2 = pf.last().unwrap() - pf[m];
        let diff = (val1 as i32 - val2 as i32).abs() as u32;

        if let Some(prev) = prev {
            if prev.1 < diff {
                return prev.0;
            }
        }

        if half == 0 {
            return m;
        }
        if val1 > val2 {
            return inner(pf, l, r - half, Some((m, diff)));
        } else {
            return inner(pf, l + half, r, Some((m, diff)));
        }
    }
    return inner(pf, 0, pf.len() - 1, None);
}

fn get_weights(data: &Vec<u8>) -> HashMap<u8, u32> {
    let mut freq = HashMap::new();
    data.iter().for_each(|byte| {
        *freq.entry(*byte).or_insert(1) += 1;
    });
    let mut freq: Vec<(u8, u32)> = freq.drain().collect();
    freq.sort();
    let mut weights = HashMap::new();
    freq.iter().enumerate().for_each(|(i, (byte, _))| {
        weights.insert(*byte, i as u32);
    });
    return weights;
}

#[derive(Debug)]
struct Metadata {
    weights: HashMap<u8, u32>,
    tree: FanoTree<u8>,
    code: HashMap<u8, Vec<Bit>>,
    remainder: u8,
}

impl Metadata {
    fn compute(data: &Vec<u8>) -> Self {
        let weights = get_weights(data);
        let tree = FanoTree::from_hashmap(&weights);
        let code = tree.get_code();

        return Self {
            weights,
            tree,
            code,
            remainder: 0,
        };
    }

    fn load(data: &Vec<u8>) -> Self {
        let remainder = data[0];
        let dict_len = data[1] as usize + 1;
        let mut weights = HashMap::new();
        for i in 0..dict_len {
            weights.insert(data[2 + 2 * i], data[2 + 2 * i + 1] as u32);
        }
        let tree = FanoTree::from_hashmap(&weights);
        let code = tree.get_code();

        return Self {
            weights,
            tree,
            code,
            remainder,
        };
    }

    fn dump(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.push(self.remainder);
        result.push((self.weights.len() - 1) as u8);
        for (byte, weight) in &self.weights {
            result.push(*byte);
            result.push(*weight as u8);
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

    let data = archive[2 + metadata.weights.len() * 2..].to_vec();
    let mut reader = BitReader::new(&data, &metadata);

    let mut state = &metadata.tree;
    while let Some(bit) = reader.read_bit() {
        if let FanoTree::Node(left, right) = state {
            match bit {
                Bit::Zero => state = left,
                Bit::One => state = right,
            }
        }

        if let FanoTree::Leaf(byte) = state {
            result.push(*byte);
            state = &metadata.tree;
        }
    }

    return result;
}
