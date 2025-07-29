进度追踪是什么？ 


1. **Added New Progress Tracking Fields to BondingCurve struct**:
   - `total_sales_lamports`: Tracks the total SOL raised
   - `total_trades`: Counts the number of transactions
   - `percent_sold`: Percentage of token supply sold (0-100%)
   - `last_trade_slot`: Timestamp (in slots) of last trade
   - `sales_velocity`: Average SOL raised per day (rolling)

2. **Initialized Progress Tracking Fields in create_bonding_curve**:
   - Set initial values to zero
   - Added status messaging to show the initial state
   - Calculated initial token allocation percentages
   - Documented the completion threshold criteria

3. **Updated Progress in the apply_buy Function**:
   - Incremented total sales amount when purchases occur
   - Incremented trade counter
   - Calculated and updated percentage sold
   - Updated timestamp of last trade
   - Implemented a sophisticated sales velocity calculation:
     - Uses weighted average (70% previous, 30% new)
     - Normalizes to a daily rate based on slot timing
     - Handles edge cases (first trade, long gaps)

This implementation provides comprehensive progress tracking for the bonding curve, enabling:
- Monitoring of sales progress
- Analysis of sales velocity
- Clear indicators of completion state
- Historical data on trading activity

The tracking updates automatically with each trade and provides an accurate picture of how close the bonding curve is to completion.