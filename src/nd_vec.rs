use std::ops::{Index, IndexMut};

type Vec2 = (i32, i32);

pub struct NdVec1<T> {
    pub offset: i32,
    pub dimension: i32,
    pub data: Vec<T>,
}

impl<T> NdVec1<T> {
    pub fn new(x: Vec2) -> Self {
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
    pub offset: Vec2,
    pub dimension: Vec2,
    pub data: Vec<T>,
}

impl<T> NdVec2<T> {
    pub fn new(x: Vec2, y: Vec2) -> Self {
        assert!(x.1 >= x.0 && y.1 >= y.0);
        let offset = (-x.0, -y.0);
        let dimension = (x.1 - x.0 + 1, y.1 - y.0 + 1);
        Self { offset, dimension, data: Vec::new() }
    }

    pub fn from_size(size: (usize, usize)) -> Self {
        NdVec2::new((0, size.0 as i32 - 1), (0, size.1 as i32 - 1))
    }

    pub fn resize(&mut self, new_len:usize, value:T)
        where T: Clone {
        self.data.resize(new_len, value);
    }

    pub fn fill(&mut self, value:T)
        where T: Clone {
        let l = self.dimension.0 * self.dimension.1;
        self.resize(l as usize, value);
    }

    pub fn index(&self, i:&Vec2) -> usize {
        (i.0 + self.offset.0 + (i.1 + self.offset.1) * self.dimension.0) as usize
    }

    pub fn rev_index(&self, i:usize) -> Vec2 {
        let i = i as i32;
        (i % self.dimension.0 - self.offset.0, i / self.dimension.0 - self.offset.1)
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

impl<T> Index<Vec2> for NdVec2<T> {
    type Output = T;

    fn index(&self, index:Vec2) -> &Self::Output {
        &self.data[self.index(&index)]
    }
}

impl<T> Index<&Vec2> for NdVec2<T> {
    type Output = T;

    fn index(&self, index:&Vec2) -> &Self::Output {
        &self.data[self.index(index)]
    }
}

impl<T> IndexMut<Vec2> for NdVec2<T> {
    fn index_mut(&mut self, index:Vec2) -> &mut Self::Output {
        let i = self.index(&index);
        &mut self.data[i]
    }
}

impl<T> IndexMut<&Vec2> for NdVec2<T> {
    fn index_mut(&mut self, index:&Vec2) -> &mut Self::Output {
        let i = self.index(index);
        &mut self.data[i]
    }
}