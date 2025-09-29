## Next Target

- **Goal:** Finish porting the tick-array bitmap state so the pool lifecycle mirrors Raydium’s implementation.

## Current Blocker

- **Problem:** The tick-array bitmap code relies on Raydium’s math utilities (`tick_math`, `tick_array_bit_map`, `big_num`, etc.). Without those libraries, the bitmap state won’t compile or behave correctly.

## Way Forward

- **Solution:** First port the essential math helpers from `raydium-clmm/programs/amm/src/libraries` (starting with `tick_math` and the bitmap utilities). Once those foundations are in place, we can bring over the tick-array bitmap/extension state and hook it into our code.