use core::marker::PhantomData;

pub struct RecursiveFn<F, T>
where
	F: Fn(T) -> T,
{
	count: usize,
	f: F,
	phantom: PhantomData<T>
}

impl<F, T> RecursiveFn<F, T>
where
	F: Fn(T) -> T,
{
	/// Creates a new recursing function with `count = 0`.
	/// When the method `call` is called, the value passed will be passed through the function `f` for `count` amount of times
	pub fn new(f: F) -> Self {
		Self { count: 0, f, phantom: PhantomData }
	}
	
	// pub fn immediate_call(f: F, amt: usize, v: T) -> T {
	// 	let s = Self {count: amt, f, phantom: PhantomData};
	// 	s.call(v)
	// }

	/// Calls the function `f` for `count` amount of times, passing the result of one as the input for the next
	pub fn call(&self, v: T) -> T {
		let mut r = v;
		for _ in 0..self.count {
			r = (self.f)(r) // Doing it with parens so that it doesn't search for a method, and calls the function
		}
		r
	}

	/// Add 1 to the `count`
	pub fn add(&mut self) {
		self.count += 1;
	}

	// /// Subtracts 1 to the `count`
	// pub fn sub(&mut self) {
	// 	self.count -= 1;
	// }
}
