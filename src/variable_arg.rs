use std::cmp::Ordering;

use crate::process_template::VariableValue;

// Used to sort args from clap.
pub struct VariableArg {
    pub index: usize,
    pub name: String,
    pub value: VariableValue,
}

impl PartialEq for VariableArg {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for VariableArg {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other).reverse())
    }
}

impl Ord for VariableArg {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl Eq for VariableArg {}