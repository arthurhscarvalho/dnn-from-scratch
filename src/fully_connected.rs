use ndarray::{Array1, Array2, Axis};
use rand::thread_rng;
use rand_distr::{Distribution, Normal};

/// Generate weights following the HE-Initialization method
fn generate_weights(input_size: usize, output_size: usize) -> Array2<f64> {
    let scale = (2.0 / input_size as f64).sqrt();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut rng = thread_rng();
    Array2::from_shape_fn((input_size, output_size), |_| {
        normal.sample(&mut rng) * scale
    })
}

pub struct FullyConnected<'a> {
    // Weights and biases
    weights: Array2<f64>,
    biases: Array1<f64>,
    // Activation-related
    activation: &'a str,
    // Optimizer-related (Adam)
    m_weights: Array2<f64>,
    v_weights: Array2<f64>,
    m_biases: Array1<f64>,
    v_biases: Array1<f64>,
    // Hyper-parameters (Adam)
    beta_1: f64,
    beta_2: f64,
    epsilon: f64,
    // Input and Output (for the forward pass)
    input: Array2<f64>,
    output: Array2<f64>,
}

impl FullyConnected<'_> {
    /// Initialize a fully-connected layer
    pub fn new<'a>(input_size: usize, output_size: usize, activation: &'a str) -> FullyConnected {
        FullyConnected {
            weights: generate_weights(input_size, output_size),
            biases: Array1::zeros(input_size),
            activation,
            m_weights: Array2::zeros((input_size, output_size)),
            v_weights: Array2::zeros((input_size, output_size)),
            m_biases: Array1::zeros(input_size),
            v_biases: Array1::zeros(input_size),
            beta_1: 0.9,
            beta_2: 0.999,
            epsilon: 1e-8,
            input: Array2::zeros((1, 1)),
            output: Array2::zeros((1, 1)),
        }
    }

    /// Forward pass
    pub fn forward(&mut self, x: Array2<f64>) -> Array2<f64> {
        self.input = x;
        let mut z = self.input.dot(&self.weights);
        // Add biases to each row
        for mut row in z.rows_mut() {
            row += &self.biases;
        }
        match self.activation {
            "relu" => {
                self.output = z.mapv(|v| v.max(0.0));
            }
            "softmax" => {
                unimplemented!("softmax not yet implemented.")
            }
            _ => {
                panic!("Invalid activation function passed. Use either relu or softmax.")
            }
        }
        self.output.clone()
    }

    /// Backpropagation
    pub fn backward(&mut self, d_values: Array2<f64>, learning_rate: f64, t: i32) -> Array2<f64> {
        let mut d_values = d_values.clone();
        // Calculate the derivative of the activation function
        match self.activation {
            "relu" => {
                d_values *= &self.output.mapv(|x| if x > 0.0 { 1.0 } else { 0.0 });
            }
            "softmax" => {
                unimplemented!("softmax not yet implemented.")
            }
            _ => {
                panic!("Invalid activation function passed. Use either relu or softmax.")
            }
        }
        // Calculate the derivative with respect to weights and biases
        let d_weights = self.output.t().dot(&d_values);
        let d_biases = d_values.sum_axis(Axis(0));
        // Clip derivatives to avoid extreme values
        let d_weights_clipped = d_weights.mapv(|x| x.max(-1.0).min(1.0));
        let d_biases_clipped = d_biases.mapv(|x| x.max(-1.0).min(1.0));
        // Calculate gradient with respect to the input
        let d_inputs = d_values.dot(&self.weights.t());
        // Update weights and biases using gradient descent
        self.weights -= &(&d_weights_clipped * learning_rate);
        self.biases -= &(&d_biases_clipped * learning_rate);
        // Adam optimizer for weights
        self.m_weights =
            &(&self.m_weights * self.beta_1) + &(&d_weights_clipped * (1.0 - self.beta_1));
        self.v_weights = &(&self.v_weights * self.beta_2)
            + &(&d_weights_clipped.mapv(|x| x * x) * (1.0 - self.beta_2));
        let m_hat_weights = &self.m_weights / (1.0 - self.beta_1.powi(t));
        let v_hat_weights = &self.v_weights / (1.0 - self.beta_2.powi(t));
        self.weights -=
            &(learning_rate * &m_hat_weights / &v_hat_weights.mapv(|x| x.sqrt() + self.epsilon));
        // Adam oxtimizer for biases
        self.m_biases =
            &(&self.m_biases * self.beta_1) + &(&d_biases_clipped * (1.0 - self.beta_1));
        self.v_biases = &(&self.v_biases * self.beta_2)
            + &(&d_biases_clipped.mapv(|x| x * x) * (1.0 - self.beta_2));
        let m_hat_biases = &self.m_biases / (1.0 - self.beta_1.powi(t));
        let v_hat_biases = &self.v_biases / (1.0 - self.beta_2.powi(t));
        self.biases -=
            &(learning_rate * &m_hat_biases / &v_hat_biases.mapv(|x| x.sqrt() + self.epsilon));
        d_inputs
    }
}
