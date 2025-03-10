use std::str::FromStr;

#[derive(Debug, Default, Clone)]
pub struct DataInput {
    pub name: String,
    pub value: String,
}

impl DataInput {
    pub fn get<T: FromStr>(&self) -> Result<T, T::Err> {
        self.value.parse::<T>()
    }
}
