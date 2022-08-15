use core::fmt::Debug;
use core::result::Result;
use core::time::Duration;

pub trait ErrorType {
    type Error: Debug;
}

impl<E> ErrorType for &E
where
    E: ErrorType,
{
    type Error = E::Error;
}

impl<E> ErrorType for &mut E
where
    E: ErrorType,
{
    type Error = E::Error;
}

pub trait Spin: ErrorType {
    fn spin(&mut self, duration: Option<Duration>) -> Result<(), Self::Error>;
}

pub trait Postbox<P>: ErrorType {
    fn post(&mut self, payload: &P, wait: Option<Duration>) -> Result<bool, Self::Error>;
}

impl<'a, P, PB> Postbox<P> for &'a mut PB
where
    PB: Postbox<P> + ErrorType,
{
    fn post(&mut self, payload: &P, wait: Option<Duration>) -> Result<bool, Self::Error> {
        (*self).post(payload, wait)
    }
}

pub trait EventBus<P>: ErrorType {
    type Subscription;

    fn subscribe(
        &mut self,
        callback: impl for<'a> FnMut(&'a P) + Send + 'static,
    ) -> Result<Self::Subscription, Self::Error>;
}

impl<'a, P, E> EventBus<P> for &'a mut E
where
    E: EventBus<P>,
{
    type Subscription = E::Subscription;

    fn subscribe(
        &mut self,
        callback: impl for<'b> FnMut(&'b P) + Send + 'static,
    ) -> Result<Self::Subscription, Self::Error> {
        (*self).subscribe(callback)
    }
}

pub trait PostboxProvider<P>: ErrorType {
    type Postbox: Postbox<P, Error = Self::Error>;

    fn postbox(&mut self) -> Result<Self::Postbox, Self::Error>;
}

impl<'a, P, PP> PostboxProvider<P> for &'a mut PP
where
    PP: PostboxProvider<P>,
{
    type Postbox = PP::Postbox;

    fn postbox(&mut self) -> Result<Self::Postbox, Self::Error> {
        (*self).postbox()
    }
}

pub trait PinnedEventBus<P>: ErrorType {
    type Subscription;

    fn subscribe(
        &mut self,
        callback: impl for<'a> FnMut(&'a P) + 'static,
    ) -> Result<Self::Subscription, Self::Error>;
}

impl<'a, P, E> PinnedEventBus<P> for &'a mut E
where
    E: PinnedEventBus<P>,
{
    type Subscription = E::Subscription;

    fn subscribe(
        &mut self,
        callback: impl for<'b> FnMut(&'b P) + 'static,
    ) -> Result<Self::Subscription, Self::Error> {
        (*self).subscribe(callback)
    }
}

#[cfg(all(feature = "nightly", feature = "experimental"))]
pub mod asynch {
    pub use super::{ErrorType, Spin};

    use core::future::Future;

    pub trait Sender {
        type Data: Send;

        type SendFuture<'a>: Future + Send
        where
            Self: 'a;

        fn send(&mut self, value: Self::Data) -> Self::SendFuture<'_>;
    }

    impl<S> Sender for &mut S
    where
        S: Sender,
    {
        type Data = S::Data;

        type SendFuture<'a>
        = S::SendFuture<'a> where Self: 'a;

        fn send(&mut self, value: Self::Data) -> Self::SendFuture<'_> {
            (*self).send(value)
        }
    }

    pub trait Receiver {
        type Data: Send;

        type RecvFuture<'a>: Future<Output = Self::Data> + Send
        where
            Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_>;
    }

    impl<R> Receiver for &mut R
    where
        R: Receiver,
    {
        type Data = R::Data;

        type RecvFuture<'a>
        = R::RecvFuture<'a> where Self: 'a;

        fn recv(&mut self) -> Self::RecvFuture<'_> {
            (*self).recv()
        }
    }

    pub trait EventBus<P>: ErrorType {
        type Subscription: Receiver<Data = P>;

        fn subscribe(&mut self) -> Result<Self::Subscription, Self::Error>;
    }

    pub trait PostboxProvider<P>: ErrorType {
        type Postbox: Sender<Data = P>;

        fn postbox(&mut self) -> Result<Self::Postbox, Self::Error>;
    }
}
