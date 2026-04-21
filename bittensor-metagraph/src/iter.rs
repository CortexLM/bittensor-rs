//! Neuron iterator — yields reconstructed [`NeuronInfo`] from columnar metagraph storage.

use bittensor_core::types::NeuronInfo;

use crate::metagraph::Metagraph;

/// Iterator over neurons in a [`Metagraph`], reconstructing each [`NeuronInfo`]
/// from the columnar field vectors.
pub struct NeuronIterator<'a> {
    pub metagraph: &'a Metagraph,
    pub index: usize,
}

impl<'a> Iterator for NeuronIterator<'a> {
    type Item = NeuronInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.metagraph.n {
            return None;
        }
        let neuron = self.metagraph.neuron_at(self.index);
        self.index += 1;
        Some(neuron)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.metagraph.n.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for NeuronIterator<'a> {}

impl<'a> IntoIterator for &'a Metagraph {
    type Item = NeuronInfo;
    type IntoIter = NeuronIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.neurons()
    }
}
