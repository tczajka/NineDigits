use crate::small::Small;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Permutation<const L: usize> {
    forward: [Small<L>; L],
    backward: [Small<L>; L],
}

impl<const L: usize> Permutation<L> {
    pub fn identity() -> Self {
        let mut forward = [Small::new(0); L];
        let mut backward = [Small::new(0); L];
        for i in Small::all() {
            forward[i] = i;
            backward[i] = i;
        }
        Self { forward, backward }
    }

    pub fn swap_forward(&mut self, i: Small<L>, j: Small<L>) {
        self.forward.swap(i.into(), j.into());
        self.backward
            .swap(self.forward[i].into(), self.forward[j].into());
    }

    pub fn inverse(&self) -> Self {
        Self {
            forward: self.backward,
            backward: self.forward,
        }
    }

    pub fn forward(&self, i: Small<L>) -> Small<L> {
        self.forward[i]
    }

    pub fn backward(&self, i: Small<L>) -> Small<L> {
        self.backward[i]
    }

    pub fn then(&self, other: &Self) -> Self {
        let mut forward = [Small::new(0); L];
        let mut backward = [Small::new(0); L];
        for i in Small::all() {
            forward[i] = other.forward[self.forward[i]];
            backward[i] = self.backward[other.backward[i]];
        }
        Self { forward, backward }
    }

    pub fn then_array<T>(&self, array: &[T; L]) -> [T; L]
    where
        T: Copy,
    {
        let mut result = [array[0]; L];
        for i in Small::all() {
            result[i] = array[self.forward[i]];
        }
        result
    }
}

pub static ALL_PERMUTATIONS_2: [Permutation<2>; 2] = [
    Permutation {
        forward: [Small::new(0), Small::new(1)],
        backward: [Small::new(0), Small::new(1)],
    },
    Permutation {
        forward: [Small::new(1), Small::new(0)],
        backward: [Small::new(1), Small::new(0)],
    },
];

pub static ALL_PERMUTATIONS_3: [Permutation<3>; 6] = [
    Permutation {
        forward: [Small::new(0), Small::new(1), Small::new(2)],
        backward: [Small::new(0), Small::new(1), Small::new(2)],
    },
    Permutation {
        forward: [Small::new(0), Small::new(2), Small::new(1)],
        backward: [Small::new(0), Small::new(2), Small::new(1)],
    },
    Permutation {
        forward: [Small::new(1), Small::new(0), Small::new(2)],
        backward: [Small::new(1), Small::new(0), Small::new(2)],
    },
    Permutation {
        forward: [Small::new(1), Small::new(2), Small::new(0)],
        backward: [Small::new(2), Small::new(0), Small::new(1)],
    },
    Permutation {
        forward: [Small::new(2), Small::new(0), Small::new(1)],
        backward: [Small::new(1), Small::new(2), Small::new(0)],
    },
    Permutation {
        forward: [Small::new(2), Small::new(1), Small::new(0)],
        backward: [Small::new(2), Small::new(1), Small::new(0)],
    },
];
