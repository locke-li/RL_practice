use std::ops::{Index, IndexMut};

pub struct NdVec1<T> {
    pub offset: i32,
    pub dimension: i32,
    pub data: Vec<T>,
}

impl<T> NdVec1<T> {
    pub fn new(x: (i32, i32)) -> Self {
        assert!(x.1 >= x.0);
        let offset = -x.0;
        let dimension = x.1 - x.0;
        Self { offset, dimension, data: Vec::new() }
    }

    pub fn index(&self, i:i32) -> usize {
        (i + self.offset) as usize
    }

    pub fn push(&mut self, v:T) {
        self.data.push(v);
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }
}

impl<T> Index<i32> for NdVec1<T> {
    type Output = T;

    fn index(&self, index:i32) -> &Self::Output {
        &self.data[self.index(index)]
    }
}

impl<T> IndexMut<i32> for NdVec1<T> {
    fn index_mut(&mut self, index:i32) -> &mut Self::Output {
        let i = self.index(index);
        &mut self.data[i]
    }
}

pub struct NdVec2<T> {
    pub offset: (i32, i32),
    pub dimension: (i32, i32),
    pub data: Vec<T>,
}

impl<T> NdVec2<T> {
    pub fn new(x: (i32, i32), y: (i32, i32)) -> Self {
        assert!(x.1 >= x.0 && y.1 >= y.0);
        let offset = (-x.0, -y.0);
        let dimension = (x.1 - x.0 + 1, y.1 - y.0 + 1);
        Self { offset, dimension, data: Vec::new() }
    }

    pub fn resize(&mut self, new_len:usize, value:T)
        where T: Clone {
        self.data.resize(new_len, value);
    }

    pub fn index(&self, i:(i32, i32)) -> usize {
        (i.0 + self.offset.0 + (i.1 + self.offset.1) * self.dimension.0) as usize
    }

    pub fn push(&mut self, v:T) {
        self.data.push(v);
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
}

impl<T> Index<(i32, i32)> for NdVec2<T> {
    type Output = T;

    fn index(&self, index:(i32, i32)) -> &Self::Output {
        &self.data[self.index(index)]
    }
}

impl<T> IndexMut<(i32, i32)> for NdVec2<T> {
    fn index_mut(&mut self, index:(i32, i32)) -> &mut Self::Output {
        let i = self.index(index);
        &mut self.data[i]
    }
}