Constraint Programming Library
==============================

[![CI](https://github.com/ptal/pcp/actions/workflows/ci.yml/badge.svg)](https://github.com/ptal/pcp/actions/workflows/ci.yml)

PCP is a library to model constraint satisfaction problems (CSPs) in the constraint programming paradigm.
Constraint programming is a paradigm for expressing problems in terms of mathematical relations, called constraints, over variables (e.g. `x > y`).
Constraints are an intuitive approach to naturally describe many real-world problems which initially emerged as a subfield of artificial intelligence and operational research.
The flagship applications in constraint programming encompass scheduling, configuration and vehicles routing problems.
Constraints are also applied to various other domains such as in music, biology and model checking.

It is a programming paradigm in its own right and being able to program in Rust is not sufficient to master the art of constraint programming.
This is why I strongly recommend the class [Basic Modeling for Discrete Optimization](https://www.coursera.org/learn/basic-modeling) for those who wants to learn this paradigm.
Meanwhile you can already look at the [documentation](https://docs.rs/crate/libpcp) for a running example.

PCP compiles on the *stable Rust* channel.

### References

Existing libraries are an invaluable source of inspiration:

* The [csar](https://github.com/soli/csar) Rust library by Sylvain Soliman helped me to get started.
* And for a lot of reasons the following libraries are also interesting: [GeCode](http://www.gecode.org/), [Choco](http://choco.sourceforge.net/), [Minion](http://minion.sourceforge.net/),...

Most of this work has its roots in the following research works:

* Guido Tack. *Constraint Propagation - Models, Techniques, Implementation*. Doctoral dissertation, Saarland University, 2009.
* Tom Schrijvers, Peter Stuckey, and Philip Wadler. Monadic constraint programming. *J. Funct. Program.*, 19(6):663–697, 2009.
* Tom Schrijvers, Guido Tack, Pieter Wuille, Horst Samulowitz, and Peter J. Stuckey. Search combinators. *Constraints*, 18(2):269–305, 2013.
* ...
