pub(crate) struct Tuplenation<T: Iterator> {
    iter: T,
    done: bool,
}

impl<T: Iterator> Tuplenation<T> {
    pub(crate) fn new(iter: T) -> Self {
        Self { iter, done: false }
    }
}

impl<T: Iterator> Iterator for Tuplenation<T> {
    type Item = (T::Item, T::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let first = match self.iter.next() {
            None => {
                self.done = true;
                return None;
            }

            Some(item) => item,
        };

        let second = match self.iter.next() {
            None => {
                self.done = true;
                return None;
            }

            Some(item) => item,
        };

        Some((first, second))
    }
}
