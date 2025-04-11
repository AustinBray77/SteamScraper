use std::{cmp::min, collections::HashSet, hash::Hash, vec::IntoIter};

pub enum Order {
    Greater,
    Equal,
    Smaller,
}

#[derive(Clone, Debug)]
pub struct MaxHeap<T: Clone, U: Hash + Eq + Clone> {
    data: Vec<T>,
    keys: HashSet<U>,
    len: usize,
    cmp: fn(&T, &T) -> Order,
    key: fn(&T) -> &U,
}

impl<T: Clone, U: Hash + Eq + Clone> MaxHeap<T, U> {
    pub fn new(cmp: fn(&T, &T) -> Order, key: fn(&T) -> &U) -> Self {
        Self {
            data: Vec::new(),
            keys: HashSet::new(),
            len: 0,
            cmp: cmp,
            key: key,
        }
    }

    pub fn new_similar(&self) -> Self {
        Self {
            data: Vec::new(),
            keys: HashSet::new(),
            len: 0,
            cmp: self.cmp.clone(),
            key: self.key.clone(),
        }
    }

    pub fn sink(&mut self, index: usize) {
        if let Some(smallest_child) = self.largest_child(index) {
            if let Order::Smaller = (self.cmp)(&self.data[index], &self.data[smallest_child]) {
                self.swap(index, smallest_child);
                self.sink(smallest_child);
            }
        }
    }

    pub fn swim(&mut self, index: usize) {
        if let Some(parent) = self.parent(index) {
            if let Order::Smaller = (self.cmp)(&self.data[parent], &self.data[index]) {
                self.swap(parent, index);
                self.swim(parent);
            }
        }
    }

    pub fn in_heap(&self, item: &T) -> bool {
        self.keys.contains((self.key)(item))
    }

    pub fn insert(&mut self, item: T) {
        if self.in_heap(&item) {
            return;
        }

        self.keys.insert((self.key)(&item).clone());

        self.data.push(item);
        self.len += 1;

        self.swim(self.len - 1);
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.get(0)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }

        self.data.swap(0, self.len - 1);

        let result = self.data.pop();
        self.len -= 1;

        self.sink(0);

        return result;
    }

    pub fn pop_many(&mut self, amount: usize) -> Vec<T> {
        let real_amount = min(self.len, amount);
        let mut result = Vec::new();
        let mut cur_index = 0_usize;

        while let Some(next) = self.pop() {
            result.push(next);
            cur_index += 1;

            if cur_index >= real_amount {
                break;
            }
        }

        return result;
    }

    fn swap(&mut self, a: usize, b: usize) {
        self.data.swap(a, b);
    }

    fn parent(&self, index: usize) -> Option<usize> {
        let parent = (index + 1) / 2;

        if parent == 0 {
            None
        } else {
            Some(parent - 1)
        }
    }

    fn largest_child(&self, index: usize) -> Option<usize> {
        let (left, right) = self.children(index);

        if left >= self.data.len() {
            None
        } else if right >= self.data.len() {
            Some(left)
        } else if let Order::Greater = (self.cmp)(&self.data[left], &self.data[right]) {
            Some(left)
        } else {
            Some(right)
        }
    }

    fn children(&self, index: usize) -> (usize, usize) {
        let right_child = (index + 1) * 2;
        (right_child - 1, right_child)
    }

    pub fn unsorted_iter(&self) -> IntoIter<T> {
        self.data.clone().into_iter()
    }

    pub fn combine_with(&self, other: &MaxHeap<T, U>) -> Self {
        let mut new_heap = self.clone();

        for item in other.unsorted_iter() {
            new_heap.insert(item);
        }

        new_heap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn truncate(&mut self, size: usize) {
        self.data.truncate(size);
        self.len = self.data.len();
    }
}
