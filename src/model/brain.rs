use ndarray::{arr1, Array, Dim};
use ndarray_rand::rand_distr::Normal;
use ndarray_rand::RandomExt;
use std::cmp::max;

pub trait Brain<const INPUT: usize, const OUTPUT: usize> {
    fn clone_mutate(&self, std: f64) -> Self;
    fn feed(&self, inputs: &[f64; INPUT], outputs: &mut [f64; OUTPUT]);
}

pub struct NeuralBrain<const INPUT: usize, const OUTPUT: usize> {
    max_size: usize,
    mats: Vec<Array<f64, Dim<[usize; 2]>>>,
}

impl<const INPUT: usize, const OUTPUT: usize> Default for NeuralBrain<INPUT, OUTPUT> {
    fn default() -> Self {
        Self::new_random(&[7])
    }
}

impl<const INPUT: usize, const OUTPUT: usize> NeuralBrain<INPUT, OUTPUT> {
    pub fn new_random(shape: &[usize]) -> Self {
        let mut max_size = INPUT;
        let mut last_size = INPUT;
        let mut mats = vec![];
        let d = Normal::new(0., 1.).unwrap();
        shape.iter().cloned().for_each(|next_size| {
            let mat = Array::random([next_size, last_size], d);
            mats.push(mat);
            last_size = next_size;
            max_size = max(max_size, next_size);
        });
        let mat = Array::random([OUTPUT, last_size], d);
        mats.push(mat);
        max_size = max(max_size, OUTPUT);

        Self { max_size, mats }
    }
}

impl<const INPUT: usize, const OUTPUT: usize> Brain<INPUT, OUTPUT> for NeuralBrain<INPUT, OUTPUT> {
    fn clone_mutate(&self, std: f64) -> Self {
        let d = Normal::new(0., std).unwrap();
        let mats = self
            .mats
            .iter()
            .map(|m| {
                let shape = [m.shape()[0], m.shape()[1]];
                let diff = Array::random(shape, d);
                m + &diff
            })
            .collect::<Vec<_>>();

        Self {
            mats,
            max_size: self.max_size,
        }
    }

    fn feed(&self, inputs: &[f64; INPUT], outputs: &mut [f64; OUTPUT]) {
        let buff = arr1(inputs);
        let buff = self.mats[..self.mats.len() - 1]
            .iter()
            .fold(buff, |buff, mat| {
                let out = mat.dot(&buff);
                out.mapv(activation)
            });
        let buff = self.mats.last().unwrap().dot(&buff);

        outputs.iter_mut().enumerate().for_each(|(i, o)| {
            *o = buff[i];
        });
    }
}

fn activation(f: f64) -> f64 {
    f.tanh()
}
