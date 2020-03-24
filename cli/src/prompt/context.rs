pub(super) struct Context<S, C>
where
    S: Fn(&str) -> Option<String>,
    C: Fn(&str) -> Vec<String>,
{
    buffer: super::buffer::Buffer,
    suggester: super::suggester::Suggester<S>,
    completer: super::completer::Completer<C>,
}

pub(super) fn new<S, C>(suggester: S, completer: C) -> Context<S, C>
    where
        S: Fn(&str) -> Option<String>,
        C: Fn(&str) -> Vec<String>,
{
    Context {

    }
}
