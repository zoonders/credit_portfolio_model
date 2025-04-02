# Credit Portfolio Model
A rust implementaiton of a credit portfolio model. 

[![Crates.io](https://img.shields.io/crates/v/credit_portfolio_model)](https://crates.io/crates/credit_portfolio_model)
[![Docs.rs](https://docs.rs/credit_portfolio_model/badge.svg)](https://docs.rs/credit_portfolio_model)
[![Build Status](https://github.com/zoonders/credit_portfolio_model/workflows/CI/badge.svg)](https://github.com/zoonders/credit_portfolio_model/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

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
* `borrower.csv`
** `borrower_id` - Unique identifier of Borrower ID (string-like) that is used to map with other files
** `risk_group` - Unique identifier of Risk Group ID (string-like). Risk Groups share one of the idiosyncratic risk drivers
** `rating` - Current rating, given as index to a vector, i.e. starting at `0` and continuous.
    The last rating class is considered default, although the current implementation does no special treatment of defaults
** `r2` - Correlation to the systematic risk factor, i.e. $\rho$.
** `eps` - Correlation to the risk group, i.e. $\epsilon$.
* `exposure.csv`

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/credit_portfolio_model).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

