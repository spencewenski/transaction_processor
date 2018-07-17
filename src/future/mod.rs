use futures::{Future, Async};

pub struct FutureValue<T: Clone> {
    val: Option<T>,
}

impl<T: Clone> FutureValue<T> {
    pub fn new() -> FutureValue<T> {
        FutureValue {
            val: Option::None,
        }
    }

    pub fn set(&mut self, val: T) {
        if let Option::Some(_) = self.val {
            panic!("Already set value!");
        }
        self.val = Option::Some(val);
    }
}

impl<T: Clone> Future for FutureValue<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Result<Async<<Self as Future>::Item>, <Self as Future>::Error> {
        if let Option::Some(ref v) = self.val {
            return Ok(Async::Ready(v.clone()));
        }
        Ok(Async::NotReady)
    }
}