# Credit Portfolio Model
A rust implementaiton of a credit portfolio model. 

[![Crates.io](https://img.shields.io/crates/v/credit_portfolio_model)](https://crates.io/crates/credit_portfolio_model)
[![Docs.rs](https://docs.rs/credit_portfolio_model/badge.svg)](https://docs.rs/credit_portfolio_model)
[![Build Status](https://img.shields.io/github/actions/workflow/status/zoonders/credit_portfolio_model/rust.yml)](https://github.com/zoonders/credit_portfolio_model/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features
This is an early implementation of a credit portfolio model that simulates correlated rating migrations in a portfolio. The
correlation structure is based on a Merton-type factor model. Rating migrations are based on the asset value $z$ exceeding
a certain threshold.
```math
z>c
```
The random variable `z` is standard normally distributed. Hence, the thresholds are calculated from pre-defined migration or default probabilities
using the cumulative normal distribution. The asset value $z$ itself is a linear combination of multiple standard normal random variables
```math
z=\sqrt{r^2}\cdot y+\sqrt{1-r^2}\cdot\left(\sqrt{1-\epsilon}\cdot e_1 + \sqrt{\epsilon}\cdot e_2\right)
```
Here, the variables are defined as follows:
* $`e_1`$ is the idiosyncratic risk variable only corresponding to the borrower
* $`e_2`$ is the idiosyncratic risk variable of the risk group that the borrower belongs to, e.g. linked to other companies through ownership
* $`y`$ is the systematic risk, which is defined as a linear combination of the vector $`x`$ which is distributed according to a multivariate
normal distribution with covariance matrix $`\Sigma`$. Besides, the borrower dependency to the risk drives is specified by weights $`\phi`$.
The resulting random variable is then defined as $`y=\frac{\phi\cdot x}{\sqrt{\phi^T\cdot\Sigma\cdot\phi}}`$.
* $`r^2`$ and $\epsilon$ specifiy the correlation of the borrowers asset value to the systematic component or the risk group.

The following features are implemented:
* Migration mode
* Risk groups
* Multi-threading

Features that could be implemented, but are not at the moment
* Expected shortfall calculations and other risk measures
* Alternative sampling methods for better tail measure sampling
* Random losses based on random process in case of default

## Installation
Install the package via

```sh
cargo install credit_portfolio_model
```

## Usage
The crate comes with a binary that reads csv-data with the portfolio information and outputs the loss-distribution.
Besides, Mean and quantile information of the loss distribution is provided on stdout.

```sh
credit_portfolio_model --input /path/to/read/input/csv/files/from --output /path/to/store/output/csv --num-trials NUMBERTRIALS --chunk-size TRIALSPERTHREAD
```

The input consists of six files. The files are

`correlation_matrix.csv`
* `risk_factor_1`, `risk_factor_2` - Number index of column and row, respectively. Starts with 0 and must be continuous
* `correlation` - Correlation value. Note that the matrix needs to be symmetric and positive semi-definite

`borrower.csv`
* `borrower_id` - Unique identifier of Borrower (string-like) that is used to map with other files
* `risk_group` - Unique identifier of Risk Group (string-like). Risk Groups share one of the idiosyncratic risk drivers
* `rating` - Current rating, given as index to a vector, i.e. starting at `0` and continuous.
    The last rating class is considered default, although the current implementation does no special treatment of defaults
* `r2` - Correlation to the systematic risk factor, i.e. $\rho$.
* `eps` - Correlation to the risk group, i.e. $\epsilon$.

`risk_factors.csv`
* `borrower_id` - See borrower, must match the other file
* `risk_factor` - Mapping to risk factor of correlation factor
* `weight` - Borrower dependency to the risk factor (relative to the other risk factors)

`transition_probabilities.csv`
* `borrower_id` - See borrower, must match the other file
* `rating` - Resulting rating class
* `probabiliy` - Probability to migrate into this class, must sum to 100%.

`exposure.csv`
* `exposure_id` - Unique identifier of Exposure (string-like) that is used to map with other files
* `borrower_id` - See borrower, must match the other file
* `outstanding` - Current outstanding, not used at the moment

`valuations.csv`
* `exposure_id` - See exposure, must match the other file
* `rating` - Resulting rating class
* `valuation` - Valuation of the exposure given the rating class.

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/credit_portfolio_model).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

