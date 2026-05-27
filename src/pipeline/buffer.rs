use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct RenderBuffer<E: Clone, const N: usize = 1> {
    width: usize,
    height: usize,
    data: Vec<E>,
}

impl<E: Clone, const N: usize> RenderBuffer<E, N> {
    pub fn new(width: usize, height: usize, default: E) -> Self {
        Self {
            width,
            height,
            data: vec![default; width * height * N],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn clear(&mut self, value: E) {
        self.data.fill(value);
    }
    pub fn as_slice(&self) -> &[E] {
        &self.data
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize, i: usize) -> usize {
        (x + y * self.width) * N + i
    }

    #[inline]
    pub fn get(&self, idx: usize) -> &E {
        &self.data[idx]
    }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> &mut E {
        &mut self.data[idx]
    }
}

/// N == 1 时 [(x, y)] 访问像素。
impl<E: Clone> Index<(usize, usize)> for RenderBuffer<E, 1> {
    type Output = E;
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[x + y * self.width]
    }
}

impl<E: Clone> IndexMut<(usize, usize)> for RenderBuffer<E, 1> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.data[x + y * self.width]
    }
}

/// N != 1 时 [(x, y, i)] 访问采样点。
impl<E: Clone, const N: usize> Index<(usize, usize, usize)> for RenderBuffer<E, N> {
    type Output = E;
    fn index(&self, (x, y, i): (usize, usize, usize)) -> &Self::Output {
        &self.data[self.idx(x, y, i)]
    }
}

impl<E: Clone, const N: usize> IndexMut<(usize, usize, usize)> for RenderBuffer<E, N> {
    fn index_mut(&mut self, (x, y, i): (usize, usize, usize)) -> &mut Self::Output {
        let idx = self.idx(x, y, i);
        &mut self.data[idx]
    }
}
