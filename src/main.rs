use std::{cmp::Ordering, rc::Rc, time::Instant};

use rand::Rng;

// Points at either a first-level entry we're located at (Upper), or a
// second-level entry we're located at along with the parent in the first level
// (Lower).
enum Position {
    Upper(usize),
    Lower(usize, usize),
}

// A trie iterator as described in the Leapfrog Triejoin paper, which can
// iterate across the first variable and then drop down to the values where that
// first variable is bound.
struct Index {
    level: Position,
    // Data is stored in a two-level index:
    //
    //    1     2     3      4    5 6 7
    //   /|\   / \   /|\    /|\   | | |
    //  2 3 4 4   5 4 6 7  5 7 8  8 7 8
    //
    data: Rc<Vec<(u64, Vec<u64>)>>,
}

impl Index {
    fn new(data: Rc<Vec<(u64, Vec<u64>)>>) -> Self {
        Self {
            level: Position::Upper(0),
            data,
        }
    }

    // In whatever level we are currently in, move the iterator to the given
    // value, or to the next value that comes after.
    fn seek(&mut self, v: u64) {
        match &mut self.level {
            Position::Upper(i) => {
                let (Ok(idx) | Err(idx)) = self.data.binary_search_by_key(&v, |(x, _)| *x);
                *i = idx;
            }
            Position::Lower(i, j) => {
                let (Ok(idx) | Err(idx)) = self.data[*i].1.binary_search(&v);
                *j = idx;
            }
        }
    }

    // Move from the lower position back up to the upper position. This
    // "unbinds" the first variable.
    fn up(&mut self) {
        match self.level {
            Position::Lower(i, _) => self.level = Position::Upper(i),
            _ => panic!(),
        }
    }

    // Move from the upper position down to the lower position. This "binds" the
    // current self.value().
    fn down(&mut self) {
        match self.level {
            Position::Upper(i) => self.level = Position::Lower(i, 0),
            _ => panic!(),
        }
    }

    // Reset the iterator to the start, at its current level. A bound variable
    // (i.e., if we are in the Lower position) remains bound.
    fn reset(&mut self) {
        match &mut self.level {
            Position::Upper(i) => *i = 0,
            Position::Lower(_, j) => *j = 0,
        }
    }

    // The current value we are pointing at, at whatever level we're at.
    fn value(&self) -> Option<u64> {
        match self.level {
            Position::Upper(i) => Some(self.data.get(i)?.0),
            Position::Lower(i, j) => self.data.get(i)?.1.get(j).cloned(),
        }
    }

    // Advance to the next value.
    fn next(&mut self) {
        match &mut self.level {
            Position::Upper(i) => {
                *i += 1;
            }
            Position::Lower(_, j) => {
                *j += 1;
            }
        }
    }
}

fn main() {
    // let data = Rc::new(vec![
    //     (1, vec![2, 3, 4]),
    //     (2, vec![4, 5]),
    //     (3, vec![4, 6, 7]),
    //     (4, vec![5, 7, 8]),
    //     (5, vec![8]),
    //     (6, vec![7]),
    //     (7, vec![8]),
    // ]);

    // Generate a random graph.
    let mut data = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 1_u64..1000 {
        data.push((i, (i + 1..1000).filter(|_| rng.gen_bool(0.5)).collect()));
    }
    let data = Rc::new(data);

    let start = Instant::now();

    // Q(a, b, c) <- R(a, b), S(b, c), T(a, c);

    // Since we're finding triangles in a graph, use the same data for all
    // three.
    let mut r = Index::new(data.clone());
    let mut s = Index::new(data.clone());
    let mut t = Index::new(data);

    let mut count = 0_u64;

    while let (Some(r_a), Some(t_a)) = (r.value(), t.value()) {
        match r_a.cmp(&t_a) {
            Ordering::Less => r.seek(t_a),
            Ordering::Greater => t.seek(r_a),
            Ordering::Equal => {
                // a is now bound.
                r.down();
                t.down();
                while let (Some(r_b), Some(s_b)) = (r.value(), s.value()) {
                    match r_b.cmp(&s_b) {
                        Ordering::Less => r.seek(s_b),
                        Ordering::Greater => s.seek(r_b),
                        Ordering::Equal => {
                            // b is now bound.
                            s.down();
                            t.reset();
                            while let (Some(s_c), Some(t_c)) = (s.value(), t.value()) {
                                match s_c.cmp(&t_c) {
                                    Ordering::Less => {
                                        s.seek(t_c);
                                    }
                                    Ordering::Greater => {
                                        t.seek(s_c);
                                    }
                                    Ordering::Equal => {
                                        // We found a triangle!
                                        count += 1;
                                        s.next();
                                        t.next();
                                    }
                                }
                            }
                            // Move on to the next value of b.
                            s.up();
                            s.next();
                        }
                    }
                }
                // Move on to the next value of a.
                s.reset();
                r.up();
                r.next();
                t.up();
                t.next();
            }
        }
    }
    println!("found {} triangles in {:?}", count, start.elapsed());
}
