use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug)]
pub struct Weighted<T> {
    pub weight: u32,
    pub value: T,
}

impl<T> PartialEq for Weighted<T> {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl<T> Eq for Weighted<T> {}

impl<T> PartialOrd for Weighted<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Weighted<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight).reverse()
    }
}

#[derive(Debug)]
pub struct PriorityQueue<T> {
    heap: BinaryHeap<Weighted<T>>,
}

impl<T> PriorityQueue<T> {
    pub fn new() -> Self {
        PriorityQueue {
            heap: BinaryHeap::new(),
        }
    }

    pub fn insert(&mut self, priority: u32, value: T) {
        self.heap.push(Weighted {
            weight: priority,
            value,
        });
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn pop(&mut self) -> Option<Weighted<T>> {
        Some(self.heap.pop()?)
    }
}
