#![allow(clippy::upper_case_acronyms)]
use expect_test::expect;
use rhdl::prelude::*;
#[cfg(test)]
mod common;
#[cfg(test)]
use common::*;
use rhdl_core::sim::testbench::kernel::{
    test_kernel_vm_and_verilog, test_kernel_vm_and_verilog_synchronous,
};

#[test]
fn test_option_is_digital() {
    #[derive(PartialEq, Debug, Digital)]
    struct Test {
        a: Option<b8>,
        b: Option<b8>,
    }

    let foo_test = Test {
        a: Some(b8::from(0b10101011)),
        b: None,
    };

    println!("foo val: {:?}", foo_test);
    assert_eq!(foo_test.a.bin(), b9::from(0b110101011).bin());
    assert_eq!(foo_test.b.bin(), bitx_parse("000000000").unwrap());
}

#[test]
fn test_result_is_digital() -> miette::Result<()> {
    #[derive(PartialEq, Debug, Digital, Default)]
    enum Eflag {
        BadNumber,
        OutOfRange,
        #[default]
        Unknown,
    }

    type FWResult<T> = Result<T, Eflag>;
    #[kernel]
    fn foo(i: b8) -> FWResult<b8> {
        if i.any() {
            Ok(b8(0b01010101))
        } else {
            Err(Eflag::BadNumber)
        }
    }

    #[kernel]
    fn bar(i: b8) -> FWResult<b8> {
        let j = foo(i)?;
        match foo(j) {
            Ok(_k) => Err(Eflag::OutOfRange),
            Err(_e) => Ok(j),
        }
    }
    test_kernel_vm_and_verilog_synchronous::<bar, _, _, _>(
        bar,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}

#[test]
fn test_option_works() -> miette::Result<()> {
    #[kernel]
    fn opt(i: b8) -> Option<b8> {
        if i.any() {
            Some(i)
        } else {
            None
        }
    }

    test_kernel_vm_and_verilog_synchronous::<opt, _, _, _>(
        opt,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}

#[test]
fn test_option_is_kernel_ok() -> miette::Result<()> {
    #[kernel]
    fn validify(i: b8) -> Option<b8> {
        if i.any() {
            Some(i)
        } else {
            None
        }
    }

    #[kernel]
    fn opt(i: b8) -> Option<b8> {
        let j = validify(i)?;
        match validify(j) {
            Some(_k) => None,
            None => Some(j),
        }
    }
    test_kernel_vm_and_verilog_synchronous::<opt, _, _, _>(
        opt,
        exhaustive().iter().map(|x| (*x,)),
    )?;
    Ok(())
}

#[test]
fn test_option_result_no_ice() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn err_map(e: AXI4Error) -> ResponseCode {
        match e {
            AXI4Error::SLVERR => ResponseCode::SLVERR,
            AXI4Error::DECERR => ResponseCode::DECERR,
        }
    }

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = if let Some(resp) = i.val() {
            match resp {
                Result::Ok(()) => Some(ResponseCode::OKAY),
                Result::Err(e) => Some(err_map(e)),
            }
        } else {
            None
        };
        signal(d)
    }

    let inputs = [
        (signal(Some(Ok(()))),),
        (signal(Some(Err(AXI4Error::SLVERR))),),
        (signal(Some(Err(AXI4Error::DECERR))),),
        (signal(None),),
    ];

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_option_result_match_func() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn err_map(e: AXI4Error) -> ResponseCode {
        match e {
            AXI4Error::SLVERR => ResponseCode::SLVERR,
            AXI4Error::DECERR => ResponseCode::DECERR,
        }
    }

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = match i.val() {
            Some(Result::Ok(())) => Some(ResponseCode::OKAY),
            Some(Result::Err(e)) => Some(err_map(e)),
            None => None,
        };
        signal(d)
    }

    let expect = expect![[r#"Err(RHDLTypeError(RHDLTypeError { cause: PathMismatchInTypeInference, src: SourcePool { source: {FnID(8dc985147aba840b): SpannedSource { source: "fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {\n    let d = match i.val() {\n        Some(Result::Ok(())) => Some(ResponseCode::OKAY),\n        Some(Result::Err(e)) => Some(err_map(e)),\n        None => None,\n    };\n    signal(d)\n}\n", name: "do_stuff", span_map: {N1: 12..42, N5: 135..137, N8: 148..166, N21: 95..246, N20: 227..240, N4: 101..108, N13: 177..197, N23: 87..247, N12: 182..196, N22: 87..247, N11: 194..195, N18: 227..231, N26: 252..261, N17: 177..218, N14: 214..215, N6: 124..138, N16: 201..217, N15: 206..216, N3: 101..102, N27: 81..263, N9: 143..167, N2: 91..92, N28: 0..263, N25: 252..261, N19: 235..239, N0: 12..13, N7: 119..139, N24: 259..260, N10: 119..168}, fallback: N28, filename: "rhdl/tests/option.rs:187", function_id: FnID(8dc985147aba840b) }}, ranges: {FnID(8dc985147aba840b): 0..264} }, err_span: SourceSpan { offset: SourceOffset(135), length: 2 } }))"#]];
    let res = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    expect.assert_eq(&format!("{:?}", res));
    Ok(())
}

#[test]
fn test_option_result_if_let() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let d = if let Some(resp) = i.val() {
            match resp {
                Ok(()) => Some(ResponseCode::OKAY),
                Err(e) => Some(match e {
                    AXI4Error::SLVERR => ResponseCode::SLVERR,
                    AXI4Error::DECERR => ResponseCode::DECERR,
                }),
            }
        } else {
            None
        };
        signal(d)
    }

    let inputs = [
        (signal(Some(Ok(()))),),
        (signal(Some(Err(AXI4Error::SLVERR))),),
        (signal(Some(Err(AXI4Error::DECERR))),),
        (signal(None),),
    ];

    test_kernel_vm_and_verilog::<do_stuff, _, _, _>(do_stuff, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_nested_matches() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    #[derive(PartialEq, Default, Digital)]
    pub struct ReadResponse<N: BitWidth> {
        data: Bits<N>,
        resp: ResponseCode,
    }

    #[kernel]
    fn do_stuff<DATA: BitWidth>(
        a: Signal<Result<Bits<DATA>, AXI4Error>, Red>,
    ) -> Signal<Option<ReadResponse<DATA>>, Red> {
        let b = match a.val() {
            Ok(data) => ReadResponse::<DATA> {
                data,
                resp: ResponseCode::OKAY,
            },
            Err(e) => ReadResponse::<DATA> {
                data: bits(0),
                resp: match e {
                    AXI4Error::SLVERR => ResponseCode::SLVERR,
                    AXI4Error::DECERR => ResponseCode::DECERR,
                },
            },
        };
        signal(Some(b))
    }
    let inputs = [
        (signal(Ok(bits(12))),),
        (signal(Err(AXI4Error::SLVERR)),),
        (signal(Err(AXI4Error::DECERR)),),
    ];
    test_kernel_vm_and_verilog::<do_stuff<W4>, _, _, _>(do_stuff::<W4>, inputs.into_iter())?;
    Ok(())
}

#[test]
fn test_option_result_nested_option_result_destructure() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        DECERR = 1,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        SLVERR = 1,
        DECERR = 2,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {
        let resp = i.val();
        let d = match resp {
            Some(Result::Ok(())) => Some(ResponseCode::OKAY),
            Some(Result::Err(e)) => Some(match e {
                AXI4Error::SLVERR => ResponseCode::SLVERR,
                AXI4Error::DECERR => ResponseCode::DECERR,
            }),
            None => None,
        };
        signal(d)
    }

    let expect = expect![[r#"Err(RHDLTypeError(RHDLTypeError { cause: PathMismatchInTypeInference, src: SourcePool { source: {FnID(28b0f91d4b051211): SpannedSource { source: "fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<ResponseCode>, Red> {\n    let resp = i.val();\n    let d = match resp {\n        Some(Result::Ok(())) => Some(ResponseCode::OKAY),\n        Some(Result::Err(e)) => {\n            Some(\n                match e {\n                    AXI4Error::SLVERR => ResponseCode::SLVERR,\n                    AXI4Error::DECERR => ResponseCode::DECERR,\n                },\n            )\n        }\n        None => None,\n    };\n    signal(d)\n}\n", name: "do_stuff", span_map: {N20: 288..330, N17: 198..218, N36: 0..481, N19: 309..329, N26: 445..449, N2: 91..95, N27: 453..457, N4: 98..105, N30: 111..465, N6: 87..106, N34: 470..479, N13: 164..188, N31: 111..465, N22: 351..393, N12: 169..187, N23: 258..411, N28: 445..458, N0: 12..13, N33: 470..479, N14: 140..189, N15: 215..216, N24: 236..426, N8: 125..129, N11: 140..160, N29: 119..464, N25: 198..436, N35: 81..481, N1: 12..42, N3: 98..99, N32: 477..478, N7: 115..116, N5: 87..106, N9: 156..158, N10: 145..159, N21: 372..392, N16: 203..217, N18: 264..265}, fallback: N36, filename: "rhdl/tests/option.rs:319", function_id: FnID(28b0f91d4b051211) }}, ranges: {FnID(28b0f91d4b051211): 0..482} }, err_span: SourceSpan { offset: SourceOffset(156), length: 2 } }))"#]];
    let res = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    expect.assert_eq(&format!("{:?}", res));
    Ok(())
}

#[test]
fn test_option_result_nested_option_result_destructure_simple() -> miette::Result<()> {
    #[derive(PartialEq, Default, Digital)]
    pub enum AXI4Error {
        #[default]
        SLVERR = 0,
        _UNUSED,
    }

    #[derive(PartialEq, Default, Digital)]
    pub enum ResponseCode {
        #[default]
        OKAY = 0,
        _UNUSED,
    }

    type Result = std::result::Result<(), AXI4Error>;

    #[kernel]
    fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<AXI4Error>, Red> {
        let resp = i.val();
        let d = match resp {
            Some(Result::Ok(())) => Some(AXI4Error::SLVERR),
            Some(Err(e)) => Some(e),
            None => None,
        };
        signal(d)
    }

    let expect_err = expect![[r#"Err(RHDLTypeError(RHDLTypeError { cause: PathMismatchInTypeInference, src: SourcePool { source: {FnID(f7b4ab65ff1d920a): SpannedSource { source: "fn do_stuff(i: Signal<Option<Result>, Red>) -> Signal<Option<AXI4Error>, Red> {\n    let resp = i.val();\n    let d = match resp {\n        Some(Result::Ok(())) => Some(AXI4Error::SLVERR),\n        Some(Err(e)) => Some(e),\n        None => None,\n    };\n    signal(d)\n}\n", name: "do_stuff", span_map: {N29: 252..261, N24: 116..246, N2: 88..92, N13: 161..184, N18: 215..216, N27: 259..260, N8: 122..126, N21: 227..231, N5: 84..103, N7: 112..113, N11: 137..157, N19: 210..217, N15: 203..204, N31: 0..263, N0: 12..13, N26: 108..247, N22: 235..239, N28: 252..261, N1: 12..42, N14: 137..185, N10: 142..156, N3: 95..96, N25: 108..247, N4: 95..102, N17: 194..206, N23: 227..240, N16: 199..205, N6: 84..103, N20: 194..218, N9: 153..155, N12: 166..183, N30: 78..263}, fallback: N31, filename: "rhdl/tests/option.rs:357", function_id: FnID(f7b4ab65ff1d920a) }}, ranges: {FnID(f7b4ab65ff1d920a): 0..264} }, err_span: SourceSpan { offset: SourceOffset(153), length: 2 } }))"#]];
    let err = compile_design::<do_stuff>(CompilationMode::Asynchronous);
    expect_err.assert_eq(&format!("{:?}", err));
    Ok(())
}
