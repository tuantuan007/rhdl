fn do_stuff<T: Digital, S: Digital>(x: Foo<T>, y: Foo<S>) -> bool {
    let c = x.a;
    let d = (x.a, y.b);
    let e = Foo::<T> { a: c, b: c };
    e == x
}
struct do_stuff<T, S> {
    __phantom_0: std::marker::PhantomData<T>,
    __phantom_1: std::marker::PhantomData<S>,
}
impl<T: Digital, S: Digital> rhdl::core::digital_fn::DigitalFn for do_stuff<T, S> {
    fn kernel_fn() -> Box<rhdl::core::ast::KernelFn> {
        rhdl::core::ast::builder::kernel_fn(
            stringify!(do_stuff),
            vec![
                rhdl::core::ast::builder::type_pat(
                    rhdl::core::ast::builder::ident_pat(stringify!(x).to_string(), false),
                    <Foo<T> as rhdl::core::Digital>::static_kind(),
                ),
                rhdl::core::ast::builder::type_pat(
                    rhdl::core::ast::builder::ident_pat(stringify!(y).to_string(), false),
                    <Foo<S> as rhdl::core::Digital>::static_kind(),
                ),
            ],
            <bool as rhdl::core::Digital>::static_kind(),
            rhdl::core::ast::builder::block(vec![
                rhdl::core::ast::builder::local_stmt(
                    rhdl::core::ast::builder::ident_pat(stringify!(c).to_string(), false),
                    Some(rhdl::core::ast::builder::field_expr(
                        rhdl::core::ast::builder::path_expr(rhdl::core::ast::builder::path(vec![
                            rhdl::core::ast::builder::path_segment(
                                stringify!(x).to_string(),
                                rhdl::core::ast::builder::path_arguments_none(),
                            ),
                        ])),
                        rhdl::core::ast::builder::member_named(stringify!(a).to_string()),
                    )),
                ),
                rhdl::core::ast::builder::local_stmt(
                    rhdl::core::ast::builder::ident_pat(stringify!(d).to_string(), false),
                    Some(rhdl::core::ast::builder::tuple_expr(vec![
                        rhdl::core::ast::builder::field_expr(
                            rhdl::core::ast::builder::path_expr(rhdl::core::ast::builder::path(
                                vec![rhdl::core::ast::builder::path_segment(
                                    stringify!(x).to_string(),
                                    rhdl::core::ast::builder::path_arguments_none(),
                                )],
                            )),
                            rhdl::core::ast::builder::member_named(stringify!(a).to_string()),
                        ),
                        rhdl::core::ast::builder::field_expr(
                            rhdl::core::ast::builder::path_expr(rhdl::core::ast::builder::path(
                                vec![rhdl::core::ast::builder::path_segment(
                                    stringify!(y).to_string(),
                                    rhdl::core::ast::builder::path_arguments_none(),
                                )],
                            )),
                            rhdl::core::ast::builder::member_named(stringify!(b).to_string()),
                        ),
                    ])),
                ),
                rhdl::core::ast::builder::local_stmt(
                    rhdl::core::ast::builder::ident_pat(stringify!(e).to_string(), false),
                    Some(rhdl::core::ast::builder::struct_expr(
                        rhdl::core::ast::builder::path(vec![
                            rhdl::core::ast::builder::path_segment(
                                stringify!(Foo).to_string(),
                                rhdl::core::ast::builder::path_arguments_angle_bracketed(vec![
                                    rhdl::core::ast::builder::generic_argument_type(
                                        <T as Digital>::static_kind(),
                                    ),
                                ]),
                            ),
                        ]),
                        vec![
                            rhdl::core::ast::builder::field_value(
                                rhdl::core::ast::builder::member_named(stringify!(a).to_string()),
                                rhdl::core::ast::builder::path_expr(
                                    rhdl::core::ast::builder::path(vec![
                                        rhdl::core::ast::builder::path_segment(
                                            stringify!(c).to_string(),
                                            rhdl::core::ast::builder::path_arguments_none(),
                                        ),
                                    ]),
                                ),
                            ),
                            rhdl::core::ast::builder::field_value(
                                rhdl::core::ast::builder::member_named(stringify!(b).to_string()),
                                rhdl::core::ast::builder::path_expr(
                                    rhdl::core::ast::builder::path(vec![
                                        rhdl::core::ast::builder::path_segment(
                                            stringify!(c).to_string(),
                                            rhdl::core::ast::builder::path_arguments_none(),
                                        ),
                                    ]),
                                ),
                            ),
                        ],
                        None,
                        Digital::kind(&Foo::<T> {
                            a: Default::default(),
                            b: Default::default(),
                        }),
                    )),
                ),
                rhdl::core::ast::builder::expr_stmt(rhdl::core::ast::builder::binary_expr(
                    rhdl::core::ast::builder::BinOp::Eq,
                    rhdl::core::ast::builder::path_expr(rhdl::core::ast::builder::path(vec![
                        rhdl::core::ast::builder::path_segment(
                            stringify!(e).to_string(),
                            rhdl::core::ast::builder::path_arguments_none(),
                        ),
                    ])),
                    rhdl::core::ast::builder::path_expr(rhdl::core::ast::builder::path(vec![
                        rhdl::core::ast::builder::path_segment(
                            stringify!(x).to_string(),
                            rhdl::core::ast::builder::path_arguments_none(),
                        ),
                    ])),
                )),
            ]),
        )
    }
}
