use std::{cmp::Ordering, rc::Rc, time::Instant};

use rand::Rng;

enum Level {
    Upper(usize),
    Lower(usize, usize),
}

struct Index {
    level: Level,
    data: Rc<Vec<(u64, Vec<u64>)>>,
}

impl Index {
    fn new(data: Rc<Vec<(u64, Vec<u64>)>>) -> Self {
        Self {
            level: Level::Upper(0),
            data,
        }
    }

    fn seek(&mut self, v: u64) {
        match &mut self.level {
            Level::Upper(i) => {
                let (Ok(idx) | Err(idx)) = self.data.binary_search_by_key(&v, |(x, _)| *x);
                *i = idx;
            }
            Level::Lower(i, j) => {
                let (Ok(idx) | Err(idx)) = self.data[*i].1.binary_search(&v);
                *j = idx;
            }
        }
    }

    fn up(&mut self) {
        match self.level {
            Level::Lower(i, _) => self.level = Level::Upper(i),
            _ => panic!(),
        }
    }

    fn down(&mut self) {
        match self.level {
            Level::Upper(i) => self.level = Level::Lower(i, 0),
            _ => panic!(),
        }
    }

    fn reset(&mut self) {
        match &mut self.level {
            Level::Upper(i) => *i = 0,
            Level::Lower(_, j) => *j = 0,
        }
    }

    fn value(&self) -> Option<u64> {
        match self.level {
            Level::Upper(i) => Some(self.data.get(i)?.0),
            Level::Lower(i, j) => self.data.get(i)?.1.get(j).cloned(),
        }
    }

    fn next(&mut self) {
        match &mut self.level {
            Level::Upper(i) => {
                *i += 1;
            }
            Level::Lower(_, j) => {
                *j += 1;
            }
        }
    }
}

fn main() {
    let mut data = Vec::new();
    let mut rng = rand::thread_rng();
    for i in 1_u64..1000 {
        data.push((i, (i + 1..1000).filter(|_| rng.gen_bool(0.5)).collect()));
    }
    let data = Rc::new(data);

    let start = Instant::now();

    // Q(a, b, c) <- R(a, b), S(b, c), T(a, c);

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
    println!("count = {} ({:?})", count, start.elapsed());
}
