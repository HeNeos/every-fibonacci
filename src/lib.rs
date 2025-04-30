use js_sys::BigInt;
use once_cell::sync::Lazy;
use std::str::FromStr;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

const MODULO: u128 = 10_000_000_000_000_000; // 10^16
const MAX_INDEX: u128 = (1u128 << 64) - 1; // 2^64 - 1

#[derive(Clone, Copy, Debug)]
struct FibonacciPair {
    fn_val: u128,   // F(n)
    fnm1_val: u128, // F(n-1)
}

struct FibonacciState {
    current_index: u128,
    current_pair: FibonacciPair,
}

impl FibonacciState {
    fn new() -> Self {
        FibonacciState {
            current_index: 1,
            current_pair: FibonacciPair {
                fn_val: 1,
                fnm1_val: 0,
            },
        }
    }

    fn matrix_power(mut matrix: [u128; 4], mut n: u128) -> [u128; 4] {
        fn matrix_multiply(a: &[u128; 4], b: &[u128; 4]) -> [u128; 4] {
            [
                (a[0]
                    .wrapping_mul(b[0])
                    .wrapping_add(a[1].wrapping_mul(b[2])))
                    % MODULO,
                (a[0]
                    .wrapping_mul(b[1])
                    .wrapping_add(a[1].wrapping_mul(b[3])))
                    % MODULO,
                (a[2]
                    .wrapping_mul(b[0])
                    .wrapping_add(a[3].wrapping_mul(b[2])))
                    % MODULO,
                (a[2]
                    .wrapping_mul(b[1])
                    .wrapping_add(a[3].wrapping_mul(b[3])))
                    % MODULO,
            ]
        }

        let mut result = [1, 0, 0, 1];
        while n > 0 {
            if n % 2 == 1 {
                result = matrix_multiply(&result, &matrix);
            }
            matrix = matrix_multiply(&matrix, &matrix);
            n /= 2;
        }
        result
    }

    fn calculate_pair(&self, n: u128) -> Result<FibonacciPair, String> {
        if n == 0 {
            return Ok(FibonacciPair {
                fn_val: 0,
                fnm1_val: 1,
            });
        }
        if n > MAX_INDEX {
            return Err(format!("Index {} exceeds maximum {}", n, MAX_INDEX));
        }
        let base_matrix = [1, 1, 1, 0];
        let result_matrix = Self::matrix_power(base_matrix, n);
        Ok(FibonacciPair {
            fn_val: result_matrix[1],   // F(n)
            fnm1_val: result_matrix[3], // F(n-1)
        })
    }

    fn set_index(&mut self, n: u128) -> Result<(), String> {
        let pair = self.calculate_pair(n)?;
        self.current_index = n;
        self.current_pair = pair;
        Ok(())
    }

    fn next(&mut self) -> Result<(), String> {
        if self.current_index == MAX_INDEX {
            return Err(format!("Already at maximum index {}", MAX_INDEX));
        }
        let next_fn = (self
            .current_pair
            .fn_val
            .wrapping_add(self.current_pair.fnm1_val))
            % MODULO;
        self.current_pair.fnm1_val = self.current_pair.fn_val;
        self.current_pair.fn_val = next_fn;
        self.current_index = self.current_index.saturating_add(1);
        Ok(())
    }

    fn previous(&mut self) -> Result<(), String> {
        if self.current_index == 0 {
            return Err("Already at index 0".to_string());
        }
        let prev_fnm1 = (self.current_pair.fn_val + MODULO - self.current_pair.fnm1_val) % MODULO;
        self.current_pair.fn_val = self.current_pair.fnm1_val;
        self.current_pair.fnm1_val = prev_fnm1;
        self.current_index = self.current_index.saturating_sub(1);
        Ok(())
    }

    fn get_current_pair_as_string_vec(&self) -> Vec<String> {
        vec![
            self.current_pair.fn_val.to_string(),   // F(n) as String
            self.current_pair.fnm1_val.to_string(), // F(n-1) as String
        ]
    }

    fn get_current_index_as_string(&self) -> String {
        self.current_index.to_string()
    }
}

fn bigint_to_u128(bi: BigInt) -> Result<u128, JsValue> {
    if bi.lt(&BigInt::from(0)) {
        return Err(JsValue::from_str("BigInt index cannot be negative"));
    }
    let max_index_js_string = JsValue::from_str(&MAX_INDEX.to_string());
    let max_index_bigint = BigInt::new(&max_index_js_string).map_err(|e| {
        JsValue::from_str(&format!("Failed to create BigInt for MAX_INDEX: {:?}", e))
    })?;
    if bi.gt(&max_index_bigint) {
        return Err(JsValue::from_str(&format!(
            "BigInt index exceeds maximum {}",
            MAX_INDEX
        )));
    }
    let dec_str = bi
        .to_string(10)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("Failed to convert BigInt to decimal string"))?;
    u128::from_str(&dec_str).map_err(|e| {
        JsValue::from_str(&format!(
            "Failed to parse decimal BigInt string '{}': {}",
            dec_str, e
        ))
    })
}

static GENERATOR: Lazy<Mutex<FibonacciState>> = Lazy::new(|| Mutex::new(FibonacciState::new()));

#[wasm_bindgen]
pub fn init_pair(n_bigint: BigInt) -> Result<Vec<String>, JsValue> {
    let n = bigint_to_u128(n_bigint)?;
    let mut generator = GENERATOR
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Mutex lock failed: {}", e)))?;
    generator.set_index(n).map_err(|e| JsValue::from_str(&e))?;
    Ok(generator.get_current_pair_as_string_vec()) // Use new method
}

#[wasm_bindgen]
pub fn next_pair() -> Result<Vec<String>, JsValue> {
    let mut generator = GENERATOR
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Mutex lock failed: {}", e)))?;
    generator.next().map_err(|e| JsValue::from_str(&e))?;
    Ok(generator.get_current_pair_as_string_vec()) // Use new method
}

#[wasm_bindgen]
pub fn prev_pair() -> Result<Vec<String>, JsValue> {
    let mut generator = GENERATOR
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Mutex lock failed: {}", e)))?;
    generator.previous().map_err(|e| JsValue::from_str(&e))?;
    Ok(generator.get_current_pair_as_string_vec()) // Use new method
}

#[wasm_bindgen]
pub fn get_current_index() -> Result<String, JsValue> {
    let generator = GENERATOR
        .lock()
        .map_err(|e| JsValue::from_str(&format!("Mutex lock failed: {}", e)))?;
    Ok(generator.get_current_index_as_string())
}
