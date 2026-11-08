//! Bounded message bridge over the official svcOpen/svcPoll/svcSend operations.
//! Hardware or boot actions are handled by the caller after script execution.
use crate::Error;
use alloc::{collections::VecDeque, rc::Rc, vec::Vec};
use core::cell::RefCell;
use pocket_runtime::{OpArg, OpProvider, OpResult, RuntimeError};

const MAX_MESSAGES: usize = 16;
const MAX_BYTES: usize = 4096;
#[derive(Default)]
pub(crate) struct Mailbox {
    incoming: VecDeque<Vec<u8>>,
    outgoing: VecDeque<Vec<u8>>,
}
pub(crate) type Shared = Rc<RefCell<Mailbox>>;
pub(crate) fn new() -> Shared {
    Rc::new(RefCell::new(Mailbox::default()))
}
pub(crate) fn send(mailbox: &Shared, message: &str) -> Result<(), Error> {
    let mut q = mailbox.borrow_mut();
    if message.len() > MAX_BYTES || q.incoming.len() >= MAX_MESSAGES {
        return Err(Error::QueueFull);
    }
    q.incoming.push_back(message.as_bytes().to_vec());
    Ok(())
}
pub(crate) fn receive(mailbox: &Shared) -> Option<Vec<u8>> {
    mailbox.borrow_mut().outgoing.pop_front()
}
pub(crate) struct Provider(pub Shared);
impl OpProvider for Provider {
    fn supports_op(&self, op: u16) -> bool {
        (30..=32).contains(&op)
    }
    fn dispatch(
        &mut self,
        _: &mut pocketjs_core::Ui,
        op: u16,
        args: &[OpArg<'_>],
    ) -> Result<OpResult, RuntimeError> {
        let mut q = self.0.borrow_mut();
        match op {
            30 => Ok(OpResult::Integer(1)),
            31 => Ok(q
                .incoming
                .pop_front()
                .map_or(OpResult::None, OpResult::Bytes)),
            32 => {
                let Some(OpArg::Bytes(bytes)) = args.first() else {
                    return Err(RuntimeError::InvalidArgument);
                };
                if bytes.len() > MAX_BYTES || q.outgoing.len() >= MAX_MESSAGES {
                    return Err(RuntimeError::InvalidArgument);
                }
                q.outgoing.push_back(bytes.to_vec());
                Ok(OpResult::None)
            }
            _ => Err(RuntimeError::UnsupportedOperation),
        }
    }
}
