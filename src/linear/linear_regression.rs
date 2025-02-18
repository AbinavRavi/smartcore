//! # Linear Regression
//!
//! Linear regression is a very straightforward approach for predicting a quantitative response \\(y\\) on the basis of a linear combination of explanatory variables \\(X\\).
//! Linear regression assumes that there is approximately a linear relationship between \\(X\\) and \\(y\\). Formally, we can write this linear relationship as
//!
//! \\[y \approx \beta_0 + \sum_{i=1}^n \beta_iX_i + \epsilon\\]
//!
//! where \\(\epsilon\\) is a mean-zero random error term and the regression coefficients \\(\beta_0, \beta_0, ... \beta_n\\) are unknown, and must be estimated.
//!
//! While regression coefficients can be estimated directly by solving
//!
//! \\[\hat{\beta} = (X^TX)^{-1}X^Ty \\]
//!
//! the \\((X^TX)^{-1}\\) term is both computationally expensive and numerically unstable. An alternative approach is to use a matrix decomposition to avoid this operation.
//! SmartCore uses [SVD](../../linalg/svd/index.html) and [QR](../../linalg/qr/index.html) matrix decomposition to find estimates of \\(\hat{\beta}\\).
//! The QR decomposition is more computationally efficient and more numerically stable than calculating the normal equation directly,
//! but does not work for all data matrices. Unlike the QR decomposition, all matrices have an SVD decomposition.
//!
//! Example:
//!
//! ```
//! use smartcore::linalg::naive::dense_matrix::*;
//! use smartcore::linear::linear_regression::*;
//!
//! // Longley dataset (https://www.statsmodels.org/stable/datasets/generated/longley.html)
//! let x = DenseMatrix::from_2d_array(&[
//!               &[234.289, 235.6, 159.0, 107.608, 1947., 60.323],
//!               &[259.426, 232.5, 145.6, 108.632, 1948., 61.122],
//!               &[258.054, 368.2, 161.6, 109.773, 1949., 60.171],
//!               &[284.599, 335.1, 165.0, 110.929, 1950., 61.187],
//!               &[328.975, 209.9, 309.9, 112.075, 1951., 63.221],
//!               &[346.999, 193.2, 359.4, 113.270, 1952., 63.639],
//!               &[365.385, 187.0, 354.7, 115.094, 1953., 64.989],
//!               &[363.112, 357.8, 335.0, 116.219, 1954., 63.761],
//!               &[397.469, 290.4, 304.8, 117.388, 1955., 66.019],
//!               &[419.180, 282.2, 285.7, 118.734, 1956., 67.857],
//!               &[442.769, 293.6, 279.8, 120.445, 1957., 68.169],
//!               &[444.546, 468.1, 263.7, 121.950, 1958., 66.513],
//!               &[482.704, 381.3, 255.2, 123.366, 1959., 68.655],
//!               &[502.601, 393.1, 251.4, 125.368, 1960., 69.564],
//!               &[518.173, 480.6, 257.2, 127.852, 1961., 69.331],
//!               &[554.894, 400.7, 282.7, 130.081, 1962., 70.551],
//!          ]);
//!
//! let y: Vec<f64> = vec![83.0, 88.5, 88.2, 89.5, 96.2, 98.1, 99.0,
//!           100.0, 101.2, 104.6, 108.4, 110.8, 112.6, 114.2, 115.7, 116.9];
//!
//! let lr = LinearRegression::fit(&x, &y,
//!             LinearRegressionParameters::default().
//!             with_solver(LinearRegressionSolverName::QR)).unwrap();
//!
//! let y_hat = lr.predict(&x).unwrap();
//! ```
//!
//! ## References:
//!
//! * ["Pattern Recognition and Machine Learning", C.M. Bishop, Linear Models for Regression](https://www.microsoft.com/en-us/research/uploads/prod/2006/01/Bishop-Pattern-Recognition-and-Machine-Learning-2006.pdf)
//! * ["An Introduction to Statistical Learning", James G., Witten D., Hastie T., Tibshirani R., 3. Linear Regression](http://faculty.marshall.usc.edu/gareth-james/ISL/)
//! * ["Numerical Recipes: The Art of Scientific Computing",  Press W.H., Teukolsky S.A., Vetterling W.T, Flannery B.P, 3rd ed., Section 15.4 General Linear Least Squares](http://numerical.recipes/)
//!
//! <script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
//! <script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::api::{Predictor, SupervisedEstimator};
use crate::error::Failed;
use crate::linalg::Matrix;
use crate::math::num::RealNumber;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
/// Approach to use for estimation of regression coefficients. QR is more efficient but SVD is more stable.
pub enum LinearRegressionSolverName {
    /// QR decomposition, see [QR](../../linalg/qr/index.html)
    QR,
    /// SVD decomposition, see [SVD](../../linalg/svd/index.html)
    SVD,
}

/// Linear Regression parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct LinearRegressionParameters {
    /// Solver to use for estimation of regression coefficients.
    pub solver: LinearRegressionSolverName,
}

/// Linear Regression
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct LinearRegression<T: RealNumber, M: Matrix<T>> {
    coefficients: M,
    intercept: T,
    _solver: LinearRegressionSolverName,
}

impl LinearRegressionParameters {
    /// Solver to use for estimation of regression coefficients.
    pub fn with_solver(mut self, solver: LinearRegressionSolverName) -> Self {
        self.solver = solver;
        self
    }
}

impl Default for LinearRegressionParameters {
    fn default() -> Self {
        LinearRegressionParameters {
            solver: LinearRegressionSolverName::SVD,
        }
    }
}

/// Linear Regression grid search parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct LinearRegressionSearchParameters {
    /// Solver to use for estimation of regression coefficients.
    pub solver: Vec<LinearRegressionSolverName>,
}

/// Linear Regression grid search iterator
pub struct LinearRegressionSearchParametersIterator {
    linear_regression_search_parameters: LinearRegressionSearchParameters,
    current_solver: usize,
}

impl IntoIterator for LinearRegressionSearchParameters {
    type Item = LinearRegressionParameters;
    type IntoIter = LinearRegressionSearchParametersIterator;

    fn into_iter(self) -> Self::IntoIter {
        LinearRegressionSearchParametersIterator {
            linear_regression_search_parameters: self,
            current_solver: 0,
        }
    }
}

impl Iterator for LinearRegressionSearchParametersIterator {
    type Item = LinearRegressionParameters;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_solver == self.linear_regression_search_parameters.solver.len() {
            return None;
        }

        let next = LinearRegressionParameters {
            solver: self.linear_regression_search_parameters.solver[self.current_solver].clone(),
        };

        self.current_solver += 1;

        Some(next)
    }
}

impl Default for LinearRegressionSearchParameters {
    fn default() -> Self {
        let default_params = LinearRegressionParameters::default();

        LinearRegressionSearchParameters {
            solver: vec![default_params.solver],
        }
    }
}

impl<T: RealNumber, M: Matrix<T>> PartialEq for LinearRegression<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.coefficients == other.coefficients
            && (self.intercept - other.intercept).abs() <= T::epsilon()
    }
}

impl<T: RealNumber, M: Matrix<T>> SupervisedEstimator<M, M::RowVector, LinearRegressionParameters>
    for LinearRegression<T, M>
{
    fn fit(
        x: &M,
        y: &M::RowVector,
        parameters: LinearRegressionParameters,
    ) -> Result<Self, Failed> {
        LinearRegression::fit(x, y, parameters)
    }
}

impl<T: RealNumber, M: Matrix<T>> Predictor<M, M::RowVector> for LinearRegression<T, M> {
    fn predict(&self, x: &M) -> Result<M::RowVector, Failed> {
        self.predict(x)
    }
}

impl<T: RealNumber, M: Matrix<T>> LinearRegression<T, M> {
    /// Fits Linear Regression to your data.
    /// * `x` - _NxM_ matrix with _N_ observations and _M_ features in each observation.
    /// * `y` - target values
    /// * `parameters` - other parameters, use `Default::default()` to set parameters to default values.
    pub fn fit(
        x: &M,
        y: &M::RowVector,
        parameters: LinearRegressionParameters,
    ) -> Result<LinearRegression<T, M>, Failed> {
        let y_m = M::from_row_vector(y.clone());
        let b = y_m.transpose();
        let (x_nrows, num_attributes) = x.shape();
        let (y_nrows, _) = b.shape();

        if x_nrows != y_nrows {
            return Err(Failed::fit(
                "Number of rows of X doesn\'t match number of rows of Y",
            ));
        }

        let a = x.h_stack(&M::ones(x_nrows, 1));

        let w = match parameters.solver {
            LinearRegressionSolverName::QR => a.qr_solve_mut(b)?,
            LinearRegressionSolverName::SVD => a.svd_solve_mut(b)?,
        };

        let wights = w.slice(0..num_attributes, 0..1);

        Ok(LinearRegression {
            intercept: w.get(num_attributes, 0),
            coefficients: wights,
            _solver: parameters.solver,
        })
    }

    /// Predict target values from `x`
    /// * `x` - _KxM_ data where _K_ is number of observations and _M_ is number of features.
    pub fn predict(&self, x: &M) -> Result<M::RowVector, Failed> {
        let (nrows, _) = x.shape();
        let mut y_hat = x.matmul(&self.coefficients);
        y_hat.add_mut(&M::fill(nrows, 1, self.intercept));
        Ok(y_hat.transpose().to_row_vector())
    }

    /// Get estimates regression coefficients
    pub fn coefficients(&self) -> &M {
        &self.coefficients
    }

    /// Get estimate of intercept
    pub fn intercept(&self) -> T {
        self.intercept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::naive::dense_matrix::*;

    #[test]
    fn search_parameters() {
        let parameters = LinearRegressionSearchParameters {
            solver: vec![
                LinearRegressionSolverName::QR,
                LinearRegressionSolverName::SVD,
            ],
        };
        let mut iter = parameters.into_iter();
        assert_eq!(iter.next().unwrap().solver, LinearRegressionSolverName::QR);
        assert_eq!(iter.next().unwrap().solver, LinearRegressionSolverName::SVD);
        assert!(iter.next().is_none());
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn ols_fit_predict() {
        let x = DenseMatrix::from_2d_array(&[
            &[234.289, 235.6, 159.0, 107.608, 1947., 60.323],
            &[259.426, 232.5, 145.6, 108.632, 1948., 61.122],
            &[258.054, 368.2, 161.6, 109.773, 1949., 60.171],
            &[284.599, 335.1, 165.0, 110.929, 1950., 61.187],
            &[328.975, 209.9, 309.9, 112.075, 1951., 63.221],
            &[346.999, 193.2, 359.4, 113.270, 1952., 63.639],
            &[365.385, 187.0, 354.7, 115.094, 1953., 64.989],
            &[363.112, 357.8, 335.0, 116.219, 1954., 63.761],
            &[397.469, 290.4, 304.8, 117.388, 1955., 66.019],
            &[419.180, 282.2, 285.7, 118.734, 1956., 67.857],
            &[442.769, 293.6, 279.8, 120.445, 1957., 68.169],
            &[444.546, 468.1, 263.7, 121.950, 1958., 66.513],
            &[482.704, 381.3, 255.2, 123.366, 1959., 68.655],
            &[502.601, 393.1, 251.4, 125.368, 1960., 69.564],
            &[518.173, 480.6, 257.2, 127.852, 1961., 69.331],
            &[554.894, 400.7, 282.7, 130.081, 1962., 70.551],
        ]);

        let y: Vec<f64> = vec![
            83.0, 88.5, 88.2, 89.5, 96.2, 98.1, 99.0, 100.0, 101.2, 104.6, 108.4, 110.8, 112.6,
            114.2, 115.7, 116.9,
        ];

        let y_hat_qr = LinearRegression::fit(
            &x,
            &y,
            LinearRegressionParameters {
                solver: LinearRegressionSolverName::QR,
            },
        )
        .and_then(|lr| lr.predict(&x))
        .unwrap();

        let y_hat_svd = LinearRegression::fit(&x, &y, Default::default())
            .and_then(|lr| lr.predict(&x))
            .unwrap();

        assert!(y
            .iter()
            .zip(y_hat_qr.iter())
            .all(|(&a, &b)| (a - b).abs() <= 5.0));
        assert!(y
            .iter()
            .zip(y_hat_svd.iter())
            .all(|(&a, &b)| (a - b).abs() <= 5.0));
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    #[cfg(feature = "serde")]
    fn serde() {
        let x = DenseMatrix::from_2d_array(&[
            &[234.289, 235.6, 159.0, 107.608, 1947., 60.323],
            &[259.426, 232.5, 145.6, 108.632, 1948., 61.122],
            &[258.054, 368.2, 161.6, 109.773, 1949., 60.171],
            &[284.599, 335.1, 165.0, 110.929, 1950., 61.187],
            &[328.975, 209.9, 309.9, 112.075, 1951., 63.221],
            &[346.999, 193.2, 359.4, 113.270, 1952., 63.639],
            &[365.385, 187.0, 354.7, 115.094, 1953., 64.989],
            &[363.112, 357.8, 335.0, 116.219, 1954., 63.761],
            &[397.469, 290.4, 304.8, 117.388, 1955., 66.019],
            &[419.180, 282.2, 285.7, 118.734, 1956., 67.857],
            &[442.769, 293.6, 279.8, 120.445, 1957., 68.169],
            &[444.546, 468.1, 263.7, 121.950, 1958., 66.513],
            &[482.704, 381.3, 255.2, 123.366, 1959., 68.655],
            &[502.601, 393.1, 251.4, 125.368, 1960., 69.564],
            &[518.173, 480.6, 257.2, 127.852, 1961., 69.331],
            &[554.894, 400.7, 282.7, 130.081, 1962., 70.551],
        ]);

        let y = vec![
            83.0, 88.5, 88.2, 89.5, 96.2, 98.1, 99.0, 100.0, 101.2, 104.6, 108.4, 110.8, 112.6,
            114.2, 115.7, 116.9,
        ];

        let lr = LinearRegression::fit(&x, &y, Default::default()).unwrap();

        let deserialized_lr: LinearRegression<f64, DenseMatrix<f64>> =
            serde_json::from_str(&serde_json::to_string(&lr).unwrap()).unwrap();

        assert_eq!(lr, deserialized_lr);
    }
}
