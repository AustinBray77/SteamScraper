use std::cmp::min;

struct MinHeap<T: Eq + Ord> {
    data: Vec<T>,
    len: usize,
}

impl<T: Eq + Ord> MinHeap<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    pub fn sink(&mut self, index: usize) {
        if let Some(smallest_child) = self.smallest_child(index) {
            if self.data[index] > self.data[smallest_child] {
                self.swap(index, smallest_child);
                self.sink(smallest_child);
            }
        }
    }

    pub fn swim(&mut self, index: usize) {
        if let Some(parent) = self.parent(index) {
            if self.data[parent] > self.data[index] {
                self.swap(parent, index);
                self.swim(parent);
            }
        }
    }

    pub fn insert(&mut self, item: T) {
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
        let mut result = Vec::with_capacity(real_amount);
        let cur_index = 0_usize;

        while let Some(next) = self.data.pop() {
            result[cur_index] = next;
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

    fn smallest_child(&self, index: usize) -> Option<usize> {
        let (left, right) = self.children(index);

        if left >= self.data.len() {
            None
        } else if right >= self.data.len() {
            Some(left)
        } else if self.data[left] > self.data[right] {
            Some(right)
        } else {
            Some(left)
        }
    }

    fn children(&self, index: usize) -> (usize, usize) {
        let right_child = (index + 1) * 2;
        (right_child - 1, right_child)
    }
}
