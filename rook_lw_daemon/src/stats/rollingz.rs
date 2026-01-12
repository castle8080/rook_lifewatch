#[derive(Debug, Clone, Copy)]
pub struct RollingZ {
	alpha: f64,
	w: f64,
	s1: f64,
	s2: f64,
	n: u64,
}

impl RollingZ {
	/// Create a new rolling z-score calculator using exponential decay.
	///
	/// `alpha` is the update rate in (0, 1]. Higher values react faster.
	pub fn new(alpha: f64) -> Self {
		assert!(alpha.is_finite());
		assert!((0.0 < alpha) && (alpha <= 1.0));
		Self {
			alpha,
			w: 0.0,
			s1: 0.0,
			s2: 0.0,
			n: 0,
		}
	}

	/// Create using a half-life expressed in number of samples.
	///
	/// After `half_life_samples`, the contribution of a sample is halved.
	pub fn with_half_life(half_life_samples: f64) -> Self {
		assert!(half_life_samples.is_finite());
		assert!(half_life_samples > 0.0);
		let alpha = 1.0 - (0.5f64).powf(1.0 / half_life_samples);
		Self::new(alpha)
	}

	pub fn reset(&mut self) {
		self.w = 0.0;
		self.s1 = 0.0;
		self.s2 = 0.0;
		self.n = 0;
	}

	pub fn alpha(&self) -> f64 {
		self.alpha
	}

	pub fn count(&self) -> u64 {
		self.n
	}

	/// Returns the current mean (if any samples have been seen).
	pub fn mean(&self) -> Option<f64> {
		if self.w > 0.0 {
			Some(self.s1 / self.w)
		} else {
			None
		}
	}

	/// Returns the current variance (if any samples have been seen).
	pub fn variance(&self) -> Option<f64> {
		if self.w <= 0.0 {
			return None;
		}

		let mean = self.s1 / self.w;
		let second_moment = self.s2 / self.w;
		let var = second_moment - mean * mean;
		Some(var.max(0.0))
	}

	pub fn std_dev(&self) -> Option<f64> {
		self.variance().map(|v| v.sqrt())
	}

	/// Incorporate `x` into the rolling stats, then compute the z-score of `x` relative
	/// to the *updated* rolling distribution.
	///
	/// If there isn't enough information yet (no samples or zero variance), returns 0.0.
	pub fn update(&mut self, x: f64) -> f64 {
		self.ingest(x);

		match (self.mean(), self.std_dev()) {
			(Some(mean), Some(std)) if std.is_finite() && std > 0.0 => (x - mean) / std,
			_ => 0.0,
		}
	}

	/// Incorporate `x` into rolling stats without computing a z-score.
	pub fn ingest(&mut self, x: f64) {
		if !x.is_finite() {
			return;
		}

		let a = self.alpha;
		self.w = (1.0 - a) * self.w + a;
		self.s1 = (1.0 - a) * self.s1 + a * x;
		self.s2 = (1.0 - a) * self.s2 + a * x * x;
		self.n = self.n.saturating_add(1);
	}
}

#[cfg(test)]
mod tests {
	use super::RollingZ;

	#[test]
	fn rollingz_returns_zero_until_variance_exists() {
		let mut rz = RollingZ::new(0.2);

		// No prior samples.
		assert!(rz.update(10.0).abs() <= 1e-12);

		// Still essentially one repeated value => variance ~ 0.
		for _ in 0..50 {
			assert!(rz.update(10.0).abs() <= 1e-6);
		}
		assert!(rz.std_dev().unwrap() <= 1e-6);
	}

	#[test]
	fn rollingz_converges_to_reasonable_stats() {
		let mut rz = RollingZ::new(0.1);

		// Feed a stable value.
		for _ in 0..200 {
			rz.ingest(5.0);
		}
		let mean = rz.mean().unwrap();
		assert!((mean - 5.0).abs() < 1e-6);

		// A larger value should be positive z.
		let z = rz.update(6.0);
		assert!(z >= 0.0);
	}
}
