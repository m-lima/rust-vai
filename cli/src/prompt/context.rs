pub(super) struct Context<'a> {
    executors: &'a super::core::executors::Executor,
    buffer: super::buffer::Buffer,
    completer: super::completer::Completer,
}
