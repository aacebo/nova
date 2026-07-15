use crate::ToType;

#[macro_export]
macro_rules! map {
    () => {{ ::std::collections::HashMap::new() }};
    ( $( { $key:expr, $value:expr } ),+ $(,)? ) => {{
        ::std::collections::HashMap::from([ $(($key, $value),)+ ])
    }};
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        ::std::collections::HashMap::from([ $(($key, $value),)+ ])
    }};
}

#[macro_export]
macro_rules! btree_map {
    () => {{ ::std::collections::BTreeMap::new() }};
    ( $( { $key:expr, $value:expr } ),+ $(,)? ) => {{
        ::std::collections::BTreeMap::from([ $(($key, $value),)+ ])
    }};
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        ::std::collections::BTreeMap::from([ $(($key, $value),)+ ])
    }};
}

#[derive(Debug, Clone)]
pub struct Map {
    pub ty: crate::MapType,
    pub data: std::collections::BTreeMap<crate::Value, crate::Value>,
}

impl PartialEq for Map {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Map {}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

impl Map {
    pub fn new(ty: &crate::MapType) -> Self {
        Self {
            ty: ty.clone(),
            data: std::collections::BTreeMap::new(),
        }
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::sync::Arc::new(self.ty.clone()))
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, crate::Value, crate::Value> {
        self.data.iter()
    }

    pub fn keys(&self) -> std::collections::btree_map::Keys<'_, crate::Value, crate::Value> {
        self.data.keys()
    }

    pub fn values(&self) -> std::collections::btree_map::Values<'_, crate::Value, crate::Value> {
        self.data.values()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &std::collections::BTreeMap<crate::Value, crate::Value> {
        &self.data
    }

    pub fn has(&self, key: &crate::Value) -> bool {
        self.data.contains_key(key)
    }

    pub fn get(&self, key: &crate::Value) -> Option<&crate::Value> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: crate::Value, value: crate::Value) {
        self.data.insert(key, value);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Map {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut ser = s.serialize_map(Some(self.data.len()))?;

        for (k, v) in &self.data {
            ser.serialize_entry(k, v)?;
        }
        ser.end()
    }
}

impl crate::ToType for Map {
    fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::sync::Arc::new(self.ty.clone()))
    }
}

impl crate::ToValue for Map {
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Map(self)
    }
}

impl std::ops::Index<&crate::Value> for Map {
    type Output = crate::Value;

    fn index(&self, index: &crate::Value) -> &Self::Output {
        self.data.get(index).unwrap_or(&crate::Value::UNDEFINED)
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (key, value) in &self.data {
            write!(f, "\n\t{}: {}", key, value)?;
        }
        if !self.data.is_empty() {
            writeln!(f)?;
        }
        write!(f, "}}")
    }
}

impl<K, V> crate::ToValue for std::collections::HashMap<K, V>
where
    K: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
    V: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(crate::DynamicRef::from_object(self))
    }
}

impl<K, V> crate::Object for std::collections::HashMap<K, V>
where
    K: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
    V: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn field_by_ref(&self, name: &str) -> crate::ValueRef<'_> {
        for (k, v) in self {
            if k.to_value_ref().to_string() == name {
                return v.to_value_ref();
            }
        }
        crate::ValueRef::Undefined
    }

    fn entries(&self) -> Option<crate::Map> {
        let mut map = Map::new(self.to_type().as_map().expect("map type"));

        for (k, v) in self {
            map.insert(k.to_value(), v.to_value());
        }

        Some(map)
    }

    fn entries_by_ref(&self) -> Option<Vec<(crate::ValueRef<'_>, crate::ValueRef<'_>)>> {
        Some(self.iter().map(|(k, v)| (k.to_value_ref(), v.to_value_ref())).collect())
    }
}

impl<K, V> crate::ToValue for std::collections::BTreeMap<K, V>
where
    K: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
    V: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value_ref(&self) -> crate::ValueRef<'_> {
        crate::ValueRef::Dynamic(crate::DynamicRef::from_object(self))
    }
}

impl<K, V> crate::Object for std::collections::BTreeMap<K, V>
where
    K: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
    V: std::fmt::Debug + Send + Sync + crate::TypeOf + crate::ToValue + 'static,
{
    fn field_by_ref(&self, name: &str) -> crate::ValueRef<'_> {
        for (k, v) in self {
            if k.to_value_ref().to_string() == name {
                return v.to_value_ref();
            }
        }
        crate::ValueRef::Undefined
    }

    fn entries(&self) -> Option<crate::Map> {
        let mut map = Map::new(self.to_type().as_map().expect("map type"));

        for (k, v) in self {
            map.insert(k.to_value(), v.to_value());
        }

        Some(map)
    }

    fn entries_by_ref(&self) -> Option<Vec<(crate::ValueRef<'_>, crate::ValueRef<'_>)>> {
        Some(self.iter().map(|(k, v)| (k.to_value_ref(), v.to_value_ref())).collect())
    }
}

#[cfg(test)]
mod test {
    use crate::value_of;

    #[test]
    pub fn to_value() {
        let map = btree_map! {
            "hello".to_string() => 123_i32,
            "world".to_string() => 111_i32
        };
        let value = value_of!(map);

        assert!(value.is_map());
        assert_eq!(value.len(), 2);
        assert_eq!(value["hello"], value_of!(123_i32));
    }
}
