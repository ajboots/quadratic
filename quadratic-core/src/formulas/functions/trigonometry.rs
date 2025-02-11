use super::*;

pub const CATEGORY: FormulaFunctionCategory = FormulaFunctionCategory {
    include_in_docs: true,
    include_in_completions: true,
    name: "Trigonometric functions",
    docs: "",
    get_functions,
};

fn get_functions() -> Vec<FormulaFunction> {
    macro_rules! trig_functions_and_inverses {
        (
            url: $url:literal;
            inverse_url: $inv_url:literal;
            $(
                $short_name:literal, $full_name:literal, $f:expr, $inv_f:expr $(, $inv_range:literal)?
            );+ $(;)?
        ) => {
            vec![
                $(
                    {
                        let f: fn(f64) -> f64 = $f;
                        FormulaFunction {
                            name: $short_name,
                            arg_completion: "${1:radians}",
                            usages: &["radians"],
                            examples: &[concat!($short_name, "(PI() * 2/3)")],
                            doc: concat!(
                                "Returns the [", $full_name, "](", $url, ") ",
                                "of an angle in radians.",
                            ),
                            eval: util::array_mapped(move |[radians]| Ok(Value::Number(f(radians.to_number()?)))),
                        }
                    },
                    {
                        let inv_f: fn(f64) -> f64 = $inv_f;
                        FormulaFunction {
                            name: concat!("A", $short_name),
                            arg_completion: "${1:number}",
                            usages: &["number"],
                            examples: &[concat!($short_name, "(A1)")],
                            doc: concat!(
                                "Returns the [inverse ", $full_name, "](", $inv_url, ") ",
                                "of a number, in radians", $(", ranging from ", $inv_range,)?
                                ".",
                            ),
                            eval: util::array_mapped(move |[number]| Ok(Value::Number(inv_f(number.to_number()?)))),
                        }
                    },
                )+
            ]
        };
    }

    let mut all_trig_functions = vec![
        FormulaFunction {
            name: "DEGREES",
            arg_completion: "${1:radians}",
            usages: &["radians"],
            examples: &["DEGREES(PI() / 2)"],
            doc: "Converts radians to degrees",
            eval: util::array_mapped(|[radians]| {
                Ok(Value::Number(radians.to_number()?.to_degrees()))
            }),
        },
        FormulaFunction {
            name: "RADIANS",
            arg_completion: "${1:degrees}",
            usages: &["degrees"],
            examples: &["RADIANS(90)"],
            doc: "Converts degrees to radians",
            eval: util::array_mapped(|[degrees]| {
                Ok(Value::Number(degrees.to_number()?.to_radians()))
            }),
        },
    ];

    all_trig_functions.extend(trig_functions_and_inverses![
        url: "https://en.wikipedia.org/wiki/Trigonometric_functions";
        inverse_url: "https://en.wikipedia.org/wiki/Inverse_trigonometric_functions";
        //                              function,             inverse function,        inv_range;
        "SIN",  "sine",                 |x| x.sin(),          |x| x.asin(),            "0 to π";
        "COS",  "cosine",               |x| x.cos(),          |x| x.acos(),            "0 to π";
        "TAN",  "tangent",              |x| x.tan(),          |x| x.atan(),         "-π/2 to π/2";
        "CSC",  "cosecant",             |x| x.sin().recip(),  |x| x.recip().asin(), "-π/2 to π/2";
        "SEC",  "secant",               |x| x.cos().recip(),  |x| x.recip().acos(),    "0 to π";
        "COT",  "cotangent",            |x| x.tan().recip(),  arc_cotangent,           "0 to π";
    ]);

    all_trig_functions.extend(trig_functions_and_inverses![
        url: "https://en.wikipedia.org/wiki/Hyperbolic_functions";
        inverse_url: "https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions";
        //                              function,             inverse function;
        "SINH", "hyperbolic sine",      |x| x.sinh(),         |x| x.asinh();
        "COSH", "hyperbolic cosine",    |x| x.cosh(),         |x| x.acosh();
        "TANH", "hyperbolic tangent",   |x| x.tanh(),         |x| x.atanh();
        "CSCH", "hyperbolic cosecant",  |x| x.sinh().recip(), |x| x.recip().asinh();
        "SECH", "hyperbolic secant",    |x| x.cosh().recip(), |x| x.recip().acosh();
        "COTH", "hyperbolic cotangent", |x| x.tanh().recip(), |x| x.recip().atanh();
    ]);

    let index_of_atan = 5;
    all_trig_functions.insert(
        index_of_atan + 1,
        FormulaFunction {
            name: "ATAN2",
            arg_completion: "{$1:x}, {$2:y}",
            usages: &["x, y"],
            examples: &["ATAN2(2, 1)"],
            doc: "Returns the counterclockwise angle, in radians, from the X axis to the \
                  point `(x, y)`. Note that the argument order is reversed compared to \
                  the [typical `atan2()` function](https://en.wikipedia.org/wiki/Atan2).",
            eval: util::array_mapped(|[x, y]| {
                Ok(Value::Number(f64::atan2(y.to_number()?, x.to_number()?)))
            }),
        },
    );

    all_trig_functions
}

/// Inverse cotangent function with the correct range.
///
/// If we just use `.recip().atan()`, then the range is discontinuous and we
/// lose `acot(0) = 0`. So this function does it properly.
fn arc_cotangent(x: f64) -> f64 {
    if x > 0.0 {
        x.recip().atan()
    } else if x < 0.0 {
        x.recip().atan() + std::f64::consts::PI
    } else {
        std::f64::consts::FRAC_PI_2
    }
}

#[cfg(test)]
mod tests {

    use crate::formulas::tests::*;

    fn test_trig_fn(name: &str, input_output_pairs: &[(f64, f64)]) {
        let g = &mut NoGrid;
        for &(input, expected_output) in input_output_pairs {
            println!("Testing that {name}({input}) = {expected_output}");
            crate::util::assert_f64_approx_eq(
                expected_output,
                &eval_to_string(g, &format!("{name}({input})")),
            );
        }
    }

    use std::f64::consts::PI;

    #[test]
    fn test_formula_radian_degree_conversion() {
        let g = &mut NoGrid;

        assert_eq!("-4", eval_to_string(g, "RADIANS(-720) / PI()"));
        assert_eq!("-720", eval_to_string(g, "DEGREES(-PI() * 4)"));
    }

    #[test]
    fn test_formula_trigonometry() {
        let g = &mut NoGrid;

        let test_cases = &[
            (-2.0 * PI, 0.0),
            (-1.5 * PI, 1.0),
            (-1.0 * PI, 0.0),
            (-0.75 * PI, -0.70711),
            (-0.5 * PI, -1.0),
            (-0.25 * PI, -0.70711),
            (0.0 * PI, 0.0),
            (0.25 * PI, 0.70711),
            (0.5 * PI, 1.0),
            (0.75 * PI, 0.70711),
            (1.0 * PI, 0.0),
            (1.5 * PI, -1.0),
            (2.0 * PI, 0.0),
        ];
        test_trig_fn("SIN", test_cases);

        let test_cases = &[
            (-2.0 * PI, 1.0),
            (-1.5 * PI, 0.0),
            (-1.0 * PI, -1.0),
            (-0.75 * PI, -0.70711),
            (-0.5 * PI, 0.0),
            (-0.25 * PI, 0.70711),
            (0.0 * PI, 1.0),
            (0.25 * PI, 0.70711),
            (0.5 * PI, 0.0),
            (0.75 * PI, -0.70711),
            (1.0 * PI, -1.0),
            (1.5 * PI, 0.0),
            (2.0 * PI, 1.0),
        ];
        test_trig_fn("COS", test_cases);

        let test_cases = &[
            (-2.0 * PI, 0.0),
            (-1.0 * PI, 0.0),
            (-0.75 * PI, 1.0),
            (-0.25 * PI, -1.0),
            (0.0 * PI, 0.0),
            (0.25 * PI, 1.0),
            (0.75 * PI, -1.0),
            (1.0 * PI, 0.0),
            (2.0 * PI, 0.0),
        ];
        test_trig_fn("TAN", test_cases);

        let test_cases = &[
            (-1.5 * PI, 1.0),
            (-0.75 * PI, -1.41421),
            (-0.5 * PI, -1.0),
            (-0.25 * PI, -1.41421),
            (0.25 * PI, 1.41421),
            (0.5 * PI, 1.0),
            (0.75 * PI, 1.41421),
            (1.5 * PI, -1.0),
        ];
        test_trig_fn("CSC", test_cases);

        let test_cases = &[
            (-2.0 * PI, 1.0),
            (-1.0 * PI, -1.0),
            (-0.75 * PI, -1.41421),
            (-0.25 * PI, 1.41421),
            (0.0 * PI, 1.0),
            (0.25 * PI, 1.41421),
            (0.75 * PI, -1.41421),
            (1.0 * PI, -1.0),
            (2.0 * PI, 1.0),
        ];
        test_trig_fn("SEC", test_cases);

        let test_cases = &[
            (-1.5 * PI, 0.0),
            (-0.9 * PI, 3.07768),
            (-0.75 * PI, 1.0),
            (-0.5 * PI, 0.0),
            (-0.25 * PI, -1.0),
            (0.25 * PI, 1.0),
            (0.5 * PI, 0.0),
            (0.75 * PI, -1.0),
            (0.9 * PI, -3.07768),
            (1.5 * PI, 0.0),
        ];
        test_trig_fn("COT", test_cases);

        let test_cases = &[
            (-2.0 * PI, -267.74489),
            (-1.5 * PI, -55.6544),
            (-1.0 * PI, -11.54874),
            (-0.75 * PI, -5.22797),
            (-0.5 * PI, -2.3013),
            (-0.25 * PI, -0.86867),
            (0.0 * PI, 0.0),
            (0.25 * PI, 0.86867),
            (0.5 * PI, 2.3013),
            (0.75 * PI, 5.22797),
            (1.0 * PI, 11.54874),
            (1.5 * PI, 55.6544),
            (2.0 * PI, 267.74489),
        ];
        test_trig_fn("SINH", test_cases);

        let test_cases = &[
            (-2.0 * PI, 267.74676),
            (-1.5 * PI, 55.66338),
            (-1.0 * PI, 11.59195),
            (-0.75 * PI, 5.32275),
            (-0.5 * PI, 2.50918),
            (-0.25 * PI, 1.32461),
            (0.0 * PI, 1.0),
            (0.25 * PI, 1.32461),
            (0.5 * PI, 2.50918),
            (0.75 * PI, 5.32275),
            (1.0 * PI, 11.59195),
            (1.5 * PI, 55.66338),
            (2.0 * PI, 267.74676),
        ];
        test_trig_fn("COSH", test_cases);

        let test_cases = &[
            (-2.0 * PI, -0.99999),
            (-1.5 * PI, -0.99984),
            (-1.0 * PI, -0.99627),
            (-0.75 * PI, -0.98219),
            (-0.5 * PI, -0.91715),
            (-0.25 * PI, -0.65579),
            (0.0 * PI, 0.0),
            (0.25 * PI, 0.65579),
            (0.5 * PI, 0.91715),
            (0.75 * PI, 0.98219),
            (1.0 * PI, 0.99627),
            (1.5 * PI, 0.99984),
            (2.0 * PI, 0.99999),
        ];
        test_trig_fn("TANH", test_cases);

        let test_cases = &[
            (-2.0 * PI, -0.00373),
            (-1.5 * PI, -0.01797),
            (-1.0 * PI, -0.08659),
            (-0.75 * PI, -0.19128),
            (-0.5 * PI, -0.43454),
            (-0.25 * PI, -1.15118),
            (0.25 * PI, 1.15118),
            (0.5 * PI, 0.43454),
            (0.75 * PI, 0.19128),
            (1.0 * PI, 0.08659),
            (1.5 * PI, 0.01797),
            (2.0 * PI, 0.00373),
        ];
        test_trig_fn("CSCH", test_cases);

        let test_cases = &[
            (-2.0 * PI, 0.00373),
            (-1.5 * PI, 0.01797),
            (-1.0 * PI, 0.08627),
            (-0.75 * PI, 0.18787),
            (-0.5 * PI, 0.39854),
            (-0.25 * PI, 0.75494),
            (0.0 * PI, 1.0),
            (0.25 * PI, 0.75494),
            (0.5 * PI, 0.39854),
            (0.75 * PI, 0.18787),
            (1.0 * PI, 0.08627),
            (1.5 * PI, 0.01797),
            (2.0 * PI, 0.00373),
        ];
        test_trig_fn("SECH", test_cases);

        let test_cases = &[
            (-2.0 * PI, -1.00001),
            (-1.5 * PI, -1.00016),
            (-1.0 * PI, -1.00374),
            (-0.75 * PI, -1.01813),
            (-0.5 * PI, -1.09033),
            (-0.25 * PI, -1.52487),
            (0.25 * PI, 1.52487),
            (0.5 * PI, 1.09033),
            (0.75 * PI, 1.01813),
            (1.0 * PI, 1.00374),
            (1.5 * PI, 1.00016),
            (2.0 * PI, 1.00001),
        ];
        test_trig_fn("COTH", test_cases);

        assert!(eval_to_string(g, "ATAN2(2, 1)").starts_with("0.4636"));
    }

    #[test]
    fn test_formula_inverse_trigonometry() {
        let test_cases = &[
            (-1.0, -0.5 * PI),
            (-0.70711, -0.25 * PI),
            (0.0, 0.0 * PI),
            (0.70711, 0.25 * PI),
            (1.0, 0.5 * PI),
        ];
        test_trig_fn("ASIN", test_cases);

        let test_cases = &[
            (1.0, 0.0 * PI),
            (0.70711, 0.25 * PI),
            (0.0, 0.5 * PI),
            (-0.70711, 0.75 * PI),
            (-1.0, 1.0 * PI),
        ];
        test_trig_fn("ACOS", test_cases);

        let test_cases = &[(-1.0, -0.25 * PI), (0.0, 0.0 * PI), (1.0, 0.25 * PI)];
        test_trig_fn("ATAN", test_cases);

        let test_cases = &[
            (-1.0, -0.5 * PI),
            (-1.41421, -0.25 * PI),
            (1.41421, 0.25 * PI),
            (1.0, 0.5 * PI),
        ];
        test_trig_fn("ACSC", test_cases);

        let test_cases = &[
            (1.0, 0.0 * PI),
            (1.41421, 0.25 * PI),
            (-1.41421, 0.75 * PI),
            (-1.0, 1.0 * PI),
        ];
        test_trig_fn("ASEC", test_cases);

        let test_cases = &[
            (1.0, 0.25 * PI),
            (0.0, 0.5 * PI),
            (-1.0, 0.75 * PI),
            (-3.07768, 0.9 * PI),
        ];
        test_trig_fn("ACOT", test_cases);

        let test_cases = &[
            (-267.74489, -2.0 * PI),
            (-55.6544, -1.5 * PI),
            (-11.54874, -1.0 * PI),
            (-5.22797, -0.75 * PI),
            (-2.3013, -0.5 * PI),
            (-0.86867, -0.25 * PI),
            (0.0, 0.0 * PI),
            (0.86867, 0.25 * PI),
            (2.3013, 0.5 * PI),
            (5.22797, 0.75 * PI),
            (11.54874, 1.0 * PI),
            (55.6544, 1.5 * PI),
            (267.74489, 2.0 * PI),
        ];
        test_trig_fn("ASINH", test_cases);

        let test_cases = &[
            (1.0, 0.0 * PI),
            (1.32461, 0.25 * PI),
            (2.50918, 0.5 * PI),
            (5.32275, 0.75 * PI),
            (11.59195, 1.0 * PI),
            (55.66338, 1.5 * PI),
            (267.74676, 2.0 * PI),
        ];
        test_trig_fn("ACOSH", test_cases);

        let test_cases = &[
            (-0.99627208, -1.0 * PI),
            (-0.98219338, -0.75 * PI),
            (-0.91715234, -0.5 * PI),
            (-0.6557942, -0.25 * PI),
            (0.0, 0.0 * PI),
            (0.6557942, 0.25 * PI),
            (0.91715234, 0.5 * PI),
            (0.98219338, 0.75 * PI),
            (0.99627208, 1.0 * PI),
        ];
        test_trig_fn("ATANH", test_cases);

        let test_cases = &[
            (-0.0037349, -2.0 * PI),
            (-0.01796803, -1.5 * PI),
            (-0.08658954, -1.0 * PI),
            (-0.19127876, -0.75 * PI),
            (-0.43453721, -0.5 * PI),
            (-1.15118387, -0.25 * PI),
            (1.15118387, 0.25 * PI),
            (0.43453721, 0.5 * PI),
            (0.19127876, 0.75 * PI),
            (0.08658954, 1.0 * PI),
            (0.01796803, 1.5 * PI),
            (0.0037349, 2.0 * PI),
        ];
        test_trig_fn("ACSCH", test_cases);

        let test_cases = &[
            (1.0, 0.0 * PI),
            (0.754939709, 0.25 * PI),
            (0.398536815, 0.5 * PI),
            (0.187872734, 0.75 * PI),
            (0.086266738, 1.0 * PI),
            (0.017965132, 1.5 * PI),
            (0.003734872, 2.0 * PI),
        ];
        test_trig_fn("ASECH", test_cases);

        let test_cases = &[
            (-1.000161412, -1.5 * PI),
            (-1.00374187, -1.0 * PI),
            (-1.01812944, -0.75 * PI),
            (-1.09033141, -0.5 * PI),
            (-1.52486862, -0.25 * PI),
            (1.52486862, 0.25 * PI),
            (1.09033141, 0.5 * PI),
            (1.01812944, 0.75 * PI),
            (1.00374187, 1.0 * PI),
            (1.000161412, 1.5 * PI),
        ];
        test_trig_fn("ACOTH", test_cases);
    }
}
