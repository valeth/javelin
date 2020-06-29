use tokio::sync::oneshot;


pub type Responder<P> = oneshot::Sender<P>;
