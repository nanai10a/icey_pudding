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

pub fn convert_range_display<T: ConvertRange<R> + Clone, R: ToString>(t: T) -> String {
    use ::core::ops::Bound::*;

    let (s, e) = t.to_turple();

    let ss = match s {
        Included(n) => n.to_string(),
        Excluded(n) => n.to_string(),
        Unbounded => String::new(),
    };

    let es = match e {
        Included(n) => n.to_string(),
        Excluded(n) => n.to_string(),
        Unbounded => String::new(),
    };

    format!("{}..{}", ss, es)
}

pub trait FutureTranspose {
    type To;

    fn transpose(self) -> Self::To;
}

impl<F, O> FutureTranspose for Option<F>
where F: ::core::future::Future<Output = O>
{
    type To = impl ::core::future::Future<Output = Option<O>>;

    fn transpose(self) -> Self::To {
        async {
            match self {
                None => None,
                Some(f) => Some(f.await),
            }
        }
    }
}
