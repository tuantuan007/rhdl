use rhdl_bits::Bits;
use rhdl_bits::SignedBits;
use rhdl_core::error::RHDLError;
use rhdl_core::kernel::ExternalKernelDef;
use rhdl_core::kernel::KernelFnKind;
use rhdl_core::DigitalFn;

use crate::impl_get_bit::get_bit;

pub fn as_signed<const N: usize>(x: Bits<N>) -> SignedBits<N> {
    // Need a sign extension here.
    if get_bit(x, (N - 1).try_into().unwrap()) {
        SignedBits((x.0 | !(Bits::<N>::mask().0)) as i128)
    } else {
        SignedBits(x.0 as i128)
    }
}

fn vm_as_signed(args: &[rhdl_core::TypedBits]) -> Result<rhdl_core::TypedBits, RHDLError> {
    args[0].as_signed()
}

#[allow(non_camel_case_types)]
pub struct as_signed<const N: usize> {}

impl<const N: usize> DigitalFn for as_signed<N> {
    fn kernel_fn() -> Option<KernelFnKind> {
        Some(        
            KernelFnKind::Extern(ExternalKernelDef {
            name: format!("signed_{N}"),
            body: format!(
                "function signed [{}:0] signed_{N}(input [{}:0] a); signed_{N} = $signed(a); endfunction",
                N - 1,
                N - 1,
            ),
            vm_stub: Some(vm_as_signed),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed() {
        assert_eq!(as_signed(Bits::<8>(0b1111_1111)), SignedBits::<8>(-1));
        assert_eq!(as_signed(Bits::<8>(0b0000_0000)), SignedBits::<8>(0));
        assert_eq!(as_signed(Bits::<8>(0b0000_0001)), SignedBits::<8>(1));
    }

    #[test]
    fn test_iverilog() -> Result<(), RHDLError> {
        let test_values = (0..=255).map(Bits::<8>::from).map(|x| (x,));
        rhdl_core::test_with_iverilog(
            as_signed::<8>,
            as_signed::<8>::kernel_fn().unwrap().try_into()?,
            test_values,
        )
    }
}
