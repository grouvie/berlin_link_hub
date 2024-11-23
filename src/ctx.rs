#[derive(Clone, Debug)]
pub(crate) struct Ctx {
    user_id: usize,
}

// Constructor.
impl Ctx {
    pub(crate) const fn new(user_id: usize) -> Self {
        Self { user_id }
    }
}

// Property Accessors.
impl Ctx {
    pub(crate) fn user_id(&self) -> usize {
        self.user_id
    }
}
