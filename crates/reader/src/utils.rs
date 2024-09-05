use std::mem::MaybeUninit;

/// Create a new [`Vec`] with uninitialized elements.
///
/// This is internally unsafe as the elements of the vector are not initialized; the
/// resultant [`Vec`] should only be used as buffers.
pub fn new_buffer<T>(size: impl Into<usize>) -> Vec<T> {
    let size = size.into();
    // Allocate memory for the vector with the specified capacity
    let mut vec: Vec<MaybeUninit<T>> = Vec::with_capacity(size);

    unsafe {
        // Set the length of the Vec without initializing its elements
        vec.set_len(size);

        // Transmute Vec<MaybeUninit<T>> to Vec<T> since we've promised to handle initialization ourselves
        std::mem::transmute(vec)
    }
}
