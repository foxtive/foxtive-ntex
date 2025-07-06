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

    pub fn inner(&self) -> &DataInput {
        self
    }

    pub fn into_inner(self) -> DataInput {
        self
    }

    pub fn parts(&self) -> (&String, &String) {
        (&self.name, &self.value)
    }

    pub fn into_parts(self) -> (String, String) {
        (self.name, self.value)
    }
}
