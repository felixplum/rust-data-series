// #![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt;
use std::ops::{Sub};
use crate::norms;

pub enum InvalidAccessPolicy {
    ReturnNone,
    ReturnClosest
}
pub struct DataSeries<I, V> {
    index: Vec<I>,
    values: Vec<V>,
    invalid_access_policy: InvalidAccessPolicy
}

impl <I,V> fmt::Display for DataSeries<I, V>
where
    I: std::fmt::Debug,
    V: std::fmt::Debug
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let zipped: Vec<(&I, &V)> = self.index.iter().zip(self.values.iter()).collect();
        write!(f, "{:?}", zipped)
    }
}

impl<I, V> DataSeries<I, V>
where
    I: std::cmp::PartialOrd
{

    pub fn new() -> DataSeries<I, V> {
        let index: Vec<I> = Vec::new();
        let values: Vec<V> = Vec::new();
        let invalid_access_policy = InvalidAccessPolicy::ReturnNone;
        DataSeries { index , values, invalid_access_policy}
    }

    pub fn push(&mut self, index: I, value: V) -> bool
    {
        let mut push_data = true;
        match self.index.last() {
            Some(index_val) => {
                push_data = index_val < &index;
            },
            None => ()
        }
        if push_data {
            self.index.push(index);
            self.values.push(value);
        }
        return push_data;
    }

    pub fn push_if_different(&mut self, index: I, value: V, tolerance: f32) -> bool
    where
        V: norms::L1
    {
        let mut push_data = true;
        match self.values.last() {
            Some(val_last) => {
                let diff = norms::L1::compute(&value, val_last);
                if diff > 0. {
                    push_data = diff >= tolerance 
                } else {
                    push_data = diff <= tolerance 
                }
            },
            None => ()
        }
        if push_data {
            return self.push(index, value);
        } else {
            return false;
        }
    }

    pub fn set_invalid_access_policy(&mut self, policy: InvalidAccessPolicy) {
        self.invalid_access_policy = policy;
    }

    pub fn as_arrays(&self) -> (&Vec<I>, &Vec<V>) {
        return (&self.index, &self.values);
    }

    // fn as_uniform_arrays(&self, ) -> (Vec<I>, Vec<V>) {
    //     return (&self.index, &self.values);
    // }

    fn as_projection(&self, new_axis: &Vec<I>) -> (Vec<I>, Vec<V>) {
        let mut axis: Vec<I> = Vec::new();
        let mut values: Vec<V> = Vec::new();
        axis.reserve(new_axis.len());
        values.reserve(new_axis.len());

        for idx in new_axis.iter() {
            // find value in old axis, such that idx
        }
        return (axis, values);
    }

    pub fn at(&self, index: &I) -> Option<&V> {
        
        assert_eq!(self.index.len(), self.values.len());
        for i in 0..self.index.len()-1 {
            if index >= &self.index[i] && index < &self.index[i+1] {
                return Some(&self.values[i]);
            }
        }

        if self.index.last().is_some() {
            if index == self.index.last().unwrap() {
                return self.values.last();
            }            
        }

        match self.invalid_access_policy {
            InvalidAccessPolicy::ReturnClosest => {
                if self.index.last().is_some() {
                    if index > self.index.last().unwrap() {
                        return self.values.last();
                    }
                } 
                if self.index.first().is_some()  {
                    if index < self.index.first().unwrap() {
                        return self.values.first();
                    }
                }
            },
            InvalidAccessPolicy::ReturnNone => return None
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::data_series::DataSeries;
    use crate::data_series::InvalidAccessPolicy;

    fn create_dataseries() -> DataSeries<u32, f32> {
        let mut ds: DataSeries<u32, f32> = DataSeries::new();
        assert!(ds.push(1, 2.));
        assert!(ds.push(3, 5.));
        assert_eq!(ds.index.len(), ds.values.len());
        assert_eq!(ds.index.len(), 2);
        return ds;
    }

    #[test]
    fn test_push() {
        let mut ds = create_dataseries();
        assert_eq!(ds.at(&1).unwrap(), &2.);
        assert_eq!(ds.at(&3).unwrap(), &5.);
        assert!(!ds.push(3, 5.));
    }

    #[test]
    fn test_at() {
        let mut ds = create_dataseries();
        ds.set_invalid_access_policy(InvalidAccessPolicy::ReturnClosest);
        assert_eq!(ds.at(&0).unwrap(), &2.);
        assert_eq!(ds.at(&1).unwrap(), &2.);
        assert_eq!(ds.at(&2).unwrap(), &2.);
        assert_eq!(ds.at(&4).unwrap(), &5.);
        ds.set_invalid_access_policy(InvalidAccessPolicy::ReturnNone);
        assert_eq!(ds.at(&4).is_none(), true);

    }

    #[test]
    fn test_as_arrays() {
        let ds = create_dataseries();
        let (idx, vals) = ds.as_arrays();
        assert!(idx == &vec![1, 3]);
        assert!(vals == &vec![2., 5.]);

    }

    #[test]
    fn test_push_if_different() {
        let mut ds = create_dataseries();
        assert!(!ds.push_if_different(10, 5.9, 1.));
        assert!(ds.push_if_different(10, 6., 1.));
    }

}
