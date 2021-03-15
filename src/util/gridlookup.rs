pub struct GridLookup<T: Copy> {
    xmax: f64,
    ymax: f64,
    xstep: f64,
    ystep: f64,
    xbins: usize,
    ybins: usize,
    data: Vec<Vec<(f64, f64, T)>>,
}

impl<T: Copy> GridLookup<T> {
    pub fn new(xmax: f64, ymax: f64, xstep: f64, ystep: f64) -> Self {
        let xbins = (xmax / xstep).ceil() as usize;
        let ybins = (ymax / ystep).ceil() as usize;

        let data = vec![vec![]; xbins * ybins];

        Self {
            xmax,
            ymax,
            xstep,
            ystep,
            xbins,
            ybins,
            data,
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.xbins + x
    }

    pub fn put(&mut self, coord: (f64, f64), val: T) {
        let (x, y) = coord;
        let xbin = ((x % self.xmax) / self.xstep).floor() as usize;
        let ybin = ((y % self.ymax) / self.ystep).floor() as usize;
        let indx = self.index(xbin, ybin);
        self.data[indx].push((x, y, val))
    }

    pub fn get_within_step<V, F: Fn(V, (f64, f64, &T)) -> V>(
        &self,
        coord: (f64, f64),
        init: V,
        fold_fn: F,
    ) -> V {
        let (x, y) = coord;
        let xbin = (x / self.xstep).floor() as i64;
        let ybin = (y / self.ystep).floor() as i64;

        let xbins = self.xbins as i64;
        let ybins = self.ybins as i64;
        [-1, 0, 1].iter().fold(init, |acc, dx| {
            [-1, 0, 1].iter().fold(acc, |acc, dy| {
                let xb = (xbins + dx + xbin) % xbins;
                let yb = (ybins + dy + ybin) % ybins;
                let indx = self.index(xb as usize, yb as usize);
                self.data[indx]
                    .iter()
                    .filter_map(|(cx, cy, t)| {
                        let mut cx = *cx;
                        let mut cy = *cy;

                        let mut dx = (cx - x).abs() % self.xmax;
                        let mut dy = (cy - y).abs() % self.ymax;

                        if dx > self.xmax / 2. {
                            dx = self.xmax - dx;
                            if cx > x {
                                cx -= self.xmax;
                            } else {
                                cx += self.xmax;
                            }
                        }
                        if dy > self.ymax / 2. {
                            dy = self.ymax - dy;
                            if cy > y {
                                cy -= self.ymax;
                            } else {
                                cy += self.ymax;
                            }
                        }

                        if dx <= self.xstep && dy <= self.ystep {
                            Some((cx, cy, t))
                        } else {
                            None
                        }
                    })
                    .fold(acc, |acc, (cx, cy, t)| fold_fn(acc, (cx, cy, t)))
            })
        })
    }

    pub fn clear(&mut self) {
        self.data.iter_mut().for_each(|v| v.clear())
    }
}

#[cfg(test)]
mod grid_tests {
    use super::*;

    #[test]
    fn basic_test() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 0)
    }

    #[test]
    fn basic_test_close() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        grid.put((0.9, 0.), 0);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 1)
    }

    #[test]
    fn basic_test_far() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        grid.put((1.1, 0.), 0);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 0)
    }

    #[test]
    fn basic_test_close_wrap() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        grid.put((9.1, 0.), 0);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 1)
    }

    #[test]
    fn double_test_close_wrap() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        grid.put((0.9, 0.), 0);
        grid.put((9.1, 0.), 1);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 2)
    }

    #[test]
    fn test_close_far_wrap() {
        let mut grid = GridLookup::<usize>::new(10., 10., 1., 1.);
        grid.put((0.9, 0.), 0);
        grid.put((9.1, 0.), 1);
        grid.put((0., 0.9), 2);
        grid.put((0., 1.1), 3);
        let count = grid.get_within_step((0., 0.), 0, |acc, _| acc + 1);
        assert_eq!(count, 3)
    }
}
