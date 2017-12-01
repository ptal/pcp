Constraint Programming Library
==============================

[![ptal on Travis CI][travis-image]][travis]

[travis-image]: https://travis-ci.org/ptal/pcp.png
[travis]: https://travis-ci.org/ptal/pcp

Compiled on the nightly channel of Rust. Use [rustup](http://www.rustup.rs) for managing compiler channels. Download the exact same version of the compiler used for PCP with `rustup override add nightly-2017-11-28`.

PCP Constraint Programming library will make you feel NP like P.

Or not. It is first designed to elegantly solve the *entailment problem* in a generic framework. I also explore a "store"-based design where the variables, constraints and spaces are stored in concrete structures – I feel that this design will be clearer but it is highly experimental.

### References

Existing library are an invaluable source of inspiration:

* The [csar](https://github.com/soli/csar) Rust library by Sylvain Soliman helped me to get started.
* And for a lot of reasons the following libraries are also interesting: [GeCode](http://www.gecode.org/), [Choco](http://choco.sourceforge.net/), [Minion](http://minion.sourceforge.net/),...

Most of this work has its roots in the following research works:

* Guido Tack. *Constraint Propagation - Models, Techniques, Implementation*. Doctoral dissertation, Saarland University, 2009.
* Tom Schrijvers, Peter Stuckey, and Philip Wadler. Monadic constraint programming. *J. Funct. Program.*, 19(6):663–697, 2009.
* Tom Schrijvers, Guido Tack, Pieter Wuille, Horst Samulowitz, and Peter J. Stuckey. Search combinators. *Constraints*, 18(2):269–305, 2013.
* ...
