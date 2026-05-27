use std::ops::{Index, IndexMut};

#[derive(Clone, Debug)]
pub struct RenderBuffer<E: Clone> {
    width: usize,
    height: usize,
    data: Vec<E>,
}

impl<E: Clone> RenderBuffer<E> {
    pub fn new(width: usize, height: usize, default: E) -> Self {
        Self {
            width,
            height,
            data: vec![default; width * height],
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
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut E {
        &mut self.data[x + y * self.width]
    }
}

impl<E: Clone> Index<(usize, usize)> for RenderBuffer<E> {
    type Output = E;
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.data[x + y * self.width]
    }
}

impl<E: Clone> IndexMut<(usize, usize)> for RenderBuffer<E> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        &mut self.data[x + y * self.width]
    }
}
