use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use std::collections::HashMap;
use std::str::FromStr;
use vrl::prelude::*;

lazy_static! {
    static ref RE: Regex = Regex::new(
        r"(?ix)                        # i: case-insensitive, x: ignore whitespace + comments
            \A
            (?P<value>[0-9]*\.?[0-9]+) # value: integer or float
            \s?                        # optional space between value and unit
            (?P<unit>[µa-z]{1,2})      # unit: one or two letters
            \z"
    )
    .unwrap();
    static ref UNITS: HashMap<String, Decimal> = vec![
        ("ns", Decimal::new(1, 9)),
        ("us", Decimal::new(1, 6)),
        ("µs", Decimal::new(1, 6)),
        ("ms", Decimal::new(1, 3)),
        ("cs", Decimal::new(1, 2)),
        ("ds", Decimal::new(1, 1)),
        ("s", Decimal::new(1, 0)),
        ("m", Decimal::new(60, 0)),
        ("h", Decimal::new(3_600, 0)),
        ("d", Decimal::new(86_400, 0)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
}

#[derive(Clone, Copy, Debug)]
pub struct ParseDuration;

impl Function for ParseDuration {
    fn identifier(&self) -> &'static str {
        "parse_duration"
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "milliseconds",
            source: r#"parse_duration!("1005ms", unit: "s")"#,
            result: Ok("1.005"),
        }]
    }

    fn compile(
        &self,
        _state: &state::Compiler,
        _ctx: &FunctionCompileContext,
        mut arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        let unit = arguments.required("unit");

        Ok(Box::new(ParseDurationFn { value, unit }))
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::BYTES,
                required: true,
            },
            Parameter {
                keyword: "unit",
                kind: kind::BYTES,
                required: true,
            },
        ]
    }
}

#[derive(Debug, Clone)]
struct ParseDurationFn {
    value: Box<dyn Expression>,
    unit: Box<dyn Expression>,
}

impl Expression for ParseDurationFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let bytes = self.value.resolve(ctx)?.try_bytes()?;
        let value = String::from_utf8_lossy(&bytes);

        let conversion_factor = {
            let bytes = self.unit.resolve(ctx)?.try_bytes()?;
            let string = String::from_utf8_lossy(&bytes);

            UNITS
                .get(string.as_ref())
                .ok_or(format!("unknown unit format: '{}'", string))?
        };

        let captures = RE
            .captures(&value)
            .ok_or(format!("unable to parse duration: '{}'", value))?;

        let value = Decimal::from_str(&captures["value"])
            .map_err(|error| format!("unable to parse number: {}", error))?;

        let unit = UNITS
            .get(&captures["unit"])
            .ok_or(format!("unknown duration unit: '{}'", &captures["unit"]))?;

        let number = value * unit / conversion_factor;
        let number = number
            .to_f64()
            .ok_or(format!("unable to format duration: '{}'", number))?;

        Ok(number.into())
    }

    fn type_def(&self, _: &state::Compiler) -> TypeDef {
        TypeDef::new().fallible().float()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    test_function![
        parse_duration => ParseDuration;

        s_m {
            args: func_args![value: "30s",
                             unit: "m"],
            want: Ok(value!(0.5)),
            tdef: TypeDef::new().fallible().float(),
        }

        ms_ms {
            args: func_args![value: "100ms",
                             unit: "ms"],
            want: Ok(100.0),
            tdef: TypeDef::new().fallible().float(),
        }

        ms_s {
            args: func_args![value: "1005ms",
                             unit: "s"],
            want: Ok(1.005),
            tdef: TypeDef::new().fallible().float(),
        }

        ns_ms {
            args: func_args![value: "100ns",
                             unit: "ms"],
            want: Ok(0.0001),
            tdef: TypeDef::new().fallible().float(),
        }

        d_s {
            args: func_args![value: "1d",
                             unit: "s"],
            want: Ok(86400.0),
            tdef: TypeDef::new().fallible().float(),
        }

        s_ns {
            args: func_args![value: "1 s",
                             unit: "ns"],
            want: Ok(1000000000.0),
            tdef: TypeDef::new().fallible().float(),
        }

        us_ms {
            args: func_args![value: "1 µs",
                             unit: "ms"],
            want: Ok(0.001),
            tdef: TypeDef::new().fallible().float(),
        }

        error_invalid {
            args: func_args![value: "foo",
                             unit: "ms"],
            want: Err("unable to parse duration: 'foo'"),
            tdef: TypeDef::new().fallible().float(),
        }

        error_ns {
            args: func_args![value: "1",
                             unit: "ns"],
            want: Err("unable to parse duration: '1'"),
            tdef: TypeDef::new().fallible().float(),
        }

        error_unit {
            args: func_args![value: "1w",
                             unit: "ns"],
            want: Err("unknown duration unit: 'w'"),
            tdef: TypeDef::new().fallible().float(),
        }

        error_format {
            args: func_args![value: "1s",
                             unit: "w"],
            want: Err("unknown unit format: 'w'"),
            tdef: TypeDef::new().fallible().float(),
        }
    ];
}
