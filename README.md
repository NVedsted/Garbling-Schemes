# Garbling Schemes
**NOT SAFE FOR USE**

This project explores the performance of Yao's garbling scheme
and the garbling scheme introduced by Zahur et al. (2014) by running
multiple circuits from Smart et al. (n.d.). It was initially intended
to compare multiple advanced garbling schemes, but time constraint
and unforeseen consequences has led to a simpler project which in turn
has led to non-surprising results that show the much more advanced
scheme heavily outperforms Yao's garbling scheme.

## Running
To benchmark the implementations run:
```
cargo bench
```

## Limitations
Encoding and decoding does not support partial application. One must
at the moment have the entire input or the entire output to do the
operations. This will need to be amended for any practical use.

The half-gates garbling schemes does not fully-implement the required
operations to ensure security.

Neither scheme implementation has been rigorously tested for correctness,
completeness, nor security.

## References
Bellare, M., Hoang, V. T., Keelveedhi, S., & Rogaway, P. (2013). Efficient
Garbling from a Fixed-Key Blockcipher. Retrieved December 11, 2021,
from https://ia.cr/2013/426

Kolesnikov, V., & Schneider, T. (2008). Improved Garbled Circuit: Free XOR
Gates and Applications. Retrieved December 11, 2021, from http :
//www.cs.toronto.edu/~vlad/papers/XOR ICALP08.pdf

Orlandi, C., Scholl, P., & Baum, C. (2021). Lecture Notes for Cryptographic
Computing: 5. Garbled Circuits.
15
Smart, N., Sijacic, D., Mertens, N., Maene, P., Lu, S., Abril, V. A., & Archer,
D. (n.d.). 'Bristol Fashion' MPC Circuits. Retrieved December 11,
2021, from https://homes.esat.kuleuven.be/~nsmart/MPC/

Zahur, S., Rosulek, M., & Evans, D. (2014). Two Halves Make a Whole: Re-
ducing Data Transfer in Garbled Circuits using Half Gates. Retrieved
December 11, 2021, from https://ia.cr/2014/756
