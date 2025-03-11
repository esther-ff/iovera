pub(crate) struct Extra<T: IntoIterator> {
    iter: T,
    steps_req: usize,
    steps_done: usize,
    func: fn() -> T::Item,
}

impl<T: Iterator> Extra<T> {
    pub(crate) fn new(iter: T, steps_req: usize, func: fn() -> T::Item) -> Self {
        Self {
            iter,
            steps_req,
            steps_done: 0,
            func,
        }
    }
}

impl<T: Iterator> Iterator for Extra<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        if self.steps_done == self.steps_req {
            self.steps_done = 0;
            return Some((self.func)());
        }

        self.steps_done += 1;

        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
