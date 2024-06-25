use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone)]
pub struct Bucket<K, V> {
    max_size: usize,
    keys: VecDeque<K>,
    values: HashMap<K, V>
}

impl<K, V> Bucket<K, V>
where
    K: Eq + std::hash::Hash + Clone
{
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            keys: VecDeque::new(),
            values: HashMap::new()
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.values.get(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.keys.retain(|k| k != key);

        self.values.remove(key)
    }

    pub fn push(&mut self, key: K, value: V) {
        if self.keys.contains(&key) {
            self.keys.retain(|k| k != &key);
        }

        if self.values.contains_key(&key) {
            self.values.remove(&key);
        }

        while self.keys.len() >= self.max_size {
            match self.keys.pop_back() {
                Some(key) => {
                    self.values.remove(&key);
                }

                None => break
            }
        }

        self.keys.push_front(key.clone());

        self.values.insert(key, value);
    }
}

impl<K, V> IntoIterator for Bucket<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone
{
    type Item = (K, V);
    type IntoIter = BucketIterator<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        BucketIterator {
            keys: self.keys.clone(),
            values: self.values.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct BucketIterator<K, V> {
    keys: VecDeque<K>,
    values: HashMap<K, V>
}

impl<K, V> Iterator for BucketIterator<K, V>
where
    K: Eq + std::hash::Hash
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(key) = self.keys.pop_front() {
            if let Some(value) = self.values.remove(&key) {
                return Some((key, value));
            }
        }

        None
    }
}

mod tests {
    #[test]
    fn push() {
        use super::Bucket;

        let mut bucket = Bucket::new(3);

        bucket.push(1, 1);
        bucket.push(2, 2);
        bucket.push(3, 3);

        let items = bucket.into_iter()
            .map(|(k, _)| k)
            .collect::<Vec<_>>();

        assert_eq!(items, [3, 2, 1]);
    }

    #[test]
    fn order() {
        use super::Bucket;

        let mut bucket = Bucket::new(3);

        bucket.push(1, 1);
        bucket.push(2, 2);
        bucket.push(3, 3);

        bucket.push(2, 2);
        bucket.push(1, 1);

        let items = bucket.into_iter()
            .map(|(k, _)| k)
            .collect::<Vec<_>>();

        assert_eq!(items, [1, 2, 3]);
    }

    #[test]
    fn remove() {
        use super::Bucket;

        let mut bucket = Bucket::new(3);

        bucket.push(1, 1);
        bucket.push(2, 2);
        bucket.push(3, 3);

        bucket.remove(&1);
        bucket.remove(&2);

        assert_eq!(bucket.get(&1), None);
        assert_eq!(bucket.get(&2), None);
        assert_eq!(bucket.get(&3), Some(&3));
    }
}
