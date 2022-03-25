//! Iterators for `str` methods.

use crate::char;
use crate::fmt::{self, Write};
use crate::iter::{Chain, FlatMap, Flatten};
use crate::iter::{Copied, Filter, FusedIterator, Map, TrustedLen};
use crate::iter::{TrustedRandomAccess, TrustedRandomAccessNoCoerce};
use crate::ops::Try;
use crate::option;
use crate::slice::{self, Split as SliceSplit};

use super::from_utf8_unchecked;
use super::pattern::Pattern;
use super::pattern::{DoubleEndedSearcher, ReverseSearcher, Searcher};
use super::validations::{next_code_point, next_code_point_reverse};
use super::LinesAnyMap;
use super::{BytesIsNotEmpty, UnsafeBytesToStr};
use super::{CharEscapeDebugContinue, CharEscapeDefault, CharEscapeUnicode};
use super::{IsAsciiWhitespace, IsNotEmpty, IsWhitespace};

/// An iterator over the [`char`]s of a string slice.
///
///
/// This struct is created by the [`chars`] method on [`str`].
/// See its documentation for more.
///
/// [`char`]: prim@char
/// [`chars`]: str::chars
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct Chars<'a> {
    pub(super) iter: slice::Iter<'a, u8>,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Chars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        // SAFETY: `str` invariant says `self.iter` is a valid UTF-8 string and
        // the resulting `ch` is a valid Unicode Scalar Value.
        unsafe { next_code_point(&mut self.iter).map(|ch| char::from_u32_unchecked(ch)) }
    }

    #[inline]
    fn count(self) -> usize {
        super::count::count_chars(self.as_str())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.iter.len();
        // `(len + 3)` can't overflow, because we know that the `slice::Iter`
        // belongs to a slice in memory which has a maximum length of
        // `isize::MAX` (that's well below `usize::MAX`).
        ((len + 3) / 4, Some(len))
    }

    #[inline]
    fn last(mut self) -> Option<char> {
        // No need to go through the entire string.
        self.next_back()
    }
}

#[stable(feature = "chars_debug_impl", since = "1.38.0")]
impl fmt::Debug for Chars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chars(")?;
        f.debug_list().entries(self.clone()).finish()?;
        write!(f, ")")?;
        Ok(())
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for Chars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<char> {
        // SAFETY: `str` invariant says `self.iter` is a valid UTF-8 string and
        // the resulting `ch` is a valid Unicode Scalar Value.
        unsafe { next_code_point_reverse(&mut self.iter).map(|ch| char::from_u32_unchecked(ch)) }
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for Chars<'_> {}

impl<'a> Chars<'a> {
    /// Views the underlying data as a subslice of the original data.
    ///
    /// This has the same lifetime as the original slice, and so the
    /// iterator can continue to be used while this exists.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut chars = "abc".chars();
    ///
    /// assert_eq!(chars.as_str(), "abc");
    /// chars.next();
    /// assert_eq!(chars.as_str(), "bc");
    /// chars.next();
    /// chars.next();
    /// assert_eq!(chars.as_str(), "");
    /// ```
    #[stable(feature = "iter_to_slice", since = "1.4.0")]
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &'a str {
        // SAFETY: `Chars` is only made from a str, which guarantees the iter is valid UTF-8.
        unsafe { from_utf8_unchecked(self.iter.as_slice()) }
    }
}

/// An iterator over the [`char`]s of a string slice, and their positions.
///
/// This struct is created by the [`char_indices`] method on [`str`].
/// See its documentation for more.
///
/// [`char`]: prim@char
/// [`char_indices`]: str::char_indices
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
pub struct CharIndices<'a> {
    pub(super) front_offset: usize,
    pub(super) iter: Chars<'a>,
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<(usize, char)> {
        let pre_len = self.iter.iter.len();
        match self.iter.next() {
            None => None,
            Some(ch) => {
                let index = self.front_offset;
                let len = self.iter.iter.len();
                self.front_offset += pre_len - len;
                Some((index, ch))
            }
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<(usize, char)> {
        // No need to go through the entire string.
        self.next_back()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for CharIndices<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<(usize, char)> {
        self.iter.next_back().map(|ch| {
            let index = self.front_offset + self.iter.iter.len();
            (index, ch)
        })
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for CharIndices<'_> {}

impl<'a> CharIndices<'a> {
    /// Views the underlying data as a subslice of the original data.
    ///
    /// This has the same lifetime as the original slice, and so the
    /// iterator can continue to be used while this exists.
    #[stable(feature = "iter_to_slice", since = "1.4.0")]
    #[must_use]
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.iter.as_str()
    }

    /// Returns the byte position of the next character, or the length
    /// of the underlying string if there are no more characters.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(char_indices_offset)]
    /// let mut chars = "a楽".char_indices();
    ///
    /// assert_eq!(chars.offset(), 0);
    /// assert_eq!(chars.next(), Some((0, 'a')));
    ///
    /// assert_eq!(chars.offset(), 1);
    /// assert_eq!(chars.next(), Some((1, '楽')));
    ///
    /// assert_eq!(chars.offset(), 4);
    /// assert_eq!(chars.next(), None);
    /// ```
    #[inline]
    #[must_use]
    #[unstable(feature = "char_indices_offset", issue = "83871")]
    pub fn offset(&self) -> usize {
        self.front_offset
    }
}

/// An iterator over the bytes of a string slice.
///
/// This struct is created by the [`bytes`] method on [`str`].
/// See its documentation for more.
///
/// [`bytes`]: str::bytes
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[stable(feature = "rust1", since = "1.0.0")]
#[derive(Clone, Debug)]
pub struct Bytes<'a>(pub(super) Copied<slice::Iter<'a, u8>>);

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for Bytes<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<u8> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        self.0.all(f)
    }

    #[inline]
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        self.0.any(f)
    }

    #[inline]
    fn find<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        self.0.find(predicate)
    }

    #[inline]
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        self.0.position(predicate)
    }

    #[inline]
    fn rposition<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
    {
        self.0.rposition(predicate)
    }

    #[inline]
    unsafe fn __iterator_get_unchecked(&mut self, idx: usize) -> u8 {
        // SAFETY: the caller must uphold the safety contract
        // for `Iterator::__iterator_get_unchecked`.
        unsafe { self.0.__iterator_get_unchecked(idx) }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl DoubleEndedIterator for Bytes<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<u8> {
        self.0.next_back()
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }

    #[inline]
    fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        self.0.rfind(predicate)
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl ExactSizeIterator for Bytes<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for Bytes<'_> {}

#[unstable(feature = "trusted_len", issue = "37572")]
unsafe impl TrustedLen for Bytes<'_> {}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl TrustedRandomAccess for Bytes<'_> {}

#[doc(hidden)]
#[unstable(feature = "trusted_random_access", issue = "none")]
unsafe impl TrustedRandomAccessNoCoerce for Bytes<'_> {
    const MAY_HAVE_SIDE_EFFECT: bool = false;
}

/// This macro generates a Clone impl for string pattern API
/// wrapper types of the form X<'a, P>
macro_rules! derive_pattern_clone {
    (
        $(#[$impl_attr:meta])*
        clone $t:ident
        $(where Searcher: ($where_clause:path))?
        with $(#[$fn_attr:meta])* |$s:ident| $e:expr
    ) => {
        $(#[$impl_attr])*
        impl<'a, P> Clone for $t<'a, P>
        where
            P: Pattern<'a, Searcher: $($where_clause +)? Clone>,
        {
            $(#[$fn_attr])*
            #[inline]
            fn clone(&self) -> Self {
                let $s = self;
                $e
            }
        }
    };
}

/// This macro generates two public iterator structs
/// wrapping a private internal one that makes use of the `Pattern` API.
///
/// For all patterns `P: Pattern<'a>` the following items will be
/// generated (generics omitted):
///
/// struct $forward_iterator($internal_iterator);
/// struct $reverse_iterator($internal_iterator);
///
/// impl Iterator for $forward_iterator
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// impl DoubleEndedIterator for $forward_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl Iterator for $reverse_iterator
///       where P::Searcher: ReverseSearcher
/// { /* internal ends up calling Searcher::next_match_back() */ }
///
/// impl DoubleEndedIterator for $reverse_iterator
///       where P::Searcher: DoubleEndedSearcher
/// { /* internal ends up calling Searcher::next_match() */ }
///
/// The internal one is defined outside the macro, and has almost the same
/// semantic as a DoubleEndedIterator by delegating to `pattern::Searcher` and
/// `pattern::ReverseSearcher` for both forward and reverse iteration.
///
/// "Almost", because a `Searcher` and a `ReverseSearcher` for a given
/// `Pattern` might not return the same elements, so actually implementing
/// `DoubleEndedIterator` for it would be incorrect.
/// (See the docs in `str::pattern` for more details)
///
/// However, the internal struct still represents a single ended iterator from
/// either end, and depending on pattern is also a valid double ended iterator,
/// so the two wrapper structs implement `Iterator`
/// and `DoubleEndedIterator` depending on the concrete pattern type, leading
/// to the complex impls seen above.
///
/// In addition, when requested, as_str methods are are also generated for all iterators.
macro_rules! generate_pattern_iterators {
    {
        // Forward iterator
        forward:
            #[$forward_stability_attribute:meta]
            #[fused($forward_fused_stability_attribute:meta)]
            $(#[$forward_iterator_attribute:meta])*
            struct $forward_iterator:ident;
            $($(#[$forward_as_str_attribute:meta])*
            fn as_str;)?

        // Reverse iterator
        reverse:
            #[$reverse_stability_attribute:meta]
            #[fused($reverse_fused_stability_attribute:meta)]
            $(#[$reverse_iterator_attribute:meta])*
            struct $reverse_iterator:ident;
            $($(#[$reverse_as_str_attribute:meta])*
            fn as_str;)?

        // Internal almost-iterator that is being delegated to
        internal:
            $internal_iterator:ident yielding ($iterty:ty);

        // Kind of delegation - either single ended or double ended
        delegate $($t:tt)*
    } => {
        $(#[$forward_iterator_attribute])*
        #[$forward_stability_attribute]
        #[repr(transparent)]
        pub struct $forward_iterator<'a, P: Pattern<'a>>(pub(super) $internal_iterator<'a, P>);

        derive_pattern_clone! {
            #[$forward_stability_attribute]
            clone $forward_iterator with |s| Self(s.0.clone())
        }

        #[$forward_stability_attribute]
        impl<'a, P> fmt::Debug for $forward_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($forward_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        #[$forward_stability_attribute]
        impl<'a, P: Pattern<'a>> Iterator for $forward_iterator<'a, P> {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }

        $(impl<'a, P: Pattern<'a>> $forward_iterator<'a, P> {
            $(#[$forward_as_str_attribute])*
            #[inline]
            pub fn as_str(&self) -> &'a str {
                self.0.as_str()
            }
        })?

        #[$forward_fused_stability_attribute]
        impl<'a, P: Pattern<'a>> FusedIterator for $forward_iterator<'a, P> {}

        $(#[$reverse_iterator_attribute])*
        #[$reverse_stability_attribute]
        #[repr(transparent)]
        pub struct $reverse_iterator<'a, P>(pub(super) $internal_iterator<'a, P>) where P: Pattern<'a, Searcher: ReverseSearcher<'a>>;

        derive_pattern_clone! {
            #[$reverse_stability_attribute]
            clone $reverse_iterator where Searcher: (ReverseSearcher<'a>) with |s| Self(s.0.clone())
        }

        #[$reverse_stability_attribute]
        impl<'a, P> fmt::Debug for $reverse_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a> + fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($reverse_iterator))
                    .field(&self.0)
                    .finish()
            }
        }

        #[$reverse_stability_attribute]
        impl<'a, P> Iterator for $reverse_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            type Item = $iterty;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.0.next_back()
            }
        }

        $(impl<'a, P> $reverse_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            $(#[$reverse_as_str_attribute])*
            #[inline]
            pub fn as_str(&self) -> &'a str {
                self.0.as_str()
            }
        })?

        #[$reverse_fused_stability_attribute]
        impl<'a, P> FusedIterator for $reverse_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {}

        generate_pattern_iterators!($($t)* with
            #[$forward_stability_attribute]
            $forward_iterator,
            #[$reverse_stability_attribute]
            $reverse_iterator,
            yielding $iterty
        );
    };
    {
        double ended; with
            #[$forward_stability_attribute:meta]
            $forward_iterator:ident,
            #[$reverse_stability_attribute:meta]
            $reverse_iterator:ident,
            yielding $iterty:ty
    } => {
        #[$forward_stability_attribute]
        impl<'a, P> DoubleEndedIterator for $forward_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: DoubleEndedSearcher<'a>>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
                self.0.next_back()
            }
        }

        #[$reverse_stability_attribute]
        impl<'a, P> DoubleEndedIterator for $reverse_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: DoubleEndedSearcher<'a>>,
        {
            #[inline]
            fn next_back(&mut self) -> Option<<Self as Iterator>::Item> {
                self.0.next()
            }
        }
    };
    {
        single ended; with
        #[$forward_stability_attribute:meta]
        $forward_iterator:ident,
        #[$reverse_stability_attribute:meta]
        $reverse_iterator:ident,
        yielding $iterty:ty
    } => {}
}

trait SplitIterInternal<'a>: Sized {
    type Pat: Pattern<'a>;
    fn next(&mut self) -> Option<&'a str>;

    fn next_back(&mut self) -> Option<&'a str>
    where
        <<Self as SplitIterInternal<'a>>::Pat as Pattern<'a>>::Searcher: ReverseSearcher<'a>;

    fn finish(&mut self) -> Option<&'a str>;

    fn as_str(&self) -> &'a str;
}

macro_rules! split_internal {
    (
        $split_struct:ident {
            $(skip_leading_empty: $skip_leading_empty:ident,)?
            $(skip_trailing_empty: $skip_trailing_empty:ident,)?
            include_leading: $include_leading:literal,
            include_trailing: $include_trailing:literal,
        }
    ) => {
        pub(super) struct $split_struct<'a, P: Pattern<'a>> {
            pub(super) start: usize,
            pub(super) end: usize,
            pub(super) matcher: P::Searcher,
            pub(super) finished: bool,
            $(pub(super) $skip_leading_empty: bool,)?
            $(pub(super) $skip_trailing_empty: bool,)?
        }

        derive_pattern_clone! {
            clone $split_struct
            with |s| $split_struct { matcher: s.matcher.clone(), ..*s }
        }

        impl<'a, P> fmt::Debug for $split_struct<'a, P>
        where
            P: Pattern<'a, Searcher: fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!(split_struct))
                    .field("start", &self.start)
                    .field("end", &self.end)
                    .field("matcher", &self.matcher)
                    .field("finished", &self.finished)
                    $(.field("skip_leading_empty", &self.$skip_leading_empty))?
                    $(.field("skip_trailing_empty", &self.$skip_trailing_empty))?
                    .finish()
            }
        }

        impl<'a, P: Pattern<'a>> $split_struct<'a, P> {
            #[inline]
            pub(super) fn new(s: &'a str, pat: P) -> Self {
                $split_struct {
                    start: 0,
                    end: s.len(),
                    matcher: pat.into_searcher(s),
                    finished: false,
                    $($skip_leading_empty: true,)?
                    $($skip_trailing_empty: true,)?
                }
            }
        }

        impl<'a, P: Pattern<'a>> SplitIterInternal<'a> for $split_struct<'a, P> {
            type Pat = P;

            #[inline]
            fn next(&mut self) -> Option<&'a str> {
                if self.finished {
                    return None;
                }

                $(if self.$skip_leading_empty {
                    self.$skip_leading_empty = false;
                    match self.next() {
                        Some(elt) if !elt.is_empty() => return Some(elt),
                        _ => {
                            if self.finished {
                                return None;
                            }
                        }
                    }
                })?

                let haystack = self.matcher.haystack();
                match self.matcher.next_match() {
                    // SAFETY: `Searcher` guarantees that `a` and `b` lie on unicode boundaries.
                    Some((a, b)) => unsafe {
                        let end_idx = if $include_trailing { b } else { a };
                        let elt = haystack.get_unchecked(self.start..end_idx);
                        self.start = if $include_leading { a } else { b };
                        Some(elt)
                    },
                    // SAFETY: `self.start` and `self.end` always lie on unicode boundaries.
                    None => unsafe {
                        let end = haystack.get_unchecked(self.start..self.end);
                        self.finished = true;
                        $(if self.$skip_trailing_empty && end == "" { return None; })?
                        Some(end)
                    },
                }
            }

            #[inline]
            fn next_back(&mut self) -> Option<&'a str>
            where
                P::Searcher: ReverseSearcher<'a>,
            {
                if self.finished {
                    return None;
                }

                $(if self.$skip_trailing_empty {
                    self.$skip_trailing_empty = false;
                    match self.next_back() {
                        Some(elt) if !elt.is_empty() => return Some(elt),
                        _ => {
                            if self.finished {
                                return None;
                            }
                        }
                    }
                })?

                let haystack = self.matcher.haystack();
                match self.matcher.next_match_back() {
                    // SAFETY: `Searcher` guarantees that `a` and `b` lie on unicode boundaries.
                    Some((a, b)) => unsafe {
                        let start_idx = if $include_leading { a } else { b };
                        let elt = haystack.get_unchecked(start_idx..self.end);
                        self.end = if $include_trailing { b } else { a };
                        Some(elt)
                    },
                    // SAFETY: `self.start` and `self.end` always lie on unicode boundaries.
                    None => unsafe {
                        let end = haystack.get_unchecked(self.start..self.end);
                        self.finished = true;
                        $(if self.$skip_leading_empty && end == "" { return None; })?
                        Some(end)
                    },
                }
            }

            #[inline]
            fn finish(&mut self) -> Option<&'a str> {
                if self.finished {
                    None
                } else {
                    self.finished = true;
                    // SAFETY: `self.start` and `self.end` always lie on unicode boundaries.
                    let end = unsafe { self.matcher.haystack().get_unchecked(self.start..self.end) };
                    if (false
                        $(|| self.$skip_leading_empty)?
                        $(|| self.$skip_trailing_empty)?
                    ) && end == "" {
                        None
                    } else {
                        Some(end)
                    }
                }
            }

            #[inline]
            fn as_str(&self) -> &'a str {
                // `Self::finish` doesn't change `self.start`
                if self.finished {
                    ""
                } else {
                    // SAFETY: `self.start` and `self.end` always lie on unicode boundaries.
                    unsafe { self.matcher.haystack().get_unchecked(self.start..self.end) }
                }
            }
        }
    }
}

macro_rules! generate_n_iterators {
    (
        forward:
            #[$forward_stability_attribute:meta]
            #[fused($forward_fused_stability_attribute:meta)]
            $(#[$forward_iterator_attribute:meta])*
            struct $forward_n_iterator:ident { inner: $forward_inner_iterator:ident }

            $(#[$forward_max_items_attribute:meta])*
            fn max_items;

            $($(#[$forward_as_str_attribute:meta])*
            fn as_str;)?
        reverse:
            #[$reverse_stability_attribute:meta]
            #[fused($reverse_fused_stability_attribute:meta)]
            $(#[$reverse_iterator_attribute:meta])*
            struct $reverse_n_iterator:ident { inner: $reverse_inner_iterator:ident }

            $(#[$reverse_max_items_attribute:meta])*
            fn max_items;

            $($(#[$reverse_as_str_attribute:meta])*
            fn as_str;)?
    ) => {
        #[$forward_stability_attribute]
        $(#[$forward_iterator_attribute])*
        pub struct $forward_n_iterator<'a, P: Pattern<'a>> {
            iter: $forward_inner_iterator<'a, P>,
            count: usize
        }

        derive_pattern_clone! {
            #[$forward_stability_attribute]
            clone $forward_n_iterator with |s| Self { iter: s.iter.clone(), count: s.count }
        }

        #[$forward_stability_attribute]
        impl<'a, P> fmt::Debug for $forward_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($forward_n_iterator))
                    .field("iter", &self.iter)
                    .field("count", &self.count)
                    .finish()
            }
        }

        #[$forward_stability_attribute]
        impl<'a, P: Pattern<'a>> Iterator for $forward_n_iterator<'a, P> {
            type Item = <$forward_inner_iterator<'a, P> as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self.count {
                    0 => None,
                    1 => {
                        self.count = 0;
                        self.iter.0.finish()
                    }
                    _ => {
                        self.count -= 1;
                        self.iter.next()
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.count))
            }
        }

        $(impl<'a, P: Pattern<'a>> $forward_n_iterator<'a, P> {
            $(#[$forward_as_str_attribute])*
            #[inline]
            pub fn as_str(&self) -> &'a str {
                self.iter.as_str()
            }
        })?

        impl<'a, P: Pattern<'a>> $forward_inner_iterator<'a, P> {
            $(#[$forward_max_items_attribute])*
            #[inline]
            pub fn max_items(self, n: usize) -> $forward_n_iterator<'a, P> {
                $forward_n_iterator { iter: self, count: n }
            }
        }

        #[$forward_fused_stability_attribute]
        impl<'a, P: Pattern<'a>> FusedIterator for $forward_n_iterator<'a, P> {}

        #[$reverse_stability_attribute]
        $(#[$reverse_iterator_attribute])*
        pub struct $reverse_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            iter: $reverse_inner_iterator<'a, P>,
            count: usize
        }

        derive_pattern_clone! {
            #[$reverse_stability_attribute]
            clone $reverse_n_iterator where Searcher: (ReverseSearcher<'a>) with |s| Self { iter: s.iter.clone(), count: s.count }
        }

        #[$reverse_stability_attribute]
        impl<'a, P> fmt::Debug for $reverse_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a> + fmt::Debug>,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($reverse_n_iterator))
                    .field("iter", &self.iter)
                    .field("count", &self.count)
                    .finish()
            }
        }

        #[$reverse_stability_attribute]
        impl<'a, P> Iterator for $reverse_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            type Item = <$reverse_inner_iterator<'a, P> as Iterator>::Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                match self.count {
                    0 => None,
                    1 => {
                        self.count = 0;
                        self.iter.0.finish()
                    }
                    _ => {
                        self.count -= 1;
                        self.iter.next()
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.count))
            }
        }

        $(impl<'a, P> $reverse_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            $(#[$reverse_as_str_attribute])*
            #[inline]
            pub fn as_str(&self) -> &'a str {
                self.iter.as_str()
            }
        })?


        impl<'a, P> $reverse_inner_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {
            $(#[$reverse_max_items_attribute])*
            #[inline]
            pub fn max_items(self, n: usize) -> $reverse_n_iterator<'a, P> {
                $reverse_n_iterator { iter: self, count: n }
            }
        }

        #[$reverse_fused_stability_attribute]
        impl<'a, P> FusedIterator for $reverse_n_iterator<'a, P>
        where
            P: Pattern<'a, Searcher: ReverseSearcher<'a>>,
        {}
    }
}

split_internal! {
    SplitInternal {
        include_leading: false,
        include_trailing: false,
    }
}

generate_pattern_iterators! {
    forward:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`split`].
        ///
        /// [`split`]: str::split
        struct Split;

        /// Returns remainder of the split string.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_as_str)]
        /// let mut split = "Mary had a little lamb".split(' ');
        /// assert_eq!(split.as_str(), "Mary had a little lamb");
        /// split.next();
        /// assert_eq!(split.as_str(), "had a little lamb");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`rsplit`].
        ///
        /// [`rsplit`]: str::rsplit
        struct RSplit;

        /// Returns remainder of the split string.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_as_str)]
        /// let mut split = "Mary had a little lamb".rsplit(' ');
        /// assert_eq!(split.as_str(), "Mary had a little lamb");
        /// split.next();
        /// assert_eq!(split.as_str(), "Mary had a little");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`splitn`].
        ///
        /// [`splitn`]: str::splitn
        struct SplitN { inner: Split }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`rsplitn`].
        ///
        /// [`rsplitn`]: str::rsplitn
        struct RSplitN { inner: RSplit }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;
}

split_internal! {
    SplitInclusiveInternal {
        skip_trailing_empty: skip_trailing_empty,
        include_leading: false,
        include_trailing: true,
    }
}

generate_pattern_iterators! {
    forward:
        #[stable(feature = "split_inclusive", since = "1.51.0")]
        #[fused(stable(feature = "split_inclusive", since = "1.51.0"))]
        /// Created with the method [`split_inclusive`].
        ///
        /// [`split_inclusive`]: str::split_inclusive
        struct SplitInclusive;

        /// Returns remainder of the split string
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_inclusive_as_str)]
        /// let mut split = "Mary had a little lamb".split_inclusive(' ');
        /// assert_eq!(split.as_str(), "Mary had a little lamb");
        /// split.next();
        /// assert_eq!(split.as_str(), "had a little lamb");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplit_inclusive`].
        ///
        /// [`rsplit_inclusive`]: str::rsplit_inclusive
        struct RSplitInclusive;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitInclusiveInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`splitn_inclusive`].
        ///
        /// [`splitn_inclusive`]: str::splitn_inclusive
        struct SplitNInclusive { inner: SplitInclusive }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplitn_inclusive`].
        ///
        /// [`rsplitn_inclusive`]: str::rsplitn_inclusive
        struct RSplitNInclusive { inner: RSplitInclusive }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;
}

split_internal! {
    SplitLeftInclusiveInternal {
        skip_leading_empty: skip_leading_empty,
        include_leading: true,
        include_trailing: false,
    }
}

generate_pattern_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`split_left_inclusive`].
        ///
        /// [`split_left_inclusive`]: str::split_left_inclusive
        struct SplitLeftInclusive;

        /// Returns remainder of the splitted string
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_inclusive_as_str)]
        /// #![feature(split_inclusive_variants)]
        /// let mut split = "Mary had a little lamb".split_left_inclusive(' ');
        /// assert_eq!(split.as_str(), "Mary had a little lamb");
        /// split.next();
        /// assert_eq!(split.as_str(), " had a little lamb");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplit_left_inclusive`].
        ///
        /// [`rsplit_left_inclusive`]: str::rsplit_left_inclusive
        struct RSplitLeftInclusive;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitLeftInclusiveInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`splitn_left_inclusive`].
        ///
        /// [`splitn_left_inclusive`]: str::splitn_left_inclusive
        struct SplitNLeftInclusive { inner: SplitLeftInclusive }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplitn_left_inclusive`].
        ///
        /// [`rsplitn_left_inclusive`]: str::rsplitn_left_inclusive
        struct RSplitNLeftInclusive { inner: RSplitLeftInclusive }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;
}

split_internal! {
    SplitInitiatorInternal {
        skip_leading_empty: skip_leading_empty,
        include_leading: false,
        include_trailing: false,
    }
}

generate_pattern_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`split_initiator`].
        ///
        /// [`split_initiator`]: str::split_initiator
        struct SplitInitiator;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplit_initiator`].
        ///
        /// [`rsplit_initiator`]: str::rsplit_initiator
        struct RSplitInitiator;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitInitiatorInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct SplitNInitiator { inner: SplitInitiator }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct RSplitNInitiator { inner: RSplitInitiator }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;
}

split_internal! {
    SplitTerminatorInternal {
        skip_trailing_empty: skip_trailing_empty,
        include_leading: false,
        include_trailing: false,
    }
}

generate_pattern_iterators! {
    forward:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`split_terminator`].
        ///
        /// [`split_terminator`]: str::split_terminator
        struct SplitTerminator;

        /// Returns remainder of the split string.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_as_str)]
        /// let mut split = "A..B..".split_terminator('.');
        /// assert_eq!(split.as_str(), "A..B..");
        /// split.next();
        /// assert_eq!(split.as_str(), ".B..");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[stable(feature = "rust1", since = "1.0.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`rsplit_terminator`].
        ///
        /// [`rsplit_terminator`]: str::rsplit_terminator
        struct RSplitTerminator;

        /// Returns remainder of the split string.
        ///
        /// # Examples
        ///
        /// ```
        /// #![feature(str_split_as_str)]
        /// let mut split = "A..B..".rsplit_terminator('.');
        /// assert_eq!(split.as_str(), "A..B..");
        /// split.next();
        /// assert_eq!(split.as_str(), "A..B");
        /// split.by_ref().for_each(drop);
        /// assert_eq!(split.as_str(), "");
        /// ```
        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitTerminatorInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct SplitNTerminator { inner: SplitTerminator }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct RSplitNTerminator { inner: RSplitTerminator }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;
}

split_internal! {
    SplitEndsInternal {
        skip_leading_empty: skip_leading_empty,
        skip_trailing_empty: skip_trailing_empty,
        include_leading: false,
        include_trailing: false,
    }
}

generate_pattern_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`split_ends`].
        ///
        /// [`split_ends`]: str::split_ends
        struct SplitEnds;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        /// Created with the method [`rsplit_ends`].
        ///
        /// [`rsplit_ends`]: str::rsplit_ends
        struct RSplitEnds;

        #[unstable(feature = "str_split_as_str", issue = "77998")]
        fn as_str;

    internal:
        SplitEndsInternal yielding (&'a str);
    delegate double ended;
}

generate_n_iterators! {
    forward:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct SplitNEnds { inner: SplitEnds }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;

    reverse:
        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        #[fused(unstable(feature = "split_inclusive_variants", issue = "none"))]
        struct RSplitNEnds { inner: RSplitEnds }

        #[unstable(feature = "split_inclusive_variants", issue = "none")]
        fn max_items;

        #[unstable(feature = "str_split_inclusive_as_str", issue = "77998")]
        fn as_str;
}

derive_pattern_clone! {
    clone MatchIndicesInternal
    with |s| MatchIndicesInternal(s.0.clone())
}

pub(super) struct MatchIndicesInternal<'a, P: Pattern<'a>>(pub(super) P::Searcher);

impl<'a, P> fmt::Debug for MatchIndicesInternal<'a, P>
where
    P: Pattern<'a, Searcher: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MatchIndicesInternal").field(&self.0).finish()
    }
}

impl<'a, P: Pattern<'a>> MatchIndicesInternal<'a, P> {
    #[inline]
    fn next(&mut self) -> Option<(usize, &'a str)> {
        self.0
            .next_match()
            // SAFETY: `Searcher` guarantees that `start` and `end` lie on unicode boundaries.
            .map(|(start, end)| unsafe { (start, self.0.haystack().get_unchecked(start..end)) })
    }

    #[inline]
    fn next_back(&mut self) -> Option<(usize, &'a str)>
    where
        P::Searcher: ReverseSearcher<'a>,
    {
        self.0
            .next_match_back()
            // SAFETY: `Searcher` guarantees that `start` and `end` lie on unicode boundaries.
            .map(|(start, end)| unsafe { (start, self.0.haystack().get_unchecked(start..end)) })
    }
}

generate_pattern_iterators! {
    forward:
        #[stable(feature = "str_match_indices", since = "1.5.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`match_indices`].
        ///
        /// [`match_indices`]: str::match_indices
        struct MatchIndices;
    reverse:
        #[stable(feature = "str_match_indices", since = "1.5.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`rmatch_indices`].
        ///
        /// [`rmatch_indices`]: str::rmatch_indices
        struct RMatchIndices;
    internal:
        MatchIndicesInternal yielding ((usize, &'a str));
    delegate double ended;
}

derive_pattern_clone! {
    clone MatchesInternal
    with |s| MatchesInternal(s.0.clone())
}

pub(super) struct MatchesInternal<'a, P: Pattern<'a>>(pub(super) P::Searcher);

impl<'a, P> fmt::Debug for MatchesInternal<'a, P>
where
    P: Pattern<'a, Searcher: fmt::Debug>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MatchesInternal").field(&self.0).finish()
    }
}

impl<'a, P: Pattern<'a>> MatchesInternal<'a, P> {
    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        // SAFETY: `Searcher` guarantees that `start` and `end` lie on unicode boundaries.
        self.0.next_match().map(|(a, b)| unsafe {
            // Indices are known to be on utf8 boundaries
            self.0.haystack().get_unchecked(a..b)
        })
    }

    #[inline]
    fn next_back(&mut self) -> Option<&'a str>
    where
        P::Searcher: ReverseSearcher<'a>,
    {
        // SAFETY: `Searcher` guarantees that `start` and `end` lie on unicode boundaries.
        self.0.next_match_back().map(|(a, b)| unsafe {
            // Indices are known to be on utf8 boundaries
            self.0.haystack().get_unchecked(a..b)
        })
    }
}

generate_pattern_iterators! {
    forward:
        #[stable(feature = "str_matches", since = "1.2.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`matches`].
        ///
        /// [`matches`]: str::matches
        struct Matches;
    reverse:
        #[stable(feature = "str_matches", since = "1.2.0")]
        #[fused(stable(feature = "fused", since = "1.26.0"))]
        /// Created with the method [`rmatches`].
        ///
        /// [`rmatches`]: str::rmatches
        struct RMatches;
    internal:
        MatchesInternal yielding (&'a str);
    delegate double ended;
}

/// An iterator over the lines of a string, as string slices.
///
/// This struct is created with the [`lines`] method on [`str`].
/// See its documentation for more.
///
/// [`lines`]: str::lines
#[stable(feature = "rust1", since = "1.0.0")]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Lines<'a>(pub(super) Map<SplitTerminator<'a, char>, LinesAnyMap>);

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> Iterator for Lines<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<&'a str> {
        self.next_back()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
impl<'a> DoubleEndedIterator for Lines<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.0.next_back()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for Lines<'_> {}

/// Created with the method [`lines_any`].
///
/// [`lines_any`]: str::lines_any
#[stable(feature = "rust1", since = "1.0.0")]
#[deprecated(since = "1.4.0", note = "use lines()/Lines instead now")]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
#[allow(deprecated)]
pub struct LinesAny<'a>(pub(super) Lines<'a>);

#[stable(feature = "rust1", since = "1.0.0")]
#[allow(deprecated)]
impl<'a> Iterator for LinesAny<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
#[allow(deprecated)]
impl<'a> DoubleEndedIterator for LinesAny<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.0.next_back()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
#[allow(deprecated)]
impl FusedIterator for LinesAny<'_> {}

/// An iterator over the non-whitespace substrings of a string,
/// separated by any amount of whitespace.
///
/// This struct is created by the [`split_whitespace`] method on [`str`].
/// See its documentation for more.
///
/// [`split_whitespace`]: str::split_whitespace
#[stable(feature = "split_whitespace", since = "1.1.0")]
#[derive(Clone, Debug)]
pub struct SplitWhitespace<'a> {
    pub(super) inner: Filter<Split<'a, IsWhitespace>, IsNotEmpty>,
}

/// An iterator over the non-ASCII-whitespace substrings of a string,
/// separated by any amount of ASCII whitespace.
///
/// This struct is created by the [`split_ascii_whitespace`] method on [`str`].
/// See its documentation for more.
///
/// [`split_ascii_whitespace`]: str::split_ascii_whitespace
#[stable(feature = "split_ascii_whitespace", since = "1.34.0")]
#[derive(Clone, Debug)]
pub struct SplitAsciiWhitespace<'a> {
    pub(super) inner:
        Map<Filter<SliceSplit<'a, u8, IsAsciiWhitespace>, BytesIsNotEmpty>, UnsafeBytesToStr>,
}

#[stable(feature = "split_whitespace", since = "1.1.0")]
impl<'a> Iterator for SplitWhitespace<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<&'a str> {
        self.next_back()
    }
}

#[stable(feature = "split_whitespace", since = "1.1.0")]
impl<'a> DoubleEndedIterator for SplitWhitespace<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.inner.next_back()
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for SplitWhitespace<'_> {}

impl<'a> SplitWhitespace<'a> {
    /// Returns remainder of the split string
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(str_split_whitespace_as_str)]
    ///
    /// let mut split = "Mary had a little lamb".split_whitespace();
    /// assert_eq!(split.as_str(), "Mary had a little lamb");
    ///
    /// split.next();
    /// assert_eq!(split.as_str(), "had a little lamb");
    ///
    /// split.by_ref().for_each(drop);
    /// assert_eq!(split.as_str(), "");
    /// ```
    #[inline]
    #[must_use]
    #[unstable(feature = "str_split_whitespace_as_str", issue = "77998")]
    pub fn as_str(&self) -> &'a str {
        self.inner.iter.as_str()
    }
}

#[stable(feature = "split_ascii_whitespace", since = "1.34.0")]
impl<'a> Iterator for SplitAsciiWhitespace<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<&'a str> {
        self.next_back()
    }
}

#[stable(feature = "split_ascii_whitespace", since = "1.34.0")]
impl<'a> DoubleEndedIterator for SplitAsciiWhitespace<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a str> {
        self.inner.next_back()
    }
}

#[stable(feature = "split_ascii_whitespace", since = "1.34.0")]
impl FusedIterator for SplitAsciiWhitespace<'_> {}

impl<'a> SplitAsciiWhitespace<'a> {
    /// Returns remainder of the split string
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(str_split_whitespace_as_str)]
    ///
    /// let mut split = "Mary had a little lamb".split_ascii_whitespace();
    /// assert_eq!(split.as_str(), "Mary had a little lamb");
    ///
    /// split.next();
    /// assert_eq!(split.as_str(), "had a little lamb");
    ///
    /// split.by_ref().for_each(drop);
    /// assert_eq!(split.as_str(), "");
    /// ```
    #[inline]
    #[must_use]
    #[unstable(feature = "str_split_whitespace_as_str", issue = "77998")]
    pub fn as_str(&self) -> &'a str {
        if self.inner.iter.iter.finished {
            return "";
        }

        // SAFETY: Slice is created from str.
        unsafe { crate::str::from_utf8_unchecked(&self.inner.iter.iter.v) }
    }
}

/// An iterator of [`u16`] over the string encoded as UTF-16.
///
/// This struct is created by the [`encode_utf16`] method on [`str`].
/// See its documentation for more.
///
/// [`encode_utf16`]: str::encode_utf16
#[derive(Clone)]
#[stable(feature = "encode_utf16", since = "1.8.0")]
pub struct EncodeUtf16<'a> {
    pub(super) chars: Chars<'a>,
    pub(super) extra: u16,
}

#[stable(feature = "collection_debug", since = "1.17.0")]
impl fmt::Debug for EncodeUtf16<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncodeUtf16").finish_non_exhaustive()
    }
}

#[stable(feature = "encode_utf16", since = "1.8.0")]
impl<'a> Iterator for EncodeUtf16<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<u16> {
        if self.extra != 0 {
            let tmp = self.extra;
            self.extra = 0;
            return Some(tmp);
        }

        let mut buf = [0; 2];
        self.chars.next().map(|ch| {
            let n = ch.encode_utf16(&mut buf).len();
            if n == 2 {
                self.extra = buf[1];
            }
            buf[0]
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.chars.size_hint();
        // every char gets either one u16 or two u16,
        // so this iterator is between 1 or 2 times as
        // long as the underlying iterator.
        (low, high.and_then(|n| n.checked_mul(2)))
    }
}

#[stable(feature = "fused", since = "1.26.0")]
impl FusedIterator for EncodeUtf16<'_> {}

/// The return type of [`str::escape_debug`].
#[stable(feature = "str_escape", since = "1.34.0")]
#[derive(Clone, Debug)]
pub struct EscapeDebug<'a> {
    pub(super) inner: Chain<
        Flatten<option::IntoIter<char::EscapeDebug>>,
        FlatMap<Chars<'a>, char::EscapeDebug, CharEscapeDebugContinue>,
    >,
}

/// The return type of [`str::escape_default`].
#[stable(feature = "str_escape", since = "1.34.0")]
#[derive(Clone, Debug)]
pub struct EscapeDefault<'a> {
    pub(super) inner: FlatMap<Chars<'a>, char::EscapeDefault, CharEscapeDefault>,
}

/// The return type of [`str::escape_unicode`].
#[stable(feature = "str_escape", since = "1.34.0")]
#[derive(Clone, Debug)]
pub struct EscapeUnicode<'a> {
    pub(super) inner: FlatMap<Chars<'a>, char::EscapeUnicode, CharEscapeUnicode>,
}

macro_rules! escape_types_impls {
    ($( $Name: ident ),+) => {$(
        #[stable(feature = "str_escape", since = "1.34.0")]
        impl<'a> fmt::Display for $Name<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.clone().try_for_each(|c| f.write_char(c))
            }
        }

        #[stable(feature = "str_escape", since = "1.34.0")]
        impl<'a> Iterator for $Name<'a> {
            type Item = char;

            #[inline]
            fn next(&mut self) -> Option<char> { self.inner.next() }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }

            #[inline]
            fn try_fold<Acc, Fold, R>(&mut self, init: Acc, fold: Fold) -> R where
                Self: Sized, Fold: FnMut(Acc, Self::Item) -> R, R: Try<Output = Acc>
            {
                self.inner.try_fold(init, fold)
            }

            #[inline]
            fn fold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
                where Fold: FnMut(Acc, Self::Item) -> Acc,
            {
                self.inner.fold(init, fold)
            }
        }

        #[stable(feature = "str_escape", since = "1.34.0")]
        impl<'a> FusedIterator for $Name<'a> {}
    )+}
}

escape_types_impls!(EscapeDebug, EscapeDefault, EscapeUnicode);
