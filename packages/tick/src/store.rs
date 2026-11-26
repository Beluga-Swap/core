use soroban_sdk::Env;

use crate::TickInfo;

/// The pool's tick storage, as seen by liquidity updates and the swap engine.
pub trait TickStore {
    fn read_tick(&self, env: &Env, tick: i32) -> TickInfo;
    fn write_tick(&self, env: &Env, tick: i32, info: &TickInfo);
    /// Toggle a tick in the initialized-tick bitmap.
    fn flip_tick(&self, env: &Env, tick: i32);
    /// Next initialized tick in the swap direction; see
    /// [`crate::next_initialized_tick`].
    fn next_initialized_tick(&self, env: &Env, tick: i32, zero_for_one: bool) -> Option<i32>;
}

/// Reads through to the wrapped store and discards every write. Quotes run the
/// real swap loop against it.
pub struct ReadOnly<'a, S>(pub &'a S);

impl<S: TickStore> TickStore for ReadOnly<'_, S> {
    fn read_tick(&self, env: &Env, tick: i32) -> TickInfo {
        self.0.read_tick(env, tick)
    }
    fn write_tick(&self, _: &Env, _: i32, _: &TickInfo) {}
    fn flip_tick(&self, _: &Env, _: i32) {}
    fn next_initialized_tick(&self, env: &Env, tick: i32, zero_for_one: bool) -> Option<i32> {
        self.0.next_initialized_tick(env, tick, zero_for_one)
    }
}
