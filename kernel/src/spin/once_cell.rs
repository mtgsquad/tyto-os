use core::cell::UnsafeCell;

pub struct OnceCell<T>(Option<UnsafeCell<T>>);

impl<T> OnceCell<T> {
    /// Creates a new empty cell.
    pub fn new() -> Self {
        Self(None)
    }

    /// Gets the reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    ///
    /// # Example
    ///
    /// ```
    /// use spin::OnceCell;
    ///
    /// let cell = OnceCell::new();
    /// assert!(cell.get().is_none());
    /// cell.set(1);
    /// assert_eq!(cell.get(), Some(&1));
    /// ```
    pub fn get(&self) -> Option<&T> {
        self.0.map(|cell| unsafe { &*cell.get() })
    }

    /// Gets the mutable reference to the underlying value.
    ///
    /// Returns `None` if the cell is empty.
    ///
    /// This method is allowed to violate the invariant of writing to a `OnceCell`
    /// at most once because it requires `&mut` access to `self`. As with all
    /// interior mutability, `&mut` access permits arbitrary modification:
    ///
    /// # Example
    ///
    /// ```
    /// use spin::OnceCell;
    ///
    /// let mut cell: OnceCell<u32> = OnceCell::new();
    /// cell.set(92).unwrap();
    /// cell = OnceCell::new();
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut().map(|cell| cell.get_mut())
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.0.as_ref().unwrap_unchecked().get()
    }

    pub unsafe fn get_unchecked_mut(&self) -> &mut T {
        &mut *self.0.as_mut().unwrap_unchecked().get()
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        if self.0.is_none() {
            self.0 = Some(UnsafeCell::new(value));
            Ok(())
        } else {
            Err(value)
        }
    }
}
