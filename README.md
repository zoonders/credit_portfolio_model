Credit Portfolio Model Implementation in rust.

This crate creates a binary, that can be used with csv-input and outputs the loss distribution quantiles.

The underlying model is a Gaussian copula model allowing multiple (correlated) normal variables and idiosyncratic random variables.
One idiosyncratic variable is shared within a risk group and one is individual to the borrower.
