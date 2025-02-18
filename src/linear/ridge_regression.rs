//! # Ridge Regression
//!
//! [Linear regression](../linear_regression/index.html) is the standard algorithm for predicting a quantitative response \\(y\\) on the basis of a linear combination of explanatory variables \\(X\\)
//! that assumes that there is approximately a linear relationship between \\(X\\) and \\(y\\).
//! Ridge regression is an extension to linear regression that adds L2 regularization term to the loss function during training.
//! This term encourages simpler models that have smaller coefficient values.
//!
//! In ridge regression coefficients \\(\beta_0, \beta_0, ... \beta_n\\) are are estimated by solving
//!
//! \\[\hat{\beta} = (X^TX + \alpha I)^{-1}X^Ty \\]
//!
//! where \\(\alpha \geq 0\\) is a tuning parameter that controls strength of regularization. When \\(\alpha = 0\\) the penalty term has no effect, and ridge regression will produce the least squares estimates.
//! However, as \\(\alpha \rightarrow \infty\\), the impact of the shrinkage penalty grows, and the ridge regression coefficient estimates will approach zero.
//!
//! SmartCore uses [SVD](../../linalg/svd/index.html) and [Cholesky](../../linalg/cholesky/index.html) matrix decomposition to find estimates of \\(\hat{\beta}\\).
//! The Cholesky decomposition is more computationally efficient and more numerically stable than calculating the normal equation directly,
//! but does not work for all data matrices. Unlike the Cholesky decomposition, all matrices have an SVD decomposition.
//!
//! Example:
//!
//! ```
//! use smartcore::linalg::naive::dense_matrix::*;
//! use smartcore::linear::ridge_regression::*;
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
//! let y_hat = RidgeRegression::fit(&x, &y, RidgeRegressionParameters::default().with_alpha(0.1)).
//!                 and_then(|lr| lr.predict(&x)).unwrap();
//! ```
//!
//! ## References:
//!
//! * ["An Introduction to Statistical Learning", James G., Witten D., Hastie T., Tibshirani R., 6.2. Shrinkage Methods](http://faculty.marshall.usc.edu/gareth-james/ISL/)
//! * ["Numerical Recipes: The Art of Scientific Computing",  Press W.H., Teukolsky S.A., Vetterling W.T, Flannery B.P, 3rd ed., Section 15.4 General Linear Least Squares](http://numerical.recipes/)
//!
//! <script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
//! <script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::api::{Predictor, SupervisedEstimator};
use crate::error::Failed;
use crate::linalg::BaseVector;
use crate::linalg::Matrix;
use crate::math::num::RealNumber;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Eq, PartialEq)]
/// Approach to use for estimation of regression coefficients. Cholesky is more efficient but SVD is more stable.
pub enum RidgeRegressionSolverName {
    /// Cholesky decomposition, see [Cholesky](../../linalg/cholesky/index.html)
    Cholesky,
    /// SVD decomposition, see [SVD](../../linalg/svd/index.html)
    SVD,
}

/// Ridge Regression parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RidgeRegressionParameters<T: RealNumber> {
    /// Solver to use for estimation of regression coefficients.
    pub solver: RidgeRegressionSolverName,
    /// Controls the strength of the penalty to the loss function.
    pub alpha: T,
    /// If true the regressors X will be normalized before regression
    /// by subtracting the mean and dividing by the standard deviation.
    pub normalize: bool,
}

/// Ridge Regression grid search parameters
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct RidgeRegressionSearchParameters<T: RealNumber> {
    /// Solver to use for estimation of regression coefficients.
    pub solver: Vec<RidgeRegressionSolverName>,
    /// Regularization parameter.
    pub alpha: Vec<T>,
    /// If true the regressors X will be normalized before regression
    /// by subtracting the mean and dividing by the standard deviation.
    pub normalize: Vec<bool>,
}

/// Ridge Regression grid search iterator
pub struct RidgeRegressionSearchParametersIterator<T: RealNumber> {
    ridge_regression_search_parameters: RidgeRegressionSearchParameters<T>,
    current_solver: usize,
    current_alpha: usize,
    current_normalize: usize,
}

impl<T: RealNumber> IntoIterator for RidgeRegressionSearchParameters<T> {
    type Item = RidgeRegressionParameters<T>;
    type IntoIter = RidgeRegressionSearchParametersIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        RidgeRegressionSearchParametersIterator {
            ridge_regression_search_parameters: self,
            current_solver: 0,
            current_alpha: 0,
            current_normalize: 0,
        }
    }
}

impl<T: RealNumber> Iterator for RidgeRegressionSearchParametersIterator<T> {
    type Item = RidgeRegressionParameters<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_alpha == self.ridge_regression_search_parameters.alpha.len()
            && self.current_solver == self.ridge_regression_search_parameters.solver.len()
        {
            return None;
        }

        let next = RidgeRegressionParameters {
            solver: self.ridge_regression_search_parameters.solver[self.current_solver].clone(),
            alpha: self.ridge_regression_search_parameters.alpha[self.current_alpha],
            normalize: self.ridge_regression_search_parameters.normalize[self.current_normalize],
        };

        if self.current_alpha + 1 < self.ridge_regression_search_parameters.alpha.len() {
            self.current_alpha += 1;
        } else if self.current_solver + 1 < self.ridge_regression_search_parameters.solver.len() {
            self.current_alpha = 0;
            self.current_solver += 1;
        } else if self.current_normalize + 1
            < self.ridge_regression_search_parameters.normalize.len()
        {
            self.current_alpha = 0;
            self.current_solver = 0;
            self.current_normalize += 1;
        } else {
            self.current_alpha += 1;
            self.current_solver += 1;
            self.current_normalize += 1;
        }

        Some(next)
    }
}

impl<T: RealNumber> Default for RidgeRegressionSearchParameters<T> {
    fn default() -> Self {
        let default_params = RidgeRegressionParameters::default();

        RidgeRegressionSearchParameters {
            solver: vec![default_params.solver],
            alpha: vec![default_params.alpha],
            normalize: vec![default_params.normalize],
        }
    }
}

/// Ridge regression
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct RidgeRegression<T: RealNumber, M: Matrix<T>> {
    coefficients: M,
    intercept: T,
    _solver: RidgeRegressionSolverName,
}

impl<T: RealNumber> RidgeRegressionParameters<T> {
    /// Regularization parameter.
    pub fn with_alpha(mut self, alpha: T) -> Self {
        self.alpha = alpha;
        self
    }
    /// Solver to use for estimation of regression coefficients.
    pub fn with_solver(mut self, solver: RidgeRegressionSolverName) -> Self {
        self.solver = solver;
        self
    }
    /// If True, the regressors X will be normalized before regression by subtracting the mean and dividing by the standard deviation.
    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

impl<T: RealNumber> Default for RidgeRegressionParameters<T> {
    fn default() -> Self {
        RidgeRegressionParameters {
            solver: RidgeRegressionSolverName::Cholesky,
            alpha: T::one(),
            normalize: true,
        }
    }
}

impl<T: RealNumber, M: Matrix<T>> PartialEq for RidgeRegression<T, M> {
    fn eq(&self, other: &Self) -> bool {
        self.coefficients == other.coefficients
            && (self.intercept - other.intercept).abs() <= T::epsilon()
    }
}

impl<T: RealNumber, M: Matrix<T>> SupervisedEstimator<M, M::RowVector, RidgeRegressionParameters<T>>
    for RidgeRegression<T, M>
{
    fn fit(
        x: &M,
        y: &M::RowVector,
        parameters: RidgeRegressionParameters<T>,
    ) -> Result<Self, Failed> {
        RidgeRegression::fit(x, y, parameters)
    }
}

impl<T: RealNumber, M: Matrix<T>> Predictor<M, M::RowVector> for RidgeRegression<T, M> {
    fn predict(&self, x: &M) -> Result<M::RowVector, Failed> {
        self.predict(x)
    }
}

impl<T: RealNumber, M: Matrix<T>> RidgeRegression<T, M> {
    /// Fits ridge regression to your data.
    /// * `x` - _NxM_ matrix with _N_ observations and _M_ features in each observation.
    /// * `y` - target values
    /// * `parameters` - other parameters, use `Default::default()` to set parameters to default values.
    pub fn fit(
        x: &M,
        y: &M::RowVector,
        parameters: RidgeRegressionParameters<T>,
    ) -> Result<RidgeRegression<T, M>, Failed> {
        //w = inv(X^t X + alpha*Id) * X.T y

        let (n, p) = x.shape();

        if n <= p {
            return Err(Failed::fit(
                "Number of rows in X should be >= number of columns in X",
            ));
        }

        if y.len() != n {
            return Err(Failed::fit("Number of rows in X should = len(y)"));
        }

        let y_column = M::from_row_vector(y.clone()).transpose();

        let (w, b) = if parameters.normalize {
            let (scaled_x, col_mean, col_std) = Self::rescale_x(x)?;
            let x_t = scaled_x.transpose();
            let x_t_y = x_t.matmul(&y_column);
            let mut x_t_x = x_t.matmul(&scaled_x);

            for i in 0..p {
                x_t_x.add_element_mut(i, i, parameters.alpha);
            }

            let mut w = match parameters.solver {
                RidgeRegressionSolverName::Cholesky => x_t_x.cholesky_solve_mut(x_t_y)?,
                RidgeRegressionSolverName::SVD => x_t_x.svd_solve_mut(x_t_y)?,
            };

            for (i, col_std_i) in col_std.iter().enumerate().take(p) {
                w.set(i, 0, w.get(i, 0) / *col_std_i);
            }

            let mut b = T::zero();

            for (i, col_mean_i) in col_mean.iter().enumerate().take(p) {
                b += w.get(i, 0) * *col_mean_i;
            }

            let b = y.mean() - b;

            (w, b)
        } else {
            let x_t = x.transpose();
            let x_t_y = x_t.matmul(&y_column);
            let mut x_t_x = x_t.matmul(x);

            for i in 0..p {
                x_t_x.add_element_mut(i, i, parameters.alpha);
            }

            let w = match parameters.solver {
                RidgeRegressionSolverName::Cholesky => x_t_x.cholesky_solve_mut(x_t_y)?,
                RidgeRegressionSolverName::SVD => x_t_x.svd_solve_mut(x_t_y)?,
            };

            (w, T::zero())
        };

        Ok(RidgeRegression {
            intercept: b,
            coefficients: w,
            _solver: parameters.solver,
        })
    }

    fn rescale_x(x: &M) -> Result<(M, Vec<T>, Vec<T>), Failed> {
        let col_mean = x.mean(0);
        let col_std = x.std(0);

        for (i, col_std_i) in col_std.iter().enumerate() {
            if (*col_std_i - T::zero()).abs() < T::epsilon() {
                return Err(Failed::fit(&format!(
                    "Cannot rescale constant column {}",
                    i
                )));
            }
        }

        let mut scaled_x = x.clone();
        scaled_x.scale_mut(&col_mean, &col_std, 0);
        Ok((scaled_x, col_mean, col_std))
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
    use crate::metrics::mean_absolute_error;

    #[test]
    fn search_parameters() {
        let parameters = RidgeRegressionSearchParameters {
            alpha: vec![0., 1.],
            ..Default::default()
        };
        let mut iter = parameters.into_iter();
        assert_eq!(iter.next().unwrap().alpha, 0.);
        assert_eq!(
            iter.next().unwrap().solver,
            RidgeRegressionSolverName::Cholesky
        );
        assert!(iter.next().is_none());
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[test]
    fn ridge_fit_predict() {
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

        let y_hat_cholesky = RidgeRegression::fit(
            &x,
            &y,
            RidgeRegressionParameters {
                solver: RidgeRegressionSolverName::Cholesky,
                alpha: 0.1,
                normalize: true,
            },
        )
        .and_then(|lr| lr.predict(&x))
        .unwrap();

        assert!(mean_absolute_error(&y_hat_cholesky, &y) < 2.0);

        let y_hat_svd = RidgeRegression::fit(
            &x,
            &y,
            RidgeRegressionParameters {
                solver: RidgeRegressionSolverName::SVD,
                alpha: 0.1,
                normalize: false,
            },
        )
        .and_then(|lr| lr.predict(&x))
        .unwrap();

        assert!(mean_absolute_error(&y_hat_svd, &y) < 2.0);
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

        let lr = RidgeRegression::fit(&x, &y, Default::default()).unwrap();

        let deserialized_lr: RidgeRegression<f64, DenseMatrix<f64>> =
            serde_json::from_str(&serde_json::to_string(&lr).unwrap()).unwrap();

        assert_eq!(lr, deserialized_lr);
    }
}
