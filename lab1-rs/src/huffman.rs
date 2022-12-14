use crate::priority_queue::PriorityQueue;
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
enum HuffmanTree<T>
where
    T: std::cmp::Eq + std::hash::Hash + Copy,
{
    Leaf(T),
    Node(Box<HuffmanTree<T>>, Box<HuffmanTree<T>>),
}

impl<T: std::cmp::Eq + std::hash::Hash + Copy> HuffmanTree<T> {
    fn from_queue(mut queue: PriorityQueue<T>) -> Self {
        let mut huf_queue = PriorityQueue::new();
        while let Some(p) = queue.pop() {
            huf_queue.insert(p.weight, Box::new(HuffmanTree::Leaf(p.value)));
        }
        assert!(huf_queue.len() > 0);
        while huf_queue.len() > 1 {
            let v1 = huf_queue.pop().unwrap();
            let v2 = huf_queue.pop().unwrap();
            huf_queue.insert(
                v1.weight + v2.weight,
                Box::new(HuffmanTree::Node(v1.value, v2.value)),
            );
        }
        return *huf_queue.pop().unwrap().value;
    }

    fn dfs(tree: &HuffmanTree<T>, mut acc: Vec<Bit>, code: &mut HashMap<T, Vec<Bit>>) -> Vec<Bit> {
        match tree {
            HuffmanTree::Leaf(val) => {
                code.insert(*val, acc.clone());
            }
            HuffmanTree::Node(left, right) => {
                acc.push(Bit::Zero);
                acc = HuffmanTree::dfs(left, acc, code);
                acc.pop();
                acc.push(Bit::One);
                acc = HuffmanTree::dfs(right, acc, code);
                acc.pop();
            }
        };
        return acc;
    }

    fn get_code(&self) -> HashMap<T, Vec<Bit>> {
        let mut code = HashMap::new();
        if let HuffmanTree::Leaf(val) = &self {
            code.insert(*val, vec![Bit::One]);
        } else {
            HuffmanTree::dfs(self, Vec::new(), &mut code);
        }
        return code;
    }
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
    tree: HuffmanTree<u8>,
    code: HashMap<u8, Vec<Bit>>,
    remainder: u8,
}

impl Metadata {
    fn compute(data: &Vec<u8>) -> Self {
        let weights = get_weights(data);
        let mut queue = PriorityQueue::new();
        for (value, priority) in weights.iter() {
            queue.insert(*priority, *value);
        }
        let tree = HuffmanTree::from_queue(queue);
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

        let mut queue = PriorityQueue::new();
        for (value, priority) in weights.iter() {
            queue.insert(*priority, *value);
        }
        let tree = HuffmanTree::from_queue(queue);
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
        if let HuffmanTree::Node(left, right) = state {
            match bit {
                Bit::Zero => state = left,
                Bit::One => state = right,
            }
        }

        if let HuffmanTree::Leaf(byte) = state {
            result.push(*byte);
            state = &metadata.tree;
        }
    }

    return result;
}
