use crate::util::splice;

use super::spec::{
    Assign, Binary, Case, Cast, Concat, DynamicIndex, DynamicSplice, Index, OpCode, Select, Splice,
    Unary,
};

impl std::fmt::Debug for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Assign(Assign { lhs, rhs }) => {
                write!(f, " {:?} <- {:?}", lhs, rhs)
            }
            OpCode::Binary(Binary {
                op,
                lhs,
                arg1,
                arg2,
            }) => {
                write!(f, " {:?} <- {:?} {:?} {:?}", lhs, arg1, op, arg2)
            }
            OpCode::Case(Case {
                lhs,
                discriminant,
                table,
            }) => {
                writeln!(f, " {:?} <- case {:?} {{", lhs, discriminant)?;
                for (cond, val) in table {
                    writeln!(f, "         {:?} => {:?}", cond, val)?;
                }
                write!(f, "}}")
            }
            OpCode::Cast(Cast {
                lhs,
                arg,
                len,
                signed,
            }) => {
                write!(
                    f,
                    " {:?} <- {:?} as {}{}",
                    lhs,
                    arg,
                    if *signed { "s" } else { "u" },
                    len
                )
            }
            OpCode::Comment(comment) => {
                write!(f, "// {}", comment)
            }
            OpCode::Concat(Concat { lhs, args }) => {
                write!(f, " {:?} <- {{ {} }}", lhs, splice(args, ", "))
            }
            OpCode::DynamicIndex(DynamicIndex {
                lhs,
                arg,
                offset,
                len,
            }) => {
                write!(f, " {:?} <- {:?}[{:?} +: {:?}]", lhs, arg, offset, len)
            }
            OpCode::DynamicSplice(DynamicSplice {
                lhs,
                arg,
                offset,
                len,
                value,
            }) => {
                write!(
                    f,
                    " {lhs:?} <- {arg:?}; {lhs:?}[{offset:?} +: {len}] <- {value:?}"
                )
            }
            OpCode::Index(Index {
                lhs,
                arg,
                bit_range,
            }) => {
                write!(f, " {:?} <- {:?}[{:?}]", lhs, arg, bit_range)
            }
            OpCode::Select(Select {
                lhs,
                cond,
                true_value,
                false_value,
            }) => {
                write!(
                    f,
                    " {:?} <- {:?} ? {:?} : {:?}",
                    lhs, cond, true_value, false_value
                )
            }
            OpCode::Splice(Splice {
                lhs,
                orig,
                bit_range,
                value,
            }) => {
                write!(f, " {:?} <- {:?}/{:?}/{:?}", lhs, value, bit_range, orig)
            }
            OpCode::Unary(Unary { op, lhs, arg1 }) => {
                write!(f, " {:?} <- {:?}{:?}", lhs, op, arg1)
            }
        }
    }
}