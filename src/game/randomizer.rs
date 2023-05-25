struct ValueWeight {
    value: usize,
    weight: usize,
}

impl ValueWeight {
    fn new(value: usize) -> Self {
        Self { value, weight: 0 }
    }
}

pub struct WeightedRandomizer {
    value_weight_vec: Vec<ValueWeight>,
    indexed: bool,
    total_weight: usize,
}
const MAX_VALUE_SEPARATION: usize = 25;

impl Default for WeightedRandomizer {
    fn default() -> Self {
        Self {
            value_weight_vec: vec![],
            indexed: true,
            total_weight: 0,
        }
    }
}

impl WeightedRandomizer {
    fn evenly_distributed_random(max_value: usize) -> usize {
        if max_value == usize::MAX {
            return rand::random::<usize>();
        }
        let get_value_below = max_value + 1;
        // if power of 2
        if get_value_below & (get_value_below - 1) == 0 {
            return rand::random::<usize>() % get_value_below;
        } else {
            let remainder = usize::MAX % get_value_below;
            let threshold = usize::MAX - remainder;
            loop {
                let candidate = rand::random::<usize>();
                if candidate < threshold {
                    return candidate % get_value_below;
                }
            }
        }
    }

    pub fn weighted_random(&self) -> Option<usize> {
        if self.total_weight == 0 {
            return None;
        }
        let random_num = Self::evenly_distributed_random(self.total_weight - 1);
        let mut running_sum = 0;
        for var in self.value_weight_vec.iter() {
            running_sum += var.weight;
            if random_num < running_sum {
                return Some(var.value);
            }
        }
        unreachable!("random_num: {}, running_sum: {}", random_num, running_sum);
    }

    fn true_find(&mut self, value: usize) -> usize {
        if self.indexed {
            if value < self.value_weight_vec.len() {
                return value;
            } else if value < self.value_weight_vec.len() + MAX_VALUE_SEPARATION {
                for idx in self.value_weight_vec.len()..=value {
                    self.value_weight_vec.push(ValueWeight::new(idx));
                }
                return value;
            } else {
                self.indexed = false;
                self.value_weight_vec.push(ValueWeight::new(value));
                return self.value_weight_vec.len() - 1;
            }
        } else {
            for idx in 0..self.value_weight_vec.len() {
                if self.value_weight_vec[idx].value == value {
                    return idx;
                } else if self.value_weight_vec[idx].value > value {
                    self.value_weight_vec.insert(idx, ValueWeight::new(value));
                    return idx;
                }
            }
            self.value_weight_vec.push(ValueWeight::new(value));
            return self.value_weight_vec.len() - 1;
        }
    }

    pub fn set_weight(&mut self, value: usize, new_weight: usize) {
        let idx = self.true_find(value);
        self.total_weight += new_weight - self.value_weight_vec[idx].weight;
        self.value_weight_vec[idx].weight = new_weight;
    }

    pub fn add_to_weight(&mut self, value: usize, weight_to_add: usize) {
        let idx = self.true_find(value);
        self.total_weight += weight_to_add;
        self.value_weight_vec[idx].weight += weight_to_add;
    }

    pub fn subtract_from_weight(&mut self, value: usize, weight_to_subtract: usize) {
        let idx = self.true_find(value);
        let capped_weight_to_subtract = std::cmp::min(weight_to_subtract, self.value_weight_vec[idx].weight);
        self.total_weight -= capped_weight_to_subtract;
        self.value_weight_vec[idx].weight -= capped_weight_to_subtract;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_randomizer() {
        let mut wr = WeightedRandomizer::default();
        wr.set_weight(0, 4);
        assert_eq!(wr.weighted_random(), Some(0));
        wr.set_weight(1, 1);
        for i in 0..100 {
            let wr_num = wr.weighted_random();
            wr.set_weight(i + 2, rand::random::<usize>() % 100);
            assert!(wr_num.is_some() && wr_num.expect("") < i + 2);
        }
    }
}
