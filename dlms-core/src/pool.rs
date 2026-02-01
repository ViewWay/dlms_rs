//! Memory pool and buffer management for performance optimization
//!
//! This module provides reusable memory pools and buffers to reduce
//! allocation overhead in high-throughput scenarios.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Configuration for buffer pool
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Size of each buffer in bytes
    pub buffer_size: usize,
    /// Number of buffers to pre-allocate
    pub initial_capacity: usize,
    /// Maximum number of buffers (0 = unlimited)
    pub max_capacity: usize,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,  // 4KB default
            initial_capacity: 10,
            max_capacity: 100,
        }
    }
}

impl BufferPoolConfig {
    /// Create a new buffer pool configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the initial capacity
    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        self.initial_capacity = capacity;
        self
    }

    /// Set the maximum capacity
    pub fn with_max_capacity(mut self, capacity: usize) -> Self {
        self.max_capacity = capacity;
        self
    }
}

/// A reusable buffer from the pool
///
/// When dropped, the buffer is returned to the pool for reuse.
pub struct PooledBuffer {
    /// The buffer data
    pub data: Vec<u8>,
    /// Pool to return to when dropped
    pool: Option<Arc<Mutex<VecDeque<Vec<u8>>>>>,
    /// Maximum capacity of the pool
    max_capacity: usize,
}

impl PooledBuffer {
    /// Create a new pooled buffer
    fn new(mut data: Vec<u8>, pool: Arc<Mutex<VecDeque<Vec<u8>>>>, max_capacity: usize) -> Self {
        // Clear the buffer data
        data.clear();
        Self {
            data,
            pool: Some(pool),
            max_capacity,
        }
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Extend the buffer with additional data
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    /// Resize the buffer
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.data.resize(new_len, value);
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get a slice of the buffer
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable slice of the buffer
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Convert to a Vec<u8> without returning to pool
    pub fn into_vec(mut self) -> Vec<u8> {
        self.pool = None; // Don't return to pool
        std::mem::take(&mut self.data)
    }

    /// Reserve additional capacity in the buffer
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }
}

impl AsRef<[u8]> for PooledBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for PooledBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            let mut pool = pool.lock().unwrap();
            // Only return to pool if under max capacity
            if pool.len() < self.max_capacity || self.max_capacity == 0 {
                pool.push_back(std::mem::take(&mut self.data));
            }
        }
    }
}

/// Buffer pool for reusable memory allocation
///
/// Reduces allocation overhead by maintaining a pool of pre-allocated
/// buffers that can be reused across multiple operations.
#[derive(Clone)]
pub struct BufferPool {
    /// Available buffers in the pool
    buffers: Arc<Mutex<VecDeque<Vec<u8>>>>,
    /// Size of each buffer
    buffer_size: usize,
    /// Maximum number of buffers
    max_capacity: usize,
}

impl BufferPool {
    /// Create a new buffer pool with default configuration
    pub fn new() -> Self {
        Self::with_config(BufferPoolConfig::default())
    }

    /// Create a new buffer pool with custom configuration
    pub fn with_config(config: BufferPoolConfig) -> Self {
        let mut pool = VecDeque::with_capacity(config.initial_capacity);

        // Pre-allocate buffers
        for _ in 0..config.initial_capacity {
            pool.push_back(vec![0u8; config.buffer_size]);
        }

        Self {
            buffers: Arc::new(Mutex::new(pool)),
            buffer_size: config.buffer_size,
            max_capacity: config.max_capacity,
        }
    }

    /// Acquire a buffer from the pool
    ///
    /// If the pool is empty, a new buffer is allocated (up to max_capacity).
    pub fn acquire(&self) -> PooledBuffer {
        let mut pool = self.buffers.lock().unwrap();

        let buffer = pool.pop_front()
            .unwrap_or_else(|| vec![0u8; self.buffer_size]);

        PooledBuffer::new(buffer, self.buffers.clone(), self.max_capacity)
    }

    /// Get the current number of available buffers
    pub fn available_count(&self) -> usize {
        self.buffers.lock().unwrap().len()
    }

    /// Pre-allocate additional buffers
    pub fn expand(&self, additional: usize) {
        let mut pool = self.buffers.lock().unwrap();
        let current_len = pool.len();

        if self.max_capacity > 0 && current_len + additional > self.max_capacity {
            return; // Would exceed max capacity
        }

        for _ in 0..additional {
            if self.max_capacity > 0 && pool.len() >= self.max_capacity {
                break;
            }
            pool.push_back(vec![0u8; self.buffer_size]);
        }
    }

    /// Shrink the pool by removing unused buffers
    pub fn shrink(&self, target_size: usize) {
        let mut pool = self.buffers.lock().unwrap();
        while pool.len() > target_size {
            pool.pop_back();
        }
    }

    /// Clear all buffers from the pool
    pub fn clear(&self) {
        let mut pool = self.buffers.lock().unwrap();
        pool.clear();
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        let available = self.available_count();
        BufferPoolStats {
            buffer_size: self.buffer_size,
            available_buffers: available,
            max_capacity: if self.max_capacity == 0 {
                None
            } else {
                Some(self.max_capacity)
            },
        }
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    /// Size of each buffer
    pub buffer_size: usize,
    /// Number of available buffers
    pub available_buffers: usize,
    /// Maximum capacity (None = unlimited)
    pub max_capacity: Option<usize>,
}

/// Zero-copy byte slice wrapper
///
/// Provides a zero-copy view into a byte slice without allocating.
#[derive(Debug, Clone, Copy)]
pub struct ByteSlice<'a> {
    data: &'a [u8],
}

impl<'a> ByteSlice<'a> {
    /// Create a new byte slice
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Get the underlying slice
    pub fn as_slice(&self) -> &'a [u8] {
        self.data
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Split the slice at the given index
    pub fn split_at(&self, mid: usize) -> (ByteSlice<'a>, ByteSlice<'a>) {
        let (first, second) = self.data.split_at(mid);
        (ByteSlice::new(first), ByteSlice::new(second))
    }

    /// Take the first n bytes
    pub fn take(&self, n: usize) -> ByteSlice<'a> {
        ByteSlice::new(&self.data[..n.min(self.data.len())])
    }

    /// Skip the first n bytes
    pub fn skip(&self, n: usize) -> ByteSlice<'a> {
        let start = n.min(self.data.len());
        ByteSlice::new(&self.data[start..])
    }
}

impl<'a> AsRef<[u8]> for ByteSlice<'a> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'a> From<&'a [u8]> for ByteSlice<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self::new(data)
    }
}

impl<'a> From<&'a Vec<u8>> for ByteSlice<'a> {
    fn from(data: &'a Vec<u8>) -> Self {
        Self::new(data.as_slice())
    }
}

/// Lazy evaluation wrapper for expensive computations
///
/// Defers computation until the value is actually needed.
#[derive(Debug)]
pub struct Lazy<T, F>
where
    F: FnOnce() -> T,
{
    value: Option<T>,
    init: Option<F>,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    /// Create a new lazy value
    pub fn new(init: F) -> Self {
        Self {
            value: None,
            init: Some(init),
        }
    }

    /// Get the value, computing it if necessary
    pub fn get(&mut self) -> &T {
        if self.value.is_none() {
            let init = self.init.take().unwrap();
            self.value = Some(init());
        }
        self.value.as_ref().unwrap()
    }

    /// Get the value, computing it if necessary (mutable)
    pub fn get_mut(&mut self) -> &mut T {
        if self.value.is_none() {
            let init = self.init.take().unwrap();
            self.value = Some(init());
        }
        self.value.as_mut().unwrap()
    }

    /// Check if the value has been computed
    pub fn is_computed(&self) -> bool {
        self.value.is_some()
    }

    /// Force computation and return the value
    pub fn force(mut self) -> T {
        if self.value.is_none() {
            let init = self.init.take().unwrap();
            self.value = Some(init());
        }
        self.value.unwrap()
    }
}

/// Create a lazy value
pub fn lazy<T, F>(init: F) -> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    Lazy::new(init)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_acquire_release() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            buffer_size: 1024,
            initial_capacity: 2,
            max_capacity: 10,
        });

        assert_eq!(pool.available_count(), 2);

        // Acquire and release buffers
        {
            let _buf1 = pool.acquire();
            assert_eq!(pool.available_count(), 1);

            let _buf2 = pool.acquire();
            assert_eq!(pool.available_count(), 0);

            // These will be returned to pool when dropped
        }

        // After dropping, buffers should be returned
        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn test_buffer_pool_stats() {
        let pool = BufferPool::new();
        let stats = pool.stats();

        assert_eq!(stats.available_buffers, 10); // default initial_capacity
        assert_eq!(stats.buffer_size, 4096); // default buffer_size
        assert_eq!(stats.max_capacity, Some(100)); // default max_capacity
    }

    #[test]
    fn test_buffer_pool_expand() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            buffer_size: 512,
            initial_capacity: 2,
            max_capacity: 10,
        });

        let initial_count = pool.available_count();
        assert_eq!(initial_count, 2);

        pool.expand(3);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_buffer_pool_shrink() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            buffer_size: 512,
            initial_capacity: 10,
            max_capacity: 10,
        });

        pool.shrink(5);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_byte_slice() {
        let data = b"hello world";
        let slice = ByteSlice::new(data);

        assert_eq!(slice.len(), 11);
        assert!(!slice.is_empty());
        assert_eq!(slice.as_slice(), data);
    }

    #[test]
    fn test_byte_slice_split() {
        let data = b"hello world";
        let slice = ByteSlice::new(data);

        let (first, second) = slice.split_at(5);
        assert_eq!(first.as_slice(), b"hello");
        assert_eq!(second.as_slice(), b" world");
    }

    #[test]
    fn test_byte_slice_take_skip() {
        let data = b"hello world";
        let slice = ByteSlice::new(data);

        assert_eq!(slice.take(5).as_slice(), b"hello");
        assert_eq!(slice.skip(6).as_slice(), b"world");
    }

    #[test]
    fn test_lazy_evaluation() {
        let mut lazy_val = lazy(|| {
            42
        });

        assert!(!lazy_val.is_computed());

        assert_eq!(*lazy_val.get(), 42);
        assert!(lazy_val.is_computed());

        // Should not recompute
        assert_eq!(*lazy_val.get(), 42);
    }

    #[test]
    fn test_lazy_force() {
        let lazy_val = lazy(|| {
            "computed"
        });

        let result = lazy_val.force();
        assert_eq!(result, "computed");
    }

    #[test]
    fn test_pooled_buffer_operations() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            buffer_size: 100,
            initial_capacity: 1,
            max_capacity: 10,
        });

        let mut buf = pool.acquire();
        assert!(buf.is_empty());

        buf.extend_from_slice(b"hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.as_slice(), b"hello");

        buf.resize(10, 0xFF);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.as_slice()[5], 0xFF);
    }

    #[test]
    fn test_pooled_buffer_into_vec() {
        let pool = BufferPool::with_config(BufferPoolConfig {
            buffer_size: 100,
            initial_capacity: 1,
            max_capacity: 10,
        });

        let mut buf = pool.acquire();
        buf.extend_from_slice(b"test data");

        // Convert to Vec without returning to pool
        let vec = buf.into_vec();
        assert_eq!(vec, b"test data");

        // Buffer should not be returned to pool
        assert_eq!(pool.available_count(), 0);
    }
}
