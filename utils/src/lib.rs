pub use InUse::*;

#[derive(Debug, PartialEq, Eq)]
pub enum WatchStatus {
    NeedsUpdate,
    Nothing,
}

impl std::ops::BitOr for WatchStatus {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        use WatchStatus::*;
        match (self, rhs) {
            // We are explicit about this pattern for code clarity:
            (NeedsUpdate, _) | (_, NeedsUpdate) => NeedsUpdate,
            (Nothing, Nothing) => Nothing,
        }
    }
}

impl std::ops::BitAnd for WatchStatus {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        use WatchStatus::*;
        match (self, rhs) {
            // We are explicit about this pattern for code clarity:
            (NeedsUpdate, NeedsUpdate) => NeedsUpdate,
            (Nothing, _) | (_, Nothing) => Nothing,
        }
    }
}

//pub trait Watchable<F, T> {
//    fn watch(&self) -> Watch<F, T>
//        where F: Fn() -> WatchStatus;
//}

pub struct Watch
{
    watchfn: Box<Fn() -> WatchStatus>,
//    reference: &'a T,
}

impl Watch
{
    pub fn new(watchfn: Box<Fn() -> WatchStatus>) -> Self {
        Self { watchfn: watchfn }
    }

    pub fn status(&self) -> WatchStatus {
        (*self.watchfn)()
    }

//    pub fn into_raw(&self) -> &'a T {
//        self.reference
//    }
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
            InUse::Released => panic!("Can't call as_ref on Released"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
//    #[test]
//    fn simple_status() {
//        let a = 2;
//        let watch = Watch::new(|| WatchStatus::NeedsUpdate, &a);
//        assert_eq!(watch.status(), WatchStatus::NeedsUpdate);
//    }
//
//    #[test]
//    fn bitor_status() {
//        let a = 1;
//        let w1 = Watch::new(|| WatchStatus::NeedsUpdate, &a);
//        let w2 = Watch::new(|| WatchStatus::Nothing, &a);
//        let cw1 = Watch::new(|| w1.status() | w2.status(), &a);
//        assert_eq!(cw1.status(), WatchStatus::NeedsUpdate);
//
//        let w3 = Watch::new(|| WatchStatus::Nothing, &a);
//        let w4 = Watch::new(|| WatchStatus::Nothing, &a);
//        let cw2 = Watch::new(|| w3.status() | w4.status(), &a);
//        assert_eq!(cw2.status(), WatchStatus::Nothing);
//    }
//
//    #[test]
//    fn bitand_status() {
//        let a = 1;
//        let w1 = Watch::new(|| WatchStatus::NeedsUpdate, &a);
//        let w2 = Watch::new(|| WatchStatus::NeedsUpdate, &a);
//        let cw1 = Watch::new(|| w1.status() & w2.status(), &a);
//        assert_eq!(cw1.status(), WatchStatus::NeedsUpdate);
//
//        let w3 = Watch::new(|| WatchStatus::Nothing, &a);
//        let w4 = Watch::new(|| WatchStatus::NeedsUpdate, &a);
//        let cw2 = Watch::new(|| w3.status() & w4.status(), &a);
//        assert_eq!(cw2.status(), WatchStatus::Nothing);
//    }

    #[test]
    fn simple_ref() {
        let a = 2;
        let watch = Watch::new(|| WatchStatus::NeedsUpdate, &a);
        assert_eq!(watch.into_raw(), &a);
    }
}
