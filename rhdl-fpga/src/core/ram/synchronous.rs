use rhdl::{
    core::hdl::ast::{index, index_bit, memory_index, Declaration},
    prelude::*,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// The synchronous version of the block ram.  This one assumes a clock
/// for both the read and write interfaces, and since the clock and reset
/// lines are implied with Synchronous circuits, they do not appear in the
/// interface.
///
#[derive(Debug, Clone)]
pub struct U<T: Digital, const N: usize> {
    initial: BTreeMap<Bits<N>, T>,
}

impl<T: Digital, const N: usize> Default for U<T, N> {
    fn default() -> Self {
        Self {
            initial: BTreeMap::new(),
        }
    }
}

impl<T: Digital, const N: usize> U<T, N> {
    pub fn new(initial: impl IntoIterator<Item = (Bits<N>, T)>) -> Self {
        let len = (1 << N) as usize;
        Self {
            initial: initial.into_iter().take(len).collect(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct Write<T: Digital, const N: usize> {
    pub addr: Bits<N>,
    pub value: T,
    pub enable: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Digital)]
pub struct I<T: Digital, const N: usize> {
    pub read_addr: Bits<N>,
    pub write: Write<T, N>,
}

impl<T: Digital, const N: usize> SynchronousDQ for U<T, N> {
    type D = ();
    type Q = ();
}

impl<T: Digital, const N: usize> SynchronousIO for U<T, N> {
    type I = I<T, N>;
    type O = T;
    type Kernel = NoKernel3<ClockReset, Self::I, (), (Self::O, ())>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct S<T: Digital, const N: usize> {
    clock: Clock,
    contents: BTreeMap<Bits<N>, T>,
    output_current: T,
    output_next: T,
    write_prev: Write<T, N>,
}

impl<T: Digital, const N: usize> Synchronous for U<T, N> {
    type S = Rc<RefCell<S<T, N>>>;

    fn init(&self) -> Self::S {
        Rc::new(RefCell::new(S {
            clock: Clock::default(),
            contents: self.initial.clone(),
            output_current: T::dont_care(),
            output_next: T::dont_care(),
            write_prev: Write::dont_care(),
        }))
    }

    fn description(&self) -> String {
        format!(
            "Synchronous RAM with {} entries of type {}",
            1 << N,
            std::any::type_name::<T>()
        )
    }

    fn sim(&self, clock_reset: ClockReset, input: Self::I, state: &mut Self::S) -> Self::O {
        trace_push_path("synchronous_ram");
        trace("input", &input);
        let state = &mut state.borrow_mut();
        let clock = clock_reset.clock;
        if !clock.raw() {
            state.output_next = state
                .contents
                .get(&input.read_addr)
                .copied()
                .unwrap_or(T::dont_care());
            state.write_prev = input.write;
        }
        if clock.raw() && !state.clock.raw() {
            if state.write_prev.enable {
                let addr = state.write_prev.addr;
                let data = state.write_prev.value;
                state.contents.insert(addr, data);
            }
            state.output_current = state.output_next;
        }
        state.clock = clock;
        trace("output", &state.output_current);
        trace_pop_path();
        state.output_current
    }

    fn descriptor(&self, name: &str) -> Result<CircuitDescriptor, RHDLError> {
        let mut flow_graph = FlowGraph::default();
        let hdl = self.hdl(&format!("{name}_inner"))?;
        let (clock_reset, input, output) = flow_graph.synchronous_black_box::<Self>(hdl);
        flow_graph.inputs = vec![clock_reset, input];
        flow_graph.output = output;
        Ok(CircuitDescriptor {
            unique_name: name.to_string(),
            flow_graph,
            input_kind: <Self::I as Digital>::static_kind(),
            output_kind: <Self::O as Digital>::static_kind(),
            d_kind: Kind::Empty,
            q_kind: Kind::Empty,
            children: Default::default(),
            rtl: None,
        })
    }

    fn hdl(&self, name: &str) -> Result<HDLDescriptor, RHDLError> {
        let module_name = name.to_owned();
        let mut module = Module {
            name: module_name.clone(),
            ..Default::default()
        };
        let output_bits = unsigned_width(T::BITS);
        let input_bits = unsigned_width(<Self::I as Digital>::BITS);
        module.ports = vec![
            port(
                "clock_reset",
                Direction::Input,
                HDLKind::Wire,
                unsigned_width(2),
            ),
            port("i", Direction::Input, HDLKind::Wire, input_bits),
            port("o", Direction::Output, HDLKind::Reg, output_bits),
        ];
        let wire_decl = |name: &str, width| Declaration {
            kind: HDLKind::Wire,
            name: name.into(),
            width: unsigned_width(width),
            alias: None,
        };
        module.declarations.extend([
            wire_decl("read_addr", N),
            wire_decl("write_addr", N),
            wire_decl("write_value", T::BITS),
            wire_decl("write_enable", 1),
            wire_decl("clock", 1),
            Declaration {
                kind: HDLKind::Reg,
                name: format!("mem[{}:0]", (1 << N) - 1),
                width: output_bits,
                alias: None,
            },
        ]);
        module.statements.push(initial(
            self.initial
                .iter()
                .map(|(addr, val)| {
                    let val: BitString = val.typed_bits().into();
                    assign(&format!("mem[{}]", addr.0), bit_string(&val))
                })
                .collect(),
        ));
        let i_kind = <Self::I as Digital>::static_kind();
        let reassign = |name: &str, path: Path| {
            continuous_assignment(name, index("i", bit_range(i_kind, &path).unwrap().0))
        };
        module.statements.extend([
            reassign("read_addr", Path::default().field("read_addr")),
            reassign("write_addr", Path::default().field("write").field("addr")),
            reassign("write_value", Path::default().field("write").field("value")),
            reassign(
                "write_enable",
                Path::default().field("write").field("enable"),
            ),
            continuous_assignment("clock", index_bit("clock_reset", 0)),
        ]);
        module.statements.push(always(
            vec![Events::Posedge("clock".into())],
            vec![non_blocking_assignment(
                "o",
                memory_index("mem", id("read_addr")),
            )],
        ));
        module.statements.push(always(
            vec![Events::Posedge("clock".into())],
            vec![if_statement(
                id("write_enable"),
                vec![non_blocking_assignment(
                    "mem[write_addr]",
                    id("write_value"),
                )],
                vec![],
            )],
        ));
        Ok(HDLDescriptor {
            name: module_name,
            body: module,
            children: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use rhdl::prelude::*;

    use super::*;
    use std::{iter::repeat, path::PathBuf};

    #[derive(Debug, Clone, PartialEq, Copy)]
    enum Cmd {
        Write(b4, b8),
        Read(b4),
    }

    fn rand_cmd() -> Cmd {
        match rand::random::<u8>() % 2 {
            0 => Cmd::Write(
                bits(rand::random::<u128>() % 16),
                bits(rand::random::<u128>() % 256),
            ),
            1 => Cmd::Read(bits(rand::random::<u128>() % 16)),
            _ => unreachable!(),
        }
    }

    struct TestItem(Cmd, b8);

    impl From<Cmd> for I<b8, 4> {
        fn from(cmd: Cmd) -> Self {
            match cmd {
                Cmd::Write(addr, value) => I {
                    read_addr: bits(0),
                    write: Write {
                        addr,
                        value,
                        enable: true,
                    },
                },
                Cmd::Read(addr) => I {
                    read_addr: addr,
                    write: Write::dont_care(),
                },
            }
        }
    }

    #[test]
    fn test_scan_out_ram() -> miette::Result<()> {
        type UC = U<b8, 4>;
        let uut: UC = U::new(
            (0..)
                .enumerate()
                .map(|(ndx, _)| (bits(ndx as u128), bits((15 - ndx) as u128))),
        );
        let test = (0..16)
            .cycle()
            .map(|ndx| TestItem(Cmd::Read(bits(ndx)), bits(15 - ndx)))
            .take(17);
        let inputs = test.clone().map(|item| item.0.into());
        let expected = test.map(|item| item.1).take(16);
        let stream = inputs.stream_after_reset(1).clock_pos_edge(100);
        let sim = uut.run(stream)?;
        let vcd = sim.clone().collect::<Vcd>();
        vcd.dump_to_file(&PathBuf::from("test_scan_out_ram.vcd"))
            .unwrap();
        let values = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .sample_at_pos_edge(|x| x.value.0.clock)
            .skip(2)
            .map(|x| x.value.2);
        assert!(values.eq(expected));
        Ok(())
    }

    fn random_command_stream(
        len: usize,
    ) -> impl Iterator<Item = TimedSample<(ClockReset, I<b8, 4>)>> {
        let inputs = (0..).map(|_| rand_cmd().into()).take(len);
        inputs.stream_after_reset(1).clock_pos_edge(100)
    }

    #[test]
    fn test_hdl_output() -> miette::Result<()> {
        type UC = U<b8, 4>;
        let uut: UC = U::new((0..).map(|ndx| (bits(ndx), bits(0))));
        let stream = random_command_stream(1000);
        let test_bench = uut.run(stream)?.collect::<SynchronousTestBench<_, _>>();
        let test_mod = test_bench.flow_graph(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        let test_mod = test_bench.rtl(&uut, &TestBenchOptions::default().skip(2))?;
        test_mod.run_iverilog()?;
        Ok(())
    }

    #[test]
    fn test_ram_write_then_read() -> miette::Result<()> {
        type UC = U<b8, 4>;
        let uut: UC = U::new(repeat((Bits(0), b8::from(0))).take(16));
        let test = vec![
            Cmd::Write(bits(0), bits(72)),
            Cmd::Write(bits(1), bits(99)),
            Cmd::Write(bits(2), bits(255)),
            Cmd::Read(bits(0)),
            Cmd::Read(bits(1)),
            Cmd::Read(bits(2)),
            Cmd::Read(bits(3)),
        ];
        let inputs = test
            .into_iter()
            .map(|x| x.into())
            .stream_after_reset(1)
            .clock_pos_edge(100);
        let sim = uut.run(inputs)?;
        let outputs = sim
            .glitch_check(|x| (x.value.0.clock, x.value.2))
            .sample_at_pos_edge(|x| x.value.0.clock)
            .skip(5)
            .take(3)
            .map(|x| x.value.2)
            .collect::<Vec<_>>();
        assert_eq!(outputs, vec![b8::from(72), b8::from(99), b8::from(255)]);
        Ok(())
    }
}
