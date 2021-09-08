pub(crate) trait LetChain {
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

pub(crate) trait AlsoChain {
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

pub(crate) fn parse_date(s: &str) -> crate::entities::Date {
    ::chrono::DateTime::parse_from_rfc3339(s)
        .unwrap()
        .with_timezone(&::chrono::Utc)
}

pub(crate) fn date_to_string(dt: crate::entities::Date) -> String {
    dt.to_rfc3339_opts(::chrono::SecondsFormat::Nanos, true)
}
