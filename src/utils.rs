pub struct PairIterator<'a> {
    iter: std::slice::Iter<'a, f64>,
}

impl<'a> PairIterator<'a> {
    pub fn new(vec: &'a [f64]) -> Self {
        Self {
            iter: vec.iter()
        }
    }
}

impl<'a> Iterator for PairIterator<'a> {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let a = match self.iter.next() {
            Some(a) => a,
            None => {return None;},
        };
        let b = match self.iter.next() {
            Some(b) => b,
            None => {return None;},
        };
        Some((*a, *b))
    }
}
