// #![allow(dead_code)]
#![allow(unused_imports)]

use std::fmt;
use std::ops::{Sub};
use crate::norms;

#[derive(Clone, Debug)]
pub enum InvalidAccessPolicy {
    ReturnNone,
    ReturnClosest
}

#[derive(Clone, Debug)]
pub enum ValueType {
    Countable,   // extensive quantity
    NonCountable // intensive quantity
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
    I: std::cmp::PartialOrd + std::fmt::Debug
{

    pub fn new() -> DataSeries<I, V> {
        let index: Vec<I> = Vec::new();
        let values: Vec<V> = Vec::new();
        let invalid_access_policy = InvalidAccessPolicy::ReturnNone;
        DataSeries { index , values, invalid_access_policy}
    }

    pub fn push(&mut self, index: I, value: V) -> bool
    {
        if let Some(index_val) = self.index.last() {
            if index_val >= &index {return false;}
        }
        self.index.push(index);
        self.values.push(value);
        true
    }

    pub fn push_if_different(&mut self, index: I, value: V, tolerance: f32) -> bool
    where
        V: norms::L1
    {
        if let Some(val_last) =  self.values.last() {
            let diff = norms::L1::compute(&value, val_last);
            if diff < tolerance && diff > -tolerance { return false;}
        }
        self.push(index, value)
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

    fn get_projection<J>(&self, index_new: &Vec<I>, value_type: ValueType) -> DataSeries<I, V>
    where
        I: std::ops::Sub<Output = J> + Copy, // index_a<I> - index_b<I> = interval<J>
        J: std::ops::Div<Output = f32>,      // interval_a<J> / interval_b<J> = fraction<f32>
        V: std::ops::Mul<f32, Output = V> + std::ops::Add<Output = V> + Copy
    {
        let mut axis: Vec<I> = index_new.to_vec();
        let mut values: Vec<V> = Vec::new();
        values.reserve(axis.len());

        assert_eq!(self.index.len(), self.values.len());

        // If new index intersects interval in old axis, extend it to match
        let mut is_index_new_extended = false;
        if let Some(&last_idx_new) = axis.last() {
            let interval_old_last_ = self.index.windows(2).enumerate().find(|x| {
                x.1[0] <= last_idx_new && x.1[1] > last_idx_new
            });
            if let Some(interval_old_last) = interval_old_last_ {
                axis.push(interval_old_last.1[1]);
                is_index_new_extended = true;
            }
        }
        
        // let mut i_o_last = 0;
        for (i_n, interval_new) in axis.windows(2).enumerate() {
            // i_o_last = if i_o_last > 0 {i_o_last - 1} else {0};
            for (i_o, interval_old) in self.index.windows(2).enumerate() {
                // Intervals overlapping?
                if interval_old[1] <= interval_new[0] || interval_old[0] >= interval_new[1] {
                    continue;
                }
                // i_o_last = i_o;
                // Determnine left and right boundary of overlapping interval
                let boundary_left = if interval_old[0] > interval_new[0]  {
                   &interval_old[0] 
                } else {
                    &interval_new[0]
                };
                let boundary_right = if interval_old[1] > interval_new[1]  {
                    &interval_new[1] 
                 } else {
                     &interval_old[1]
                 };
              
                let compute_value_to_add = | interval: &[I]| -> V {
                    let interval_len = interval[1] - interval[0]; 
                    let value_old = &self.values[i_o];
                    let frac = (*boundary_right - *boundary_left) / interval_len;
                    *value_old * frac
                };

                let value_to_add = match value_type {
                    ValueType::Countable => {
                        compute_value_to_add(interval_old)
                    },
                    ValueType::NonCountable => {
                        compute_value_to_add(interval_new)
                    }
                };
                if i_n >= values.len() {
                    values.push(value_to_add);
                } else {
                    values[i_n] = values[i_n] + value_to_add;
                }
            }
        }
        if is_index_new_extended { axis.pop(); }
        
        let result : DataSeries<I, V> = DataSeries {
            index: axis,
            values: values,
            invalid_access_policy: self.invalid_access_policy.clone()
        };
        return result
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
    use crate::data_series::ValueType;

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
        assert!(!ds.push_if_different(10, 4.1, 1.));
        assert!(ds.push_if_different(10, 6., 1.));
        assert!(ds.push_if_different(11, 4., 1.));
    }

    #[test]
    fn test_get_projection_countable() {
        let mut ds: DataSeries<f32, f32> = DataSeries::new();
        ds.push(1., 2.);
        ds.push(3., 3.);
        ds.push(5., 7.);
        ds.push(10., 0.);
        let index_new: Vec<f32> = vec![1.,2., 3., 4., 5.];
        let proj = ds.get_projection(&index_new, ValueType::Countable);
        assert_eq!(proj.index, vec![1.,2., 3., 4., 5.]);
        assert_eq!(proj.values, vec![1.,1., 1.5, 1.5, 7.]);

        let index_new: Vec<f32> = vec![1.,5., 6.];
        let proj = ds.get_projection(&index_new, ValueType::Countable);
        assert_eq!(proj.index, vec![1.,5., 6.]);
        assert_eq!(proj.values, vec![5., 1.4, 5.6]);
        
    }


    #[test]

    fn test_get_projection_non_countable() {
        let mut ds: DataSeries<f32, f32> = DataSeries::new();
        ds.push(1., 2.);
        ds.push(3., 3.);
        ds.push(5., 7.);
        ds.push(10., 0.);
        let index_new: Vec<f32> = vec![1.,2., 3., 4., 5.];
        let proj = ds.get_projection(&index_new, ValueType::NonCountable);
        assert_eq!(proj.index, vec![1.,2., 3., 4., 5.]);
        assert_eq!(proj.values, vec![2.,2., 3., 3., 7.]);

        let index_new: Vec<f32> = vec![1.,5., 6.];
        let proj = ds.get_projection(&index_new, ValueType::NonCountable);
        assert_eq!(proj.index, vec![1.,5., 6.]);
        assert_eq!(proj.values, vec![2.5, 7., 7.]);
    }

}

