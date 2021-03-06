//! Compact representation of multiple forward linked lists.
//!
//! This module defines [``NestedForwardList``].  Client
//! code will typically use this type via [``crate::EdgeBuffer``].
//! See the documentation of that type for details.
//!
//! Most of API for this type is used internally, but
//! it is public in case anyone finds other uses for
//! this data structure.

use thiserror::Error;

/// Errror type for [``NestedForwardList``] operations.
#[derive(Error, Debug, PartialEq)]
pub enum NestedForwardListError {
    /// Tail of a list is unexpectedly null.
    #[error("Tail is null")]
    NullTail,
    /// Used for invalid index values.
    #[error("Invalid index")]
    InvalidIndex,
}

/// Result type for [``NestedForwardList``] operations.
pub type Result<T> = std::result::Result<T, NestedForwardListError>;

// NOTE: I am unclear how to add a Key
// to this generic.  Unlike C++, there's
// no notion of a static_assert.  Gotta Google
// this in the future!

/// Representation of multiple forward linked
/// lists flattend into vectors.
///
/// # Overview
///
/// A typical representation of a forward list
/// involves `head`, `next`, and `tail` pointers
/// to allow iteration in one direction over
/// a `Value`.
///
/// The memory layout of such a list could be
/// "node" pointers allocated on the heap.
/// For example, the node pointer could be intrusive,
/// responsible for managing ownership of the `Value`
/// elements.
///
/// Alternately, we could use flat arrays.
///
/// This type is capable of holding multiple
/// linked lists in a single set of flattened arrays.
///
/// Internally, the `head` array is accessed by list indexes.
/// In other words, to access the i-th list, make a request
/// for the i-th `head`.  Likewise for `tail`.
///
/// # Usage
///
/// This section documents typical use.
/// Other parts of the API exist, but are mostly used internally.
/// They are `pub` in case anyone finds them useful.
///
/// ```
/// // Our value type will be i32.
/// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
/// let mut l = ListType::new();
///
/// // Create 4 empty lists
/// l.reset(4);
/// assert_eq!(l.len(), 4);
///
/// // add some data to list starting
/// // at index 2
/// l.extend(2, -1);
///
/// // There are two methods to traverse the list.
/// // 1. Explicitly use head/next values:
///
/// let mut output = vec![];
/// // Get the head of this list
/// let mut n = l.head(2).unwrap();
/// while n != ListType::null() {
///    // fetch returns an immutable reference.
///    output.push(*l.fetch(n).unwrap());
///    // proceed to the next element
///    n = l.next(n).unwrap();
/// }
/// assert_eq!(output, vec![-1]);
///
/// // 2. Use a closure.  The closure
/// // returns `true` to keep iterating,
/// // or `false` to stop, allowing searches.
/// output.clear();
/// l.for_each(2, |x| {output.push(*x); true});
/// assert_eq!(output, vec![-1]);
///
/// // We can add data out of order:
/// l.extend(0, 13).unwrap();
///
/// // Adding at a new index > that what we
/// // asked for with "reset" will automatically
/// // reallocate.  Create a list starting at index
/// // 13
/// l.extend(13, 13512).unwrap();
/// assert_eq!(l.len(), 14);
///
/// // Our existing data are still fine after
/// // this reallocation:
/// output.clear();
/// l.for_each(2, |x| {output.push(*x); true});
/// assert_eq!(output, vec![-1]);
/// ```
///
/// The following functions may be useful:
///
/// * [``NestedForwardList::clear``]
/// * [``NestedForwardList::fetch_mut``]
pub struct NestedForwardList<Value> {
    head_: Vec<i32>,
    tail_: Vec<i32>,
    next_: Vec<i32>,
    data_: Vec<Value>,
}

impl<Value> NestedForwardList<Value> {
    fn insert_new_record(&mut self, k: i32, v: Value) {
        self.data_.push(v);
        let x = (self.data_.len() - 1) as i32;
        self.head_[k as usize] = x;
        self.tail_[k as usize] = self.head_[k as usize];
        self.next_.push(NestedForwardList::<Value>::null());
    }

    fn check_key(&self, k: i32) -> Result<()> {
        if k < 0 {
            Err(NestedForwardListError::InvalidIndex)
        } else {
            Ok(())
        }
    }

    fn check_key_range(&self, k: usize, n: usize) -> Result<()> {
        if k >= n {
            Err(NestedForwardListError::InvalidIndex)
        } else {
            Ok(())
        }
    }

    // Public functions:

    /// Create a new instance
    pub const fn new() -> NestedForwardList<Value> {
        NestedForwardList {
            head_: Vec::<i32>::new(),
            tail_: Vec::<i32>::new(),
            next_: Vec::<i32>::new(),
            data_: Vec::<Value>::new(),
        }
    }

    /// Add an element to a list starting at index `k`.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap();
    /// ```
    pub fn extend(&mut self, k: i32, v: Value) -> Result<()> {
        self.check_key(k)?;

        let idx = k as usize;
        if idx >= self.head_.len() {
            self.head_
                .resize(idx + 1, NestedForwardList::<Value>::null());
            self.tail_
                .resize(idx + 1, NestedForwardList::<Value>::null());
        }

        if self.head_[idx] == NestedForwardList::<Value>::null() {
            self.insert_new_record(idx as i32, v);
            return Ok(());
        }
        let t = self.tail_[idx];
        if t == NestedForwardList::<Value>::null() {
            return Err(NestedForwardListError::NullTail {});
        }
        self.data_.push(v);
        self.tail_[idx] = (self.data_.len() - 1) as i32;
        // NOTE: this differs from fwdpp, to avoid cast,
        // and thus could be failure point?
        self.next_[t as usize] = self.tail_[idx];
        self.next_.push(NestedForwardList::<Value>::null());
        Ok(())
    }

    /// Return the null value,
    /// which is -1.
    #[inline]
    pub fn null() -> i32 {
        -1
    }

    /// Get a mutable reference to a `Value`.
    ///
    /// See [``NestedForwardList::fetch``]
    /// to get an immutable reference.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap(); // 0
    /// l.extend(1, 7).unwrap(); // 3
    /// l.extend(0, 11).unwrap(); // 2
    ///
    /// // Go through elements of list starting at 0
    /// let mut i = l.head(0).unwrap();
    /// assert_eq!(*l.fetch(i).unwrap(), 1);
    ///
    /// // We can change the data contents:
    /// *l.fetch_mut(i).unwrap() = -33;
    /// assert_eq!(*l.fetch(i).unwrap(), -33);
    /// ```    
    #[inline]
    pub fn fetch_mut(&mut self, at: i32) -> Result<&mut Value> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.data_.len())?;
        Ok(&mut self.data_[at as usize])
    }

    /// Get a reference to a `Value`.
    ///
    /// See [``NestedForwardList::fetch_mut``]
    /// to get an immutable reference.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap(); // 0
    /// l.extend(1, 7).unwrap(); // 3
    /// l.extend(0, 11).unwrap(); // 2
    ///
    /// // Go through elements of list starting at
    /// // 0
    /// let mut i = l.head(0).unwrap();
    /// assert_eq!(*l.fetch(i).unwrap(), 1);
    /// i = l.next(i).unwrap();
    /// assert_eq!(*l.fetch(i).unwrap(), 11);
    /// ```    
    #[inline]
    pub fn fetch(&self, at: i32) -> Result<&Value> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.data_.len())?;
        Ok(&self.data_[at as usize])
    }

    /// Get the index of the head entry of a list
    /// beginning at index `at`.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap(); // 0
    /// l.extend(1, 3).unwrap(); // 1
    /// l.extend(0, 5).unwrap(); // 2
    /// l.extend(1, 5).unwrap(); // 3
    /// assert_eq!(l.head(0).unwrap(), 0);
    /// assert_eq!(l.head(1).unwrap(), 1);
    /// ```    
    #[inline]
    pub fn head(&self, at: i32) -> Result<i32> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.head_.len())?;
        Ok(self.head_[at as usize])
    }

    /// Get the index of the tail entry of a list
    /// beginning at index `at`.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap(); // 0
    /// l.extend(1, 3).unwrap(); // 1
    /// l.extend(0, 5).unwrap(); // 2
    /// l.extend(1, 5).unwrap(); // 3
    /// assert_eq!(l.tail(0).unwrap(), 2);
    /// assert_eq!(l.tail(1).unwrap(), 3);
    /// ```    
    #[inline]
    pub fn tail(&self, at: i32) -> Result<i32> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.tail_.len())?;
        Ok(self.tail_[at as usize])
    }

    /// Get the index of the next data element in a list
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap();
    /// l.extend(1, 3).unwrap();
    /// l.extend(0, 5).unwrap();
    /// l.extend(1, 5).unwrap();
    /// let mut i = l.head(1).unwrap();
    /// assert_eq!(i, 1);
    /// let i = l.next(i).unwrap();
    /// assert_eq!(i, 3);
    /// ```
    #[inline]
    pub fn next(&self, at: i32) -> Result<i32> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.next_.len())?;
        Ok(self.next_[at as usize])
    }

    /// Clears all data.
    /// Memory is not released.
    pub fn clear(&mut self) {
        self.data_.clear();
        self.head_.clear();
        self.tail_.clear();
        self.next_.clear();
    }

    /// Traverse all data elements for the list
    /// beginning at index `at`.
    ///
    /// The callback returns a [``bool``].
    /// If the callback returns [``false``], then traversal
    /// ends, allowing linear searches through the data.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut list = ListType::new();
    /// for i in 0..3 {
    ///     list.extend(0, 2 * i).unwrap();
    /// }
    /// for i in 0..5 {
    ///     list.extend(1, 3 * i).unwrap();
    /// }
    /// let mut output = vec![];
    /// list.for_each(0, |x: &i32| {
    ///     output.push(*x);
    ///     true
    /// })
    /// .unwrap();
    /// assert_eq!(output, vec![0, 2, 4]);
    /// ```
    pub fn for_each(&self, at: i32, mut f: impl FnMut(&Value) -> bool) -> Result<()> {
        let mut itr = self.head(at)?;
        while itr != NestedForwardList::<Value>::null() {
            let val = self.fetch(itr)?;
            let check = f(val);
            if !check {
                break;
            }
            itr = self.next(itr)?;
        }
        Ok(())
    }

    /// Set the head/tail elements of the list
    /// beginning at index ``at`` to [``NestedForwardList::null``].
    ///
    /// # Notes
    ///
    /// This effectively "kills off" the list, preventing traversal.
    /// However, the internal data are unaffected and there is
    /// no attempt to reclaim memory.
    ///
    /// A future release may implement a stack of nullified locations,
    /// allowing their re-use.
    ///
    pub fn nullify_list(&mut self, at: i32) -> Result<()> {
        self.check_key(at)?;
        self.check_key_range(at as usize, self.head_.len())?;
        self.check_key_range(at as usize, self.tail_.len())?;
        self.head_[at as usize] = NestedForwardList::<Value>::null();
        self.tail_[at as usize] = NestedForwardList::<Value>::null();
        Ok(())
    }

    /// Executes the following:
    ///
    /// 1. Clears all data via [``NestedForwardList::clear``].
    /// 2. Sets the size of the number of lists to `newize`.
    /// 3. The lists are all empty, and this have head/tail
    ///    values of [``NestedForwardList::null``].
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap();
    /// l.extend(0, 3).unwrap();
    /// l.extend(0, 5).unwrap();
    /// l.extend(1, 4).unwrap();
    /// assert_eq!(l.len(), 2);
    /// assert_eq!(l.head(0).unwrap(), 0);
    /// assert_eq!(l.tail(0).unwrap(), 2);
    /// l.reset(1);
    /// assert_eq!(l.len(), 1);
    /// // The head of this list is now null
    /// assert_eq!(l.head(0).unwrap(), ListType::null());
    /// ```
    pub fn reset(&mut self, newsize: usize) {
        self.clear();
        self.head_
            .resize(newsize, NestedForwardList::<Value>::null());
        self.tail_
            .resize(newsize, NestedForwardList::<Value>::null());
        // .fill() is experimental right now...
        //self.head_.fill(NestedForwardList::<Value>::null());
        //self.tail_.fill(NestedForwardList::<Value>::null());
        // ... so we do this lambda mapping instead
        self.head_.iter_mut().for_each(|x| *x = Self::null());
        self.tail_.iter_mut().for_each(|x| *x = Self::null());
    }

    /// Obtain an iterator over the vector of list
    /// heads.
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// l.extend(0, 1).unwrap();
    /// l.extend(0, 3).unwrap();
    /// l.extend(0, 5).unwrap();
    /// l.extend(1, -11).unwrap();
    /// // List 0 starts at index 0
    /// // and lines 1 starts at index 3
    /// let heads = vec![0, 3];
    /// for (i, j) in l.head_itr().enumerate() {
    ///     assert_eq!(*j, heads[i]);
    /// }
    ///
    /// let mut output = vec![];
    /// for (i, j) in l.head_itr().enumerate() {
    ///     l.for_each(i as i32, |x| {output.push(*x); true});
    /// }
    /// assert_eq!(output, vec![1,3,5,-11]);
    /// ```
    pub fn head_itr(&self) -> std::slice::Iter<'_, i32> {
        self.head_.iter()
    }

    /// Return length of the vector
    /// of list heads.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// assert_eq!(l.len(), 0);
    /// l.extend(10, 1);
    /// assert_eq!(l.len(), 11);
    /// ```
    pub fn len(&self) -> usize {
        self.head_.len()
    }

    /// Return [``true``] if the
    /// vector of list heads is empty.
    ///
    /// ```
    /// type ListType = forrustts::nested_forward_list::NestedForwardList<i32>;
    /// let mut l = ListType::new();
    /// assert_eq!(l.is_empty(), true);
    /// l.extend(10, 1);
    /// assert_eq!(l.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.head_.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    type ListType = NestedForwardList<i32>;

    struct Datum {
        datum: i32,
    }

    type DatumList = NestedForwardList<Datum>;

    fn make_data_for_testing() -> ListType {
        let mut list = ListType::new();
        list.reset(2);
        for i in 0..3 {
            list.extend(0, 2 * i).unwrap();
        }
        for i in 0..5 {
            list.extend(1, 3 * i).unwrap();
        }
        assert_eq!(list.head_.len(), 2);
        assert_eq!(list.data_.len(), 8);
        assert_eq!(list.tail_.len(), 2);
        list
    }

    #[test]
    fn test_head_tail() {
        let list = make_data_for_testing();
        assert_eq!(*list.fetch(list.head(0).unwrap()).unwrap(), 0);
        assert_eq!(*list.fetch(list.tail(0).unwrap()).unwrap(), 4);
        assert_eq!(*list.fetch(list.head(1).unwrap()).unwrap(), 0);
        assert_eq!(*list.fetch(list.tail(1).unwrap()).unwrap(), 12);
    }

    #[test]
    fn test_fetch_mut() {
        let mut list = make_data_for_testing();
        let x = list.tail(1).unwrap();
        let y = list.fetch_mut(x).unwrap();
        *y += 1;
        assert_eq!(*list.fetch(list.tail(1).unwrap()).unwrap(), 13);
    }

    #[test]
    fn test_fetch_mut_struct() {
        let mut list = DatumList::new();
        list.reset(1);
        list.extend(0, Datum { datum: 0 }).unwrap();
        let x = list.tail(0).unwrap();
        let y = list.fetch_mut(x).unwrap();
        y.datum = 111;
        assert_eq!(list.fetch(list.tail(0).unwrap()).unwrap().datum, 111);
    }

    #[test]
    fn test_explicit_data_round_trip() {
        let list = make_data_for_testing();

        let mut output = Vec::<i32>::new();
        let mut itr = list.head(0).unwrap();
        while itr != ListType::null() {
            let val = list.fetch(itr).unwrap();
            output.push(*val);
            itr = list.next(itr).unwrap();
        }
        assert_eq!(3, output.len());

        for (idx, val) in output.iter().enumerate().take(3) {
            assert_eq!(2 * idx, *val as usize);
        }

        output.clear();

        itr = list.head(1).unwrap();
        while itr != ListType::null() {
            let val = list.fetch(itr).unwrap();
            output.push(*val);
            itr = list.next(itr).unwrap();
        }
        assert_eq!(5, output.len());

        for (idx, val) in output.iter().enumerate().take(5) {
            assert_eq!(3 * idx, *val as usize);
        }
    }

    #[test]
    fn test_nullify() {
        let mut list = make_data_for_testing();
        list.nullify_list(0).unwrap();
        let mut output = Vec::<i32>::new();
        list.for_each(0, |x: &i32| {
            output.push(*x);
            true
        })
        .unwrap();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_for_each_data_round_trip() {
        let list = make_data_for_testing();
        let mut output = Vec::<i32>::new();

        list.for_each(0, |x: &i32| {
            output.push(*x);
            true
        })
        .unwrap();

        for (idx, val) in output.iter().enumerate().take(3) {
            assert_eq!(2 * idx, *val as usize);
        }

        output.clear();

        list.for_each(1, |x: &i32| {
            output.push(*x);
            true
        })
        .unwrap();

        for (idx, val) in output.iter().enumerate().take(5) {
            assert_eq!(3 * idx, *val as usize);
        }

        output.clear();
        list.for_each(1, |_: &i32| false).unwrap();

        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_check_key() {
        let mut list = make_data_for_testing();
        list.reset(1);
        let result = list.extend(-1, 2);
        match result {
            Ok(_) => panic!(),
            Err(NestedForwardListError::InvalidIndex) => (),
            Err(NestedForwardListError::NullTail) => panic!(),
        }
    }

    #[test]
    fn test_head_forward_iteration() {
        let list = make_data_for_testing();
        let mut output = Vec::<i32>::new();
        for (i, _) in list.head_itr().enumerate() {
            list.for_each(i as i32, |x: &i32| {
                output.push(*x);
                true
            })
            .unwrap();
        }
        assert_eq!(output.len(), 8);
        for (idx, val) in output.iter().enumerate().take(3) {
            assert_eq!(2 * idx, *val as usize);
        }
        for (idx, val) in output.iter().enumerate().skip(3) {
            assert_eq!(3 * (idx - 3), *val as usize);
        }
    }

    #[test]
    fn test_head_reverse_iteration() {
        let list = make_data_for_testing();
        let mut output = Vec::<i32>::new();
        for (i, _) in list.head_itr().rev().enumerate() {
            list.for_each((list.len() - i - 1) as i32, |x: &i32| {
                output.push(*x);
                true
            })
            .unwrap();
        }
        assert_eq!(output.len(), 8);
        for (idx, val) in output.iter().enumerate().take(5) {
            assert_eq!(3 * idx, *val as usize);
        }
        for (idx, val) in output.iter().enumerate().skip(5) {
            assert_eq!(2 * (idx - 5), *val as usize);
        }
    }
}
