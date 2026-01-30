use crate::math::geometry::vector_to_unit_or_null;
use anyhow::{Result, bail};
use glam::DVec3;

pub(super) struct EdgeCollection
{
	edges_vec: Vec<(usize, usize)>,
}

impl EdgeCollection
{
	pub fn new() -> Self
	{
		return Self {
			edges_vec: Vec::new(),
		};
	}

	pub fn add(&mut self, edge: (usize, usize)) -> Result<()>
	{
		// Each edge on a face connects two vertices, and each vertex should connect
		// only two edges. We try and insert edges into the list so that iterating up
		// the list iterates over connected chains of edges.

		if self.contains(&edge)
		{
			bail!("Duplicate edge");
		}

		for index in 0..self.edges_vec.len()
		{
			let existing: &(usize, usize) = &self.edges_vec[index];

			if existing.1 == edge.0
			{
				// Existing edge connects to the beginning of our new edge, so insert the new
				// edge in front.
				self.edges_vec.insert(index + 1, edge);
				return Ok(());
			}

			if existing.0 == edge.1
			{
				// Existing edge connects to the end of our new edge, so insert the edge behind.
				self.edges_vec.insert(index, edge);
				return Ok(());
			}

			if existing.0 == edge.0
			{
				// Existing edge connects to the end of our new edge, and the
				// new edge should be the other way around. Insert the edge behind.
				self.edges_vec.insert(index, (edge.1, edge.0));
				return Ok(());
			}

			if existing.1 == edge.1
			{
				// Existing edge connects to the beginning of our new edge, and
				// the new edge should be the other way around. Insert the new
				// edge in front.
				self.edges_vec.insert(index + 1, (edge.1, edge.0));
				return Ok(());
			}
		}

		// No matches, so just add on the end.
		self.edges_vec.push(edge);
		return Ok(());
	}

	pub fn contains(&self, edge: &(usize, usize)) -> bool
	{
		return self
			.edges_vec
			.iter()
			.find(|item| *item == edge || *item == &(edge.1, edge.0))
			.is_some();
	}

	// Returns a vector of slices, where each slice represents a chain of connected
	// edges. If the vector is empty, there are no edges.
	pub fn chains(&self) -> Vec<&[(usize, usize)]>
	{
		let mut out: Vec<&[(usize, usize)]> = Vec::new();
		let mut slice_indices: Option<(usize, usize)> = None;

		for index in 0..self.edges_vec.len()
		{
			if index == 0 || slice_indices.is_none()
			{
				slice_indices = Some((index, index));
				continue;
			}

			let edge: &(usize, usize) = &self.edges_vec[index];
			let last_edge: &(usize, usize) = &self.edges_vec[index - 1];

			if last_edge.1 == edge.0
			{
				// Lengthen the chain.
				slice_indices = Some((slice_indices.unwrap().0, index))
			}
			else
			{
				// Commit the chain and start a new one.
				let indices = slice_indices.unwrap();
				out.push(&self.edges_vec[indices.0..=indices.1]);

				slice_indices = Some((index, index));
			}
		}

		// Commit any remaining chain.
		if slice_indices.is_some()
		{
			let indices = slice_indices.unwrap();
			out.push(&self.edges_vec[indices.0..=indices.1]);
		}

		return out;
	}

	pub fn verify(&self) -> Result<()>
	{
		let chains: Vec<&[(usize, usize)]> = self.chains();

		if chains.len() == 0
		{
			bail!("No edges found");
		}

		if chains.len() > 1
		{
			bail!("Could not compute contiguous edge sequence");
		}

		let chain: &[(usize, usize)] = chains[0];
		let first_edge = chain.first().unwrap();
		let last_edge = chain.last().unwrap();

		if first_edge.0 != last_edge.1
		{
			bail!(
				"Edges did not form closed loop (first vertex {} was different to last vertex {})",
				first_edge.0,
				last_edge.1
			);
		}

		return Ok(());
	}

	// Only applies if the edge collection is valid (ie. verify() returns success).
	// Otherwise, results are undefined. Vertices list must be large enough to be
	// indexed into by edges, otherwise the function will panic.
	pub fn normal(&self, vertices: &Vec<DVec3>, zero_epsilon: f64) -> Option<DVec3>
	{
		if self.edges_vec.len() < 2
		{
			return None;
		}

		let v0_index: usize = self.edges_vec[0].0;
		let v1_index: usize = self.edges_vec[0].1;
		let v2_index: usize = self.edges_vec[1].1;

		assert!(v0_index < vertices.len());
		assert!(v1_index < vertices.len());
		assert!(v2_index < vertices.len());

		let v0: DVec3 = vertices[v0_index];
		let v1: DVec3 = vertices[v1_index];
		let v2: DVec3 = vertices[v2_index];

		let normal = vector_to_unit_or_null((v1 - v0).cross(v2 - v0), zero_epsilon);
		return if normal.1 { Some(normal.0) } else { None };
	}

	pub fn reverse(&mut self)
	{
		self.edges_vec.reverse();

		self.edges_vec = self
			.edges_vec
			.iter_mut()
			.map(|edge| (edge.1, edge.0))
			.collect();
	}
}
