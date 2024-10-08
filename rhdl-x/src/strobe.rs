use rhdl::prelude::*;

use crate::constant;
use crate::dff;

#[derive(PartialEq, Clone, Copy, Debug, Digital)]
pub struct I {
    pub enable: bool,
}

#[derive(Clone, Debug, Synchronous)]
#[rhdl(kernel=strobe::<{N}>)]
pub struct U<const N: usize> {
    counter: dff::U<Bits<N>>,
    threshold: constant::U<Bits<N>>,
}

impl<const N: usize> U<N> {
    pub fn new(threshold: Bits<N>) -> Self {
        Self {
            counter: dff::U::new(Bits::ZERO),
            threshold: constant::U::new(threshold),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Digital)]
pub struct D<const N: usize> {
    counter: Bits<N>,
    threshold: (),
}

#[derive(Clone, Copy, PartialEq, Debug, Digital)]
pub struct Q<const N: usize> {
    counter: Bits<N>,
    threshold: Bits<N>,
}

impl<const N: usize> SynchronousIO for U<N> {
    type I = I;
    type O = bool;
}

impl<const N: usize> SynchronousDQ for U<N> {
    type D = D<N>;
    type Q = Q<N>;
}

impl<const N: usize> Default for D<N> {
    fn default() -> Self {
        Self {
            counter: bits(0),
            threshold: (),
        }
    }
}

#[kernel]
pub fn strobe<const N: usize>(cr: ClockReset, i: I, q: Q<N>) -> (bool, D<N>) {
    let mut d = D::<{ N }>::default();
    let count_next = if i.enable { q.counter + 1 } else { q.counter };
    let strobe = i.enable & (q.counter == q.threshold);
    let count_next = if strobe || cr.reset.any() {
        bits(0)
    } else {
        count_next
    };
    d.counter = count_next;
    (strobe, d)
}

#[cfg(test)]
mod tests {
    use rhdl::core::timing::simplest_cost;

    use super::*;

    #[test]
    fn test_strobe_timing() -> miette::Result<()> {
        let uut: U<4> = U::new(bits(12));
        let mut counter_uut = uut.descriptor()?.flow_graph.clone();
        rhdl::core::timing::compute_node_costs(&mut counter_uut, simplest_cost);
        eprintln!("counter {:?}", counter_uut);
        let mut dot = std::fs::File::create("strobe_fg.dot").unwrap();
        write_dot(&counter_uut, &mut dot).unwrap();
        Ok(())
    }
}
