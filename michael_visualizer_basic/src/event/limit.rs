use crate::data_types::LimitLabel;

pub enum LimitEvent<Key, LimitData> {
    Value(Key, LimitData),
    Label(Key, LimitLabel),
    ToPlot(Key),
    //FormulaAdded(Key),
    //FormulaRemoved(Key),
}
