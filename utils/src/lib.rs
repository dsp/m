pub use InUse::*;

#[derive(Debug, PartialEq, Eq)]
pub enum WatchStatus {
    NeedsUpdate,
    Nothing,
}

//pub trait Watchable {
//    fn watch() -> Watch;
//}

pub struct Watch<'a, F, T>
where
    F: Fn() -> WatchStatus,
{
    watchfn: F,
    reference: &'a T,
}

impl<'a, F, T> Watch<'a, F, T>
where
    F: Fn() -> WatchStatus,
{
    pub fn new(watchfn: F, reference: &'a T) -> Self {
        Self { watchfn, reference }
    }

    pub fn status(&self) -> WatchStatus {
        (self.watchfn)()
    }

    pub fn into_raw(&self) -> &'a T {
        self.reference
    }
}

pub enum InUse<T> {
    Released,
    Used(T),
}

impl<T> InUse<T> {
    pub fn take(&mut self) -> T {
        match std::mem::replace(self, InUse::Released) {
            InUse::Used(t) => t,
            InUse::Released => panic!("Can't call take on Released"),
        }
    }

    pub fn as_ref(&self) -> &T {
        match self {
            InUse::Used(t) => &t,
            InUse::Released=> panic!("Can't call as_ref on Released"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple_status() {
        let a = 2;
        let watch = Watch::new(|| WatchStatus::NeedsUpdate, &a);
        assert_eq!(watch.status(), WatchStatus::NeedsUpdate);
    }

    #[test]
    fn simple_ref() {
        let a = 2;
        let watch = Watch::new(|| WatchStatus::NeedsUpdate, &a);
        assert_eq!(watch.into_raw(), &a);
    }
}

