use anyhow::{Result, bail};

pub(super) struct EdgeCollection
{
	chains: Vec<Chain>,
}

type Chain = Vec<usize>;

enum Mergeable
{
	No,
	Yes(usize),
	Reversed(usize),
}

impl Mergeable
{
	pub fn is_reversed(&self) -> bool
	{
		return match self
		{
			Mergeable::Reversed(_) => true,
			_ => false,
		};
	}
}

impl EdgeCollection
{
	pub fn new() -> Self
	{
		return Self { chains: Vec::new() };
	}

	pub fn add(&mut self, edge: (usize, usize)) -> Result<()>
	{
		if edge.0 == edge.1
		{
			bail!("Edge vertices must be different");
		}

		match self.check_merge_chain(None, edge)
		{
			Mergeable::Yes(target_index) =>
			{
				self.merge_with_chain(vec![edge.0, edge.1].into(), target_index);
			}
			Mergeable::Reversed(target_index) =>
			{
				self.merge_with_chain(vec![edge.1, edge.0].into(), target_index);
			}
			Mergeable::No =>
			{
				self.chains.push(vec![edge.0, edge.1].into());

				// Could not merge with any existing chain, so no need to run
				// try_merge_chains().
				return Ok(());
			}
		};

		// We merged the new edge into one of the existing chains.
		// Now see if this allows us to merge any of our existing chains together. Keep
		// trying until there are no more to merge.
		loop
		{
			if !self.try_merge_chains()
			{
				break;
			}
		}

		return Ok(());
	}

	pub fn has_single_edge_loop(&self) -> bool
	{
		if self.chains.len() != 1
		{
			return false;
		}

		let chain: &Chain = self.chains.first().unwrap();
		assert!(chain.len() >= 2);

		return *chain.first().unwrap() == *chain.last().unwrap();
	}

	pub fn num_chains(&self) -> usize
	{
		return self.chains.len();
	}

	pub fn into_edge_loop(mut self) -> Result<Vec<usize>>
	{
		if !self.has_single_edge_loop()
		{
			bail!("Container does not contain a single edge loop");
		}

		let mut out: Chain = self.chains.pop().unwrap();

		// Remove the last element since it'll be a duplicate of the first.
		out.pop();

		return Ok(out);
	}

	pub fn validate(&self) -> Result<()>
	{
		if self.chains.len() < 1
		{
			bail!("No edges found");
		}

		if !self.has_single_edge_loop()
		{
			if self.chains.len() == 1
			{
				bail!("Edges did not form a closed loop");
			}
			else
			{
				bail!(
					"Edges did not form a single closed loop (found {} chains of edges)",
					self.chains.len()
				);
			}
		}

		return Ok(());
	}

	// Check all chains of edges in the container, and try and merge two of them
	// together to form a longer chain. Returns true if this was possible, and false
	// if not.
	fn try_merge_chains(&mut self) -> bool
	{
		if self.chains.len() < 2
		{
			return false;
		}

		for (candidate_index, candidate) in self.chains.iter().enumerate()
		{
			match self.check_merge_chain(
				Some(candidate_index),
				(*candidate.first().unwrap(), *candidate.last().unwrap()),
			)
			{
				Mergeable::No => continue,
				Mergeable::Yes(target_index) =>
				{
					self.merge_chains(candidate_index, target_index, false);
				}
				Mergeable::Reversed(target_index) =>
				{
					self.merge_chains(candidate_index, target_index, true);
				}
			};

			return true;
		}

		return false;
	}

	fn check_merge_chain(&self, chain_index: Option<usize>, chain: (usize, usize)) -> Mergeable
	{
		for (target_index, target_chain) in self.chains.iter().enumerate()
		{
			if let Some(idx) = chain_index
				&& target_index == idx
			{
				continue;
			}

			assert!(target_chain.len() >= 2);

			let mergeable: Mergeable = EdgeCollection::can_merge(
				chain,
				(
					*target_chain.first().unwrap(),
					*target_chain.last().unwrap(),
				),
				target_index,
			);

			match mergeable
			{
				Mergeable::No => continue,
				_ => return mergeable,
			}
		}

		return Mergeable::No;
	}

	fn merge_chains(&mut self, candidate_index: usize, target_index: usize, reverse: bool)
	{
		let mut candidate_chain: Chain = self.chains.remove(candidate_index);
		assert!(candidate_chain.len() >= 2);

		if reverse
		{
			candidate_chain.reverse();
		}

		let target_index = if candidate_index < target_index
		{
			target_index - 1
		}
		else
		{
			target_index
		};

		self.merge_with_chain(candidate_chain, target_index);
	}

	fn merge_with_chain(&mut self, mut to_merge: Chain, target_index: usize)
	{
		let target_chain: &mut Chain = &mut self.chains[target_index];
		assert!(target_chain.len() >= 2);

		if to_merge.first().unwrap() == target_chain.last().unwrap()
		{
			to_merge.remove(0);
			target_chain.extend(to_merge.into_iter());
			return;
		}

		if to_merge.last().unwrap() == target_chain.first().unwrap()
		{
			to_merge.pop();
			target_chain.splice(0..0, to_merge.into_iter());
			return;
		}

		unreachable!("Expected candidate and target edge chains to link up");
	}

	fn can_merge(
		candidate: (usize, usize),
		target: (usize, usize),
		target_index: usize,
	) -> Mergeable
	{
		if candidate.0 == target.1 || candidate.1 == target.0
		{
			return Mergeable::Yes(target_index);
		}

		if candidate.0 == target.0 || candidate.1 == target.1
		{
			return Mergeable::Reversed(target_index);
		}

		return Mergeable::No;
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn no_edges()
	{
		let collection: EdgeCollection = EdgeCollection::new();

		assert_eq!(collection.num_chains(), 0);
		assert!(!collection.has_single_edge_loop());
	}

	#[test]
	fn one_edge()
	{
		{
			let mut collection: EdgeCollection = EdgeCollection::new();
			collection.add((0, 1));

			assert_eq!(collection.num_chains(), 1);
			assert!(!collection.has_single_edge_loop());
		}

		{
			let mut collection: EdgeCollection = EdgeCollection::new();
			assert!(collection.add((0, 0)).is_err());
		}
	}
}
