pub trait LetChain {
    fn let_<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnOnce(Self) -> R;
}
impl<T> LetChain for T {
    fn let_<F, R>(self, f: F) -> R
    where
        Self: Sized,
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

pub trait AlsoChain {
    fn also_<F, R>(self, f: F) -> Self
    where
        Self: Sized,
        F: FnOnce(&mut Self) -> R;
}
impl<T> AlsoChain for T {
    fn also_<F, R>(mut self, f: F) -> Self
    where
        Self: Sized,
        F: FnOnce(&mut Self) -> R,
    {
        f(&mut self);
        self
    }
}

pub fn parse_date(s: &str) -> crate::entities::Date {
    ::chrono::DateTime::parse_from_rfc3339(s)
        .unwrap()
        .with_timezone(&::chrono::Utc)
}

pub fn date_to_string(dt: crate::entities::Date) -> String {
    dt.to_rfc3339_opts(::chrono::SecondsFormat::Nanos, true)
}

pub trait ConvertRange<T>: ::core::ops::RangeBounds<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>);
}
impl<T> ConvertRange<T> for ::core::ops::Range<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::Range { start, end } = self;
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Excluded(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeFrom<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeFrom { start } = self;
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Unbounded,
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeFull {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        (::core::ops::Bound::Unbounded, ::core::ops::Bound::Unbounded)
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeInclusive<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let (start, end) = self.into_inner();
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Included(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeTo<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeTo { end } = self;
        (
            ::core::ops::Bound::Unbounded,
            ::core::ops::Bound::Excluded(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeToInclusive<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeToInclusive { end } = self;
        (
            ::core::ops::Bound::Unbounded,
            ::core::ops::Bound::Included(end),
        )
    }
}
impl<T> ConvertRange<T> for (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) { self }
}
