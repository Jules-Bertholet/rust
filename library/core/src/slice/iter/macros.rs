//! Macros used by iterators of slice.

// Inlining is_empty and len makes a huge performance difference
macro_rules! is_empty {
    // The way we encode the length of a ZST iterator, this works both for ZST
    // and non-ZST.
    ($self: ident) => {
        $self.ptr.as_ptr() as *const T == $self.end
    };
}

// To get rid of some bounds checks (see `position`), we compute the length in a somewhat
// unexpected way. (Tested by `codegen/slice-position-bounds-check`.)
macro_rules! len {
    ($self: ident) => {{
        #![allow(unused_unsafe)] // we're sometimes used within an unsafe block

        let start = $self.ptr;
        let size = size_from_ptr(start.as_ptr());
        if size == 0 {
            // This _cannot_ use `unchecked_sub` because we depend on wrapping
            // to represent the length of long ZST slice iterators.
            $self.end.addr().wrapping_sub(start.as_ptr().addr())
        } else {
            // We know that `start <= end`, so can do better than `offset_from`,
            // which needs to deal in signed.  By setting appropriate flags here
            // we can tell LLVM this, which helps it remove bounds checks.
            // SAFETY: By the type invariant, `start <= end`
            let diff = unsafe { unchecked_sub($self.end.addr(), start.as_ptr().addr()) };
            // By also telling LLVM that the pointers are apart by an exact
            // multiple of the type size, it can optimize `len() == 0` down to
            // `start == end` instead of `(end - start) < size`.
            // SAFETY: By the type invariant, the pointers are aligned so the
            //         distance between them must be a multiple of pointee size
            unsafe { exact_div(diff, size) }
        }
    }};
}

// The shared definition of the `Iter` and `IterMut` iterators
macro_rules! iterator {
    (
        struct $name:ident -> $ptr:ty,
        $elem:ty,
        $raw_mut:tt,
        {$( $mut_:tt )?},
        {$($extra:tt)*}
    ) => {
        // Returns the first element and moves the start of the iterator forwards by 1.
        // Greatly improves performance compared to an inlined function. The iterator
        // must not be empty.
        macro_rules! next_unchecked {
            ($self: ident) => {& $( $mut_ )? *$self.post_inc_start(1)}
        }

        // Returns the last element and moves the end of the iterator backwards by 1.
        // Greatly improves performance compared to an inlined function. The iterator
        // must not be empty.
        macro_rules! next_back_unchecked {
            ($self: ident) => {& $( $mut_ )? *$self.pre_dec_end(1)}
        }

        // Shrinks the iterator when T is a ZST, by moving the end of the iterator
        // backwards by `n`. `n` must not exceed `self.len()`.
        macro_rules! zst_shrink {
            ($self: ident, $n: ident) => {
                $self.end = ($self.end as * $raw_mut u8).wrapping_offset(-$n) as * $raw_mut T;
            }
        }

        impl<'a, T> $name<'a, T> {
            // Helper function for creating a slice from the iterator.
            #[inline(always)]
            fn make_slice(&self) -> &'a [T] {
                // SAFETY: the iterator was created from a slice with pointer
                // `self.ptr` and length `len!(self)`. This guarantees that all
                // the prerequisites for `from_raw_parts` are fulfilled.
                unsafe { from_raw_parts(self.ptr.as_ptr(), len!(self)) }
            }

            // Helper function for moving the start of the iterator forwards by `offset` elements,
            // returning the old start.
            // Unsafe because the offset must not exceed `self.len()`.
            #[inline(always)]
            unsafe fn post_inc_start(&mut self, offset: isize) -> * $raw_mut T {
                if mem::size_of::<T>() == 0 {
                    zst_shrink!(self, offset);
                    self.ptr.as_ptr()
                } else {
                    let old = self.ptr.as_ptr();
                    // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`,
                    // so this new pointer is inside `self` and thus guaranteed to be non-null.
                    self.ptr = unsafe { NonNull::new_unchecked(self.ptr.as_ptr().offset(offset)) };
                    old
                }
            }

            // Helper function for moving the end of the iterator backwards by `offset` elements,
            // returning the new end.
            // Unsafe because the offset must not exceed `self.len()`.
            #[inline(always)]
            unsafe fn pre_dec_end(&mut self, offset: isize) -> * $raw_mut T {
                if mem::size_of::<T>() == 0 {
                    zst_shrink!(self, offset);
                    self.ptr.as_ptr()
                } else {
                    // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`,
                    // which is guaranteed to not overflow an `isize`. Also, the resulting pointer
                    // is in bounds of `slice`, which fulfills the other requirements for `offset`.
                    self.end = unsafe { self.end.offset(-offset) };
                    self.end
                }
            }
        }

        #[stable(feature = "rust1", since = "1.0.0")]
        impl<T> ExactSizeIterator for $name<'_, T> {
            #[inline(always)]
            fn len(&self) -> usize {
                len!(self)
            }

            #[inline(always)]
            fn is_empty(&self) -> bool {
                is_empty!(self)
            }
        }

        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T> Iterator for $name<'a, T> {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks

                // SAFETY: `assume` calls are safe since a slice's start pointer
                // must be non-null, and slices over non-ZSTs must also have a
                // non-null end pointer. The call to `next_unchecked!` is safe
                // since we check if the iterator is empty first.
                unsafe {
                    assume(!self.ptr.as_ptr().is_null());
                    if mem::size_of::<T>() != 0 {
                        assume(!self.end.is_null());
                    }
                    if is_empty!(self) {
                        None
                    } else {
                        Some(next_unchecked!(self))
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let exact = len!(self);
                (exact, Some(exact))
            }

            #[inline]
            fn count(self) -> usize {
                len!(self)
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<$elem> {
                if n >= len!(self) {
                    // This iterator is now empty.
                    if mem::size_of::<T>() == 0 {
                        // We have to do it this way as `ptr` may never be 0, but `end`
                        // could be (due to wrapping).
                        self.end = self.ptr.as_ptr();
                    } else {
                        // SAFETY: end can't be 0 if T isn't ZST because ptr isn't 0 and end >= ptr
                        unsafe {
                            self.ptr = NonNull::new_unchecked(self.end as *mut T);
                        }
                    }
                    return None;
                }
                // SAFETY: We are in bounds. `post_inc_start` does the right thing even for ZSTs.
                unsafe {
                    self.post_inc_start(n as isize);
                    Some(next_unchecked!(self))
                }
            }

            #[inline]
            fn advance_by(&mut self, n: usize) -> Result<(), usize> {
                let advance = cmp::min(len!(self), n);
                // SAFETY: By construction, `advance` does not exceed `self.len()`.
                unsafe { self.post_inc_start(advance as isize) };
                if advance == n { Ok(()) } else { Err(advance) }
            }

            #[inline]
            fn last(mut self) -> Option<$elem> {
                self.next_back()
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile.
            #[inline]
            fn for_each<F>(mut self, mut f: F)
            where
                Self: Sized,
                F: FnMut(Self::Item),
            {
                while let Some(x) = self.next() {
                    f(x);
                }
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile.
            #[inline]
            fn all<F>(&mut self, mut f: F) -> bool
            where
                Self: Sized,
                F: FnMut(Self::Item) -> bool,
            {
                while let Some(x) = self.next() {
                    if !f(x) {
                        return false;
                    }
                }
                true
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile.
            #[inline]
            fn any<F>(&mut self, mut f: F) -> bool
            where
                Self: Sized,
                F: FnMut(Self::Item) -> bool,
            {
                while let Some(x) = self.next() {
                    if f(x) {
                        return true;
                    }
                }
                false
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile.
            #[inline]
            fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
            where
                Self: Sized,
                P: FnMut(&Self::Item) -> bool,
            {
                while let Some(x) = self.next() {
                    if predicate(&x) {
                        return Some(x);
                    }
                }
                None
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile.
            #[inline]
            fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
            where
                Self: Sized,
                F: FnMut(Self::Item) -> Option<B>,
            {
                while let Some(x) = self.next() {
                    if let Some(y) = f(x) {
                        return Some(y);
                    }
                }
                None
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile. Also, the `assume` avoids a bounds check.
            #[inline]
            #[rustc_inherit_overflow_checks]
            fn position<P>(&mut self, mut predicate: P) -> Option<usize> where
                Self: Sized,
                P: FnMut(Self::Item) -> bool,
            {
                let n = len!(self);
                let mut i = 0;
                while let Some(x) = self.next() {
                    if predicate(x) {
                        // SAFETY: we are guaranteed to be in bounds by the loop invariant:
                        // when `i >= n`, `self.next()` returns `None` and the loop breaks.
                        unsafe { assume(i < n) };
                        return Some(i);
                    }
                    i += 1;
                }
                None
            }

            // We override the default implementation, which uses `try_fold`,
            // because this simple implementation generates less LLVM IR and is
            // faster to compile. Also, the `assume` avoids a bounds check.
            #[inline]
            fn rposition<P>(&mut self, mut predicate: P) -> Option<usize> where
                P: FnMut(Self::Item) -> bool,
                Self: Sized + ExactSizeIterator + DoubleEndedIterator
            {
                let n = len!(self);
                let mut i = n;
                while let Some(x) = self.next_back() {
                    i -= 1;
                    if predicate(x) {
                        // SAFETY: `i` must be lower than `n` since it starts at `n`
                        // and is only decreasing.
                        unsafe { assume(i < n) };
                        return Some(i);
                    }
                }
                None
            }

            #[inline]
            unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> Self::Item {
                // SAFETY: the caller must guarantee that `i` is in bounds of
                // the underlying slice, so `i` cannot overflow an `isize`, and
                // the returned references is guaranteed to refer to an element
                // of the slice and thus guaranteed to be valid.
                //
                // Also note that the caller also guarantees that we're never
                // called with the same index again, and that no other methods
                // that will access this subslice are called, so it is valid
                // for the returned reference to be mutable in the case of
                // `IterMut`
                unsafe { & $( $mut_ )? * self.ptr.as_ptr().add(idx) }
            }

            $($extra)*
        }

        #[stable(feature = "rust1", since = "1.0.0")]
        impl<'a, T> DoubleEndedIterator for $name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks

                // SAFETY: `assume` calls are safe since a slice's start pointer must be non-null,
                // and slices over non-ZSTs must also have a non-null end pointer.
                // The call to `next_back_unchecked!` is safe since we check if the iterator is
                // empty first.
                unsafe {
                    assume(!self.ptr.as_ptr().is_null());
                    if mem::size_of::<T>() != 0 {
                        assume(!self.end.is_null());
                    }
                    if is_empty!(self) {
                        None
                    } else {
                        Some(next_back_unchecked!(self))
                    }
                }
            }

            #[inline]
            fn nth_back(&mut self, n: usize) -> Option<$elem> {
                if n >= len!(self) {
                    // This iterator is now empty.
                    self.end = self.ptr.as_ptr();
                    return None;
                }
                // SAFETY: We are in bounds. `pre_dec_end` does the right thing even for ZSTs.
                unsafe {
                    self.pre_dec_end(n as isize);
                    Some(next_back_unchecked!(self))
                }
            }

            #[inline]
            fn advance_back_by(&mut self, n: usize) -> Result<(), usize> {
                let advance = cmp::min(len!(self), n);
                // SAFETY: By construction, `advance` does not exceed `self.len()`.
                unsafe { self.pre_dec_end(advance as isize) };
                if advance == n { Ok(()) } else { Err(advance) }
            }
        }

        #[stable(feature = "fused", since = "1.26.0")]
        impl<T> FusedIterator for $name<'_, T> {}

        #[unstable(feature = "trusted_len", issue = "37572")]
        unsafe impl<T> TrustedLen for $name<'_, T> {}
    }
}

macro_rules! split_iter {
    (
        #[$stability:meta]
        #[debug($debug_stability:meta)]
        #[fused($fused_stability:meta)]
        $(#[$outer:meta])*
        struct $split_iter:ident<
            $(shared_ref: & $lt:lifetime)?
            $(mut_ref: & $m_lt:lifetime)?
        > {
            include_leading: $include_leading:literal,
            include_trailing: $include_trailing:literal,
        }
    ) => {
        $(#[$outer])*
        #[$stability]
        pub struct $split_iter<$($lt)? $($m_lt)?, T: $($lt)? $($m_lt)?, P>
        where
            P: FnMut(&T) -> bool,
        {
            // Used for `SplitWhitespace` and `SplitAsciiWhitespace` `as_str` methods
            pub(crate) v: &$($lt)?$($m_lt mut)? [T],
            pred: P,
            // Used for `SplitAsciiWhitespace` `as_str` method
            pub(crate) finished: bool,
        }

        impl<$($lt)?$($m_lt)?, T: $($lt)?$($m_lt)?, P: FnMut(&T) -> bool> $split_iter<$($lt)?$($m_lt)?, T, P> {
            #[inline]
            pub(super) fn new(slice: &$($lt)?$($m_lt mut)? [T], pred: P) -> Self {
                Self {
                    v: slice,
                    pred,
                    finished: false,
                }
            }
        }

        #[$debug_stability]
        impl<T: fmt::Debug, P> fmt::Debug for $split_iter<'_, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!(split_iter))
                    .field("v", &self.v)
                    .field("finished", &self.finished)
                    .finish()
            }
        }

        split_iter! {
            #[$stability]
            impl Clone for $split_iter<$(shared_ref: &$lt)? $(mut_ref: &$m_lt)?> {}
        }

        #[$stability]
        impl<$($lt)?$($m_lt)?, T, P> Iterator for $split_iter<$($lt)?$($m_lt)?, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            type Item = &$($lt)?$($m_lt mut)? [T];

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                if self.finished {
                    return None;
                }

                if $include_leading && self.v.is_empty() {
                    self.finished = true;
                    return None;
                }

                let offset = if $include_leading {
                    // The first index of self.v is already checked and found to match
                    // by the last iteration, so we start searching a new match
                    // one index to the right.
                    1
                } else {
                    0
                };

                let idx_opt = {
                    // work around borrowck limitations
                    let pred = &mut self.pred;
                    self.v[offset..].iter().position(|x| (*pred)(x)).map(|i| i + offset)
                };

                match idx_opt {
                    None => {
                        self.finished = true;
                        if $include_trailing && self.v.is_empty() {
                            return None;
                        }
                        $(let ret: & $lt [T] = self.v;)?
                        $(let ret: & $m_lt mut [T] = mem::replace(&mut self.v, &mut []);)?
                        Some(ret)
                    },
                    Some(idx) => {
                        // For shared ref iters
                        $(
                            let ret_end = if $include_trailing { idx + 1 } else { idx };
                            let ret: &$lt [T] = &self.v[..ret_end];
                            let v_start = if $include_leading { idx } else { idx + 1 };
                            self.v = &self.v[v_start..];
                            Some(ret)
                        )?

                        // For mut ref iters
                        $(
                            // Assert that include_leading and include_trailing are not both true
                            const _: [(); 0 - !{ const A: bool = !($include_leading && $include_trailing); A } as usize] = [];
                            let tmp: &$m_lt mut [T] = mem::replace(&mut self.v, &mut []);
                            let split_idx = if $include_trailing { idx + 1 } else { idx };
                            let (head, tail) = tmp.split_at_mut(split_idx);
                            let tail_start = if ($include_leading ^ $include_trailing) { 0 } else { 1 };
                            self.v = &mut tail[tail_start..];
                            Some(head)
                        )?
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                if self.finished {
                    (0, Some(0))
                } else {
                    // If the predicate doesn't match anything, we yield one slice
                    // for exclusive iterators, and zero for inclusive ones.
                    // If it matches every element, we yield `len() + 1` empty slices.
                    let min = if $include_leading || $include_trailing { 0 } else { 1 };
                    (min, Some(self.v.len() + min))
                }
            }
        }

        #[$stability]
        impl<$($lt)?$($m_lt)?, T, P> DoubleEndedIterator for $split_iter<$($lt)?$($m_lt)?, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            #[inline]
            fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
                if self.finished {
                    return None;
                }

                if $include_trailing && self.v.is_empty() {
                    self.finished = true;
                    return None;
                }

                let offset = if $include_trailing {
                    // The last index of self.v is already checked and found to match
                    // by the last iteration, so we start searching a new match
                    // one index to the left.
                    1
                } else {
                    0
                };

                let idx_opt = {
                    // work around borrowck limitations
                    let pred = &mut self.pred;
                    self.v[..(self.v.len() - offset)].iter().rposition(|x| (*pred)(x))
                };

                match idx_opt {
                    None => {
                        self.finished = true;
                        if $include_leading && self.v.is_empty() {
                            return None;
                        }
                        $(let ret: & $lt [T] = self.v;)?
                        $(let ret: & $m_lt mut [T] = mem::replace(&mut self.v, &mut []);)?
                        Some(ret)
                    },
                    Some(idx) => {
                        // For shared ref iters
                        $(
                            let ret_start = if $include_leading { idx } else { idx + 1 };
                            let ret: &$lt [T] = &self.v[ret_start..];
                            let v_end = if $include_trailing { idx + 1 } else { idx };
                            self.v = &self.v[..v_end];
                            Some(ret)
                        )?

                        // For mut ref iters
                        $(
                            // Assert that include_leading and include_trailing are not both true
                            const _: [(); 0 - !{ const A: bool = !($include_leading && $include_trailing); A } as usize] = [];
                            let tmp: &$m_lt mut [T] = mem::replace(&mut self.v, &mut []);
                            let split_idx = if $include_trailing { idx + 1 } else { idx };
                            let (head, tail) = tmp.split_at_mut(split_idx);
                            let tail_start = if ($include_leading ^ $include_trailing) { 0 } else { 1 };
                            self.v = head;
                            let ret = &mut tail[tail_start..];
                            Some(ret)
                        )?
                    }
                }
            }
        }

        impl<$($lt)? $($m_lt)?, T, P> SplitIter for $split_iter<$($lt)? $($m_lt)?, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            #[inline]
            fn finish(&mut self) -> Option<<Self as Iterator>::Item> {
                if self.finished {
                    None
                } else {
                    self.finished = true;
                    $(let ret: & $lt [T] = self.v;)?
                    $(let ret: & $m_lt mut [T] = mem::replace(&mut self.v, &mut []);)?
                    if ($include_leading || $include_trailing) && ret.is_empty() {
                        None
                    } else {
                        Some(ret)
                    }
                }
            }
        }

        #[$fused_stability]
        impl<T, P> FusedIterator for $split_iter<'_, T, P> where P: FnMut(&T) -> bool {}
    };

    (
        #[$stability:meta]
        impl Clone for $split_iter:ident<shared_ref: & $lt:lifetime> {}
    ) => {
        // FIXME(#26925) Remove in favor of `#[derive(Clone)]`
        #[$stability]
        impl<$lt, T, P> Clone for $split_iter<$lt, T, P>
        where
            P: Clone + FnMut(&T) -> bool,
        {
            fn clone(&self) -> Self {
                Self {
                    v: self.v,
                    pred: self.pred.clone(),
                    finished: self.finished
                }
            }
        }
    };

    (
        #[$stability:meta]
        impl Clone for $split_iter:ident<mut_ref: & $m_lt:lifetime> {}
    ) => {};
}

macro_rules! reverse_iter {
    (
        #[$stability:meta]
        $(#[$outer:meta])*
        $vis:vis struct $rev:ident { inner: $inner:ident } $(: $clone:ident)?
    ) => {
        $(#[$outer])*
        #[$stability]
        $vis struct $rev<'a, T: 'a, P>
        where
            P: FnMut(&T) -> bool,
        {
            inner: $inner<'a, T, P>,
        }

        impl<'a, T: 'a, P: FnMut(&T) -> bool> $rev<'a, T, P> {
            #[inline]
            pub(super) fn new(slice: <$inner<'a, T, P> as Iterator>::Item, pred: P) -> Self {
                Self { inner: $inner::new(slice, pred) }
            }
        }

        #[$stability]
        impl<T: fmt::Debug, P> fmt::Debug for $rev<'_, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($rev))
                    .field("v", &self.inner.v)
                    .field("finished", &self.inner.finished)
                    .finish()
            }
        }

        $(
        // FIXME(#26925) Remove in favor of `#[derive(Clone)]`
        #[$stability]
        impl<'a, T, P> $clone for $rev<'a, T, P>
        where
            P: Clone + FnMut(&T) -> bool,
        {
            fn clone(&self) -> Self {
                Self { inner: self.inner.clone() }
            }
        }
        )?

        #[$stability]
        impl<'a, T, P> Iterator for $rev<'a, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            type Item = <$inner<'a, T, P> as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next_back()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        #[$stability]
        impl<'a, T, P> DoubleEndedIterator for $rev<'a, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.inner.next()
            }
        }

        #[$stability]
        impl<T, P> FusedIterator for $rev<'_, T, P> where P: FnMut(&T) -> bool {}

        #[$stability]
        impl<'a, T, P> SplitIter for $rev<'a, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            #[inline]
            fn finish(&mut self) -> Option<Self::Item> {
                self.inner.finish()
            }
        }
    };
}

#[allow(unused)]
macro_rules! iter_n {
    (
        #[$stability:meta]
        #[fused($fused_stability:meta)]
        $(#[$outer:meta])*
        $vis:vis struct $iter_n:ident { inner: $inner:ident } $(: $clone:ident)?

        $(#[$max_items_attrs:meta])*
        fn max_items;
    ) => {
        $(#[$outer])*
        #[$stability]
        pub struct $iter_n<'a, T: 'a, P>
        where
            P: FnMut(&T) -> bool,
        {
            inner: GenericSplitN<$inner<'a, T, P>>,
        }

        impl<'a, T: 'a, P: FnMut(&T) -> bool> $iter_n<'a, T, P> {
            #[inline]
            pub(super) fn new(s: $inner<'a, T, P>, n: usize) -> Self {
                Self { inner: GenericSplitN { iter: s, count: n } }
            }
        }

        $(
        #[$stability]
        impl<'a, T: 'a, P> $clone for $iter_n<'a, T, P>
        where
            P: Clone + FnMut(&T) -> bool,
        {
            fn clone(&self) -> Self {
                Self { inner: self.inner.clone() }
            }
        }
        )?

        #[$stability]
        impl<T: fmt::Debug, P> fmt::Debug for $iter_n<'_, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($iter_n)).field("inner", &self.inner).finish()
            }
        }

        #[$stability]
        impl<'a, T, P> Iterator for $iter_n<'a, T, P>
        where
            P: FnMut(&T) -> bool,
        {
            type Item = <$inner<'a, T, P> as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        #[$fused_stability]
        impl<'a, T, P> FusedIterator for $iter_n<'a, T, P> where P: FnMut(&T) -> bool {}

        impl<'a, T, P> $inner<'a, T, P> where P: FnMut(&T) -> bool {
            $(#[$max_items_attrs])*
            #[inline]
            pub fn max_items(self, n: usize) -> $iter_n<'a, T, P> {
                $iter_n::new(self, n)
            }
        }
    };
}
