use crate::state::State;
pub const ABI_VERSION: u32 = 1;
#[unsafe(no_mangle)]
pub extern "C" fn ascii_abi_version() -> u32 {
    ABI_VERSION
}
/// Advance the experimental hot-reload counter.
///
/// # Safety
/// `state` must point to a live, exclusively borrowed ABI-v1 State.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmain(state: *mut State) {
    if let Some(state) = unsafe { state.as_mut() } {
        state.counter = state.counter.wrapping_add(1);
    }
}
