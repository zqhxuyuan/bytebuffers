#[derive(Debug, Clone)]
pub struct Buffer {
    pub mark: i32,
    pub position: i32,
    pub limit: i32,
    pub cap: i32,
}

pub trait IBuffer {
    fn mark(&self) -> i32;
    fn cap(&self) -> i32;
    fn position(&self) -> i32;
    fn limit(&self) -> i32;

    fn reset(&mut self) -> &mut Self;

    fn limit_(&mut self, limit: i32) -> &mut Self;

    fn position_(&mut self, position: i32) -> &mut Self;

    fn mark_(&mut self) -> &mut Self;

    fn clear(&mut self) -> &mut Self;

    fn truncate(&mut self);

    fn flip(&mut self) -> &mut Self;

    fn rewind(&mut self) -> &mut Self;

    fn remaining(&self) -> i32 {
        self.limit() - self.position()
    }

    fn has_remaining(&self) -> bool {
        self.position() < self.limit()
    }

    fn slice(&self) -> &Self;

    fn get(&mut self) -> u8;
}

impl IBuffer for Buffer {
    fn mark(&self) -> i32 {
        self.mark
    }
    fn cap(&self) -> i32 {
        self.cap
    }
    fn position(&self) -> i32 {
        self.position
    }
    fn limit(&self) -> i32 {
        self.limit
    }

    fn reset(&mut self) -> &mut Self {
        let m = self.mark;
        if m < 0 {
            panic!("invalid mark!")
        }
        self.position = m;
        self
    }

    fn limit_(&mut self, limit: i32) -> &mut Self {
        if limit > self.cap || limit < 0 {
            panic!("illegal argument!")
        }
        self.limit = limit;
        if self.position > self.limit {
            self.position = self.limit;
        }
        if self.mark > self.limit {
            self.mark = -1;
        }
        self
    }

    fn position_(&mut self, position: i32) -> &mut Self {
        if position > self.limit || position < 0 {
            panic!("illegal argument!")
        }
        self.position = position;
        if self.mark > self.position {
            self.mark = -1;
        }
        self
    }

    fn mark_(&mut self) -> &mut Self {
        self.mark = self.position;
        self
    }

    fn clear(&mut self) -> &mut Self {
        self.position = 0;
        self.limit = self.cap;
        self.mark = -1;
        self
    }

    fn truncate(&mut self) {
        self.mark = -1;
        self.position = 0;
        self.limit = 0;
        self.cap = 0;
    }

    fn flip(&mut self) -> &mut Self {
        self.limit = self.position;
        self.position = 0;
        self.mark = -1;
        self
    }

    fn rewind(&mut self) -> &mut Self {
        self.position = 0;
        self.mark = -1;
        self
    }

    fn slice(&self) -> &Self {
        self
    }

    fn get(&mut self) -> u8 {
        unimplemented!()
    }
}

impl Buffer {
    pub fn default() -> Self {
        Self {
            mark: -1,
            position: 0,
            limit: 0,
            cap: 0,
        }
    }

    #[deprecated]
    pub fn new(&mut self, mark: i32, position: i32, limit: i32, cap: i32) {
        if cap < 0 {
            panic!("illegal argument")
        }
        self.cap = cap;
        self.limit_(limit);
        self.position_(position);

        if mark > 0 {
            if mark > position {
                panic!("illegal argument")
            }
            self.mark = mark;
        }
    }

    pub fn init(&mut self) -> &mut Self {
        Self::new_(self.mark, self.position, self.limit, self.cap);
        self.limit_(self.limit);
        self.position_(self.position);
        self
    }

    pub fn new_(mark: i32, position: i32, limit: i32, cap: i32) -> Self {
        if cap < 0 {
            panic!("illegal argument")
        }
        if mark > 0 && mark > position {
            panic!("illegal argument")
        }
        Self {
            mark,
            position,
            limit,
            cap,
        }
    }

    pub fn discard_mark(&mut self) {
        self.mark = -1;
    }

    pub fn next_get_index(&mut self) -> i32 {
        if self.position >= self.limit {
            panic!("buffer under flow!");
        }
        let pos = self.position;
        self.position += 1;
        pos
    }

    pub fn next_get_index_nb(&mut self, nb: i32) -> i32 {
        if self.limit - self.position < nb {
            panic!("buffer under flow!")
        }
        let p = self.position;
        self.position += nb;
        p
    }

    pub fn next_put_index(&mut self) -> i32 {
        if self.position >= self.limit {
            panic!("buffer over flow!");
        }
        let pos = self.position;
        self.position += 1;
        pos
    }

    pub fn next_put_index_nb(&mut self, nb: i32) -> i32 {
        if self.limit - self.position < nb {
            panic!("buffer over flow!");
        }
        let p = self.position;
        self.position += nb;
        p
    }

    pub fn check_index(&mut self, i: i32) -> i32 {
        if i < 0 || i >= self.limit {
            panic!("index out of bound")
        }
        i
    }

    pub fn check_index_nb(&mut self, i: i32, nb: i32) -> i32 {
        if i < 0 || nb >= self.limit - i {
            panic!("index out of bound")
        }
        i
    }

    pub fn check_bounds(off: i32, len: i32, size: i32) {
        if (off | len | (off + len) | (size - (off + len))) < 0 {
            panic!("index out of bounds!")
        }
    }
}