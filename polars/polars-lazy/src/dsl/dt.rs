use polars_time::prelude::TemporalMethods;

use super::*;
use crate::prelude::function_expr::TemporalFunction;

/// Specialized expressions for [`Series`] with dates/datetimes.
pub struct DateLikeNameSpace(pub(crate) Expr);

impl DateLikeNameSpace {
    /// Format Date/datetime with a formatting rule
    /// See [chrono strftime/strptime](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html).
    pub fn strftime(self, fmt: &str) -> Expr {
        let fmt = fmt.to_string();
        let function = move |s: Series| s.strftime(&fmt);
        self.0
            .map(function, GetOutput::from_type(DataType::Utf8))
            .with_fmt("strftime")
    }

    /// Change the underlying [`TimeUnit`]. And update the data accordingly.
    pub fn cast_time_unit(self, tu: TimeUnit) -> Expr {
        self.0.map(
            move |s| match s.dtype() {
                DataType::Datetime(_, _) => {
                    let ca = s.datetime().unwrap();
                    Ok(ca.cast_time_unit(tu).into_series())
                }
                #[cfg(feature = "dtype-duration")]
                DataType::Duration(_) => {
                    let ca = s.duration().unwrap();
                    Ok(ca.cast_time_unit(tu).into_series())
                }
                dt => Err(PolarsError::ComputeError(
                    format!("Series of dtype {:?} has got no time unit", dt).into(),
                )),
            },
            GetOutput::map_dtype(move |dtype| match dtype {
                DataType::Duration(_) => DataType::Duration(tu),
                DataType::Datetime(_, tz) => DataType::Datetime(tu, tz.clone()),
                _ => panic!("expected duration or datetime"),
            }),
        )
    }

    /// Change the underlying [`TimeUnit`] of the [`Series`]. This does not modify the data.
    pub fn with_time_unit(self, tu: TimeUnit) -> Expr {
        self.0.map(
            move |s| match s.dtype() {
                DataType::Datetime(_, _) => {
                    let mut ca = s.datetime().unwrap().clone();
                    ca.set_time_unit(tu);
                    Ok(ca.into_series())
                }
                #[cfg(feature = "dtype-duration")]
                DataType::Duration(_) => {
                    let mut ca = s.duration().unwrap().clone();
                    ca.set_time_unit(tu);
                    Ok(ca.into_series())
                }
                dt => Err(PolarsError::ComputeError(
                    format!("Series of dtype {:?} has got no time unit", dt).into(),
                )),
            },
            GetOutput::same_type(),
        )
    }

    /// Change the underlying [`TimeZone`] of the [`Series`]. This does not modify the data.
    pub fn with_time_zone(self, tz: Option<TimeZone>) -> Expr {
        self.0.map(
            move |s| match s.dtype() {
                DataType::Datetime(_, _) => {
                    let mut ca = s.datetime().unwrap().clone();
                    ca.set_time_zone(tz.clone());
                    Ok(ca.into_series())
                }
                dt => Err(PolarsError::ComputeError(
                    format!("Series of dtype {:?} has got no time zone", dt).into(),
                )),
            },
            GetOutput::same_type(),
        )
    }

    /// Get the year of a Date/Datetime
    pub fn year(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Year))
    }

    /// Get the iso-year of a Date/Datetime.
    /// This may not correspond with a calendar year.
    pub fn iso_year(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::IsoYear))
    }

    /// Get the month of a Date/Datetime
    pub fn month(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Month))
    }

    /// Extract quarter from underlying NaiveDateTime representation.
    /// Quarters range from 1 to 4.
    pub fn quarter(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Quarter))
    }

    /// Extract the week from the underlying Date representation.
    /// Can be performed on Date and Datetime

    /// Returns the ISO week number starting from 1.
    /// The return value ranges from 1 to 53. (The last week of year differs by years.)
    pub fn week(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Week))
    }

    /// Extract the week day from the underlying Date representation.
    /// Can be performed on Date and Datetime.

    /// Returns the weekday number where monday = 0 and sunday = 6
    pub fn weekday(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::WeekDay))
    }

    /// Get the month of a Date/Datetime
    pub fn day(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Day))
    }
    /// Get the ordinal_day of a Date/Datetime
    pub fn ordinal_day(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::OrdinalDay))
    }
    /// Get the hour of a Datetime/Time64
    pub fn hour(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Hour))
    }
    /// Get the minute of a Datetime/Time64
    pub fn minute(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Minute))
    }

    /// Get the second of a Datetime/Time64
    pub fn second(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::Second))
    }
    /// Get the nanosecond of a Time64
    pub fn nanosecond(self) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::NanoSecond))
    }

    pub fn timestamp(self, tu: TimeUnit) -> Expr {
        self.0
            .map_private(FunctionExpr::TemporalExpr(TemporalFunction::TimeStamp(tu)))
    }

    /// Offset this `Date/Datetime` by a given offset [`Duration`].
    /// This will take leap years/ months into account.
    #[cfg(feature = "date_offset")]
    pub fn offset_by(self, by: Duration) -> Expr {
        self.0.map_private(FunctionExpr::DateOffset(by))
    }
}
