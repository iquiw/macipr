trait ResettableIterator: Iterator {
    fn reset(&mut self);
}

struct IterBundle<I> {
    iters: Vec<I>,
}

struct BundledIter<I> {
    iters: Vec<I>,
    offset: i64,
    done: Vec<bool>,
}

impl<I> IterBundle<I> {
    fn new() -> Self {
        IterBundle::<I> { iters: Vec::new() }
    }

    fn push(&mut self, iter: I) {
        self.iters.push(iter);
    }
}

impl<I> IntoIterator for IterBundle<I>
where
    I: ResettableIterator,
{
    type Item = Vec<I::Item>;
    type IntoIter = BundledIter<I>;

    fn into_iter(self) -> Self::IntoIter {
        let done = self.iters.iter().map(|_| false).collect();
        BundledIter::<I> {
            iters: self.iters,
            offset: 0,
            done: done,
        }
    }
}

impl<I> BundledIter<I> {
    fn is_all_done(&self) -> bool {
        self.done.iter().all(|b| *b)
    }

    fn count_done(&self) -> usize {
        self.done.iter().fold(0, |c, b| if *b { c + 1 } else { c })
    }

    fn len(&self) -> usize {
        self.done.len()
    }
}

impl<I> Iterator for BundledIter<I>
where
    I: ResettableIterator,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut count = self.count_done();
        if self.is_all_done() {
            None
        } else {
            let mut v = vec![];
            for iter in &mut self.iters {
                let next = match iter.next() {
                    Some(next) => next,
                    None => {
                        if !self.done[v.len()] {
                            count += 1;
                        }
                        self.done[v.len()] = true;
                        iter.reset();
                        if let Some(next) = iter.next() {
                            next
                        } else {
                            return None;
                        }
                    }
                };
                v.push(next);
            }
            if count < self.len() {
                self.offset += 1;
                Some(v)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IterBundle, ResettableIterator};

    struct VecIter {
        v: Vec<u32>,
        offset: usize,
    }

    impl Iterator for VecIter {
        type Item = u32;

        fn next(&mut self) -> Option<Self::Item> {
            if self.offset < self.v.len() {
                let n = self.v[self.offset];
                self.offset += 1;
                Some(n)
            } else {
                None
            }
        }
    }

    impl ResettableIterator for VecIter {
        fn reset(&mut self) {
            self.offset = 0;
        }
    }

    #[test]
    fn resettable_iter_reset() {
        let mut vi = VecIter {
            v: vec![1, 2, 3],
            offset: 0,
        };
        assert_eq!(vi.next(), Some(1));
        assert_eq!(vi.next(), Some(2));
        assert_eq!(vi.next(), Some(3));
        assert_eq!(vi.next(), None);
        vi.reset();
        assert_eq!(vi.next(), Some(1));
    }

    #[test]
    fn bundled_iter_one_iter() {
        let mut bundle = IterBundle::new();
        bundle.push(VecIter {
            v: vec![1, 2, 3],
            offset: 0,
        });
        let mut iter = bundle.into_iter();
        assert_eq!(iter.next(), Some(vec![1]));
        assert_eq!(iter.next(), Some(vec![2]));
        assert_eq!(iter.next(), Some(vec![3]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn bundled_iter_multiple_iters() {
        let mut bundle = IterBundle::new();
        bundle.push(VecIter {
            v: vec![1, 2, 3],
            offset: 0,
        });
        bundle.push(VecIter {
            v: vec![5, 6, 7, 8, 9],
            offset: 0,
        });
        bundle.push(VecIter {
            v: vec![10],
            offset: 0,
        });
        let mut iter = bundle.into_iter();
        assert_eq!(iter.next(), Some(vec![1, 5, 10]));
        assert_eq!(iter.next(), Some(vec![2, 6, 10]));
        assert_eq!(iter.next(), Some(vec![3, 7, 10]));
        assert_eq!(iter.next(), Some(vec![1, 8, 10]));
        assert_eq!(iter.next(), Some(vec![2, 9, 10]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn bundled_iter_always_none() {
        let mut bundle = IterBundle::new();
        bundle.push(VecIter {
            v: vec![1, 2, 3],
            offset: 0,
        });
        bundle.push(VecIter {
            v: vec![],
            offset: 0,
        });
        let mut iter = bundle.into_iter();
        assert_eq!(iter.next(), None);
    }
}
