use std::{
  sync::{Arc, Condvar, Mutex},
  time::Duration,
};

use rand::Rng;

struct Semaphore {
  current_weight: Mutex<usize>,
  max_weight: usize,
  cond_var: Condvar,
}

impl Semaphore {
  pub fn new(max_weight: usize) -> Self {
    Self {
      current_weight: Mutex::new(0),
      cond_var: Condvar::new(),
      max_weight,
    }
  }

  pub fn acquire<'a>(&'a self, weight: usize) -> Acquired<'a> {
    if weight > self.max_weight {
      panic!(
        "tried to acquire more weight thant the semaphore has. weight={} max_weight={}",
        weight, self.max_weight
      );
    }

    let current_weight = self.current_weight.lock().unwrap();
    let max_weight = self.max_weight;

    let mut current_weight = self
      .cond_var
      .wait_while(current_weight, |current_weight| {
        max_weight - *current_weight < weight
      })
      .unwrap();

    *current_weight += weight;

    Acquired {
      weight,
      semaphore: self,
    }
  }

  /// Note that this is a private method that should be called only by
  /// the Acquired Drop implementation.
  fn release(&self, weight: usize) {
    if weight > self.max_weight {
      panic!(
        "tried to release more weight thant the semaphore has. weight={} max_weight={}",
        weight, self.max_weight
      );
    }

    let mut current_weight = self.current_weight.lock().unwrap();

    *current_weight -= weight;

    self.cond_var.notify_all();
  }
}

struct Acquired<'a> {
  weight: usize,
  semaphore: &'a Semaphore,
}

impl<'a> Drop for Acquired<'a> {
  fn drop(&mut self) {
    self.semaphore.release(self.weight);
  }
}

fn main() {
  let sema = Arc::new(Semaphore::new(3));

  let threads: Vec<_> = (0..=10)
    .map(|i| {
      let sema_clone = Arc::clone(&sema);

      std::thread::spawn(move || {
        println!("thread {}: TRYING to acquire sema", i);
        let _guard = sema_clone.acquire(1);
        println!("thread {}: ACQUIRED sema", i);
        std::thread::sleep(Duration::from_secs(rand::thread_rng().gen_range(1..=5)));
        println!("thread {}: RELEASING sema", i);
      })
    })
    .collect();

  for thread in threads {
    thread.join().unwrap();
  }
}
